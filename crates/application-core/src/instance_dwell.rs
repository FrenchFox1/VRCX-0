use std::collections::HashMap;
use std::sync::Mutex;

use vrcx_0_application_contracts::InstanceRosterSnapshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceDwellSource {
    GameLog,
    Observed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InstanceDwellEntry {
    location: String,
    since_ms: i64,
    source: InstanceDwellSource,
}

#[derive(Debug, Default)]
struct InstanceDwellState {
    presence: HashMap<String, InstanceDwellEntry>,
    roster_location: String,
    roster_joins: HashMap<String, i64>,
}

#[derive(Debug, Default)]
pub struct InstanceDwellRegistry {
    state: Mutex<InstanceDwellState>,
}

fn normalized(value: &str) -> &str {
    value.trim()
}

impl InstanceDwellRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe_roster(&self, snapshot: &InstanceRosterSnapshot) {
        let location = normalized(&snapshot.location).to_string();
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());

        state.roster_location = location.clone();
        state.roster_joins.clear();
        if location.is_empty() {
            return;
        }

        for member in &snapshot.members {
            let user_id = normalized(&member.user_id).to_string();
            if user_id.is_empty() {
                continue;
            }
            let Some(joined_at_ms) = member.joined_at_ms.filter(|value| *value > 0) else {
                continue;
            };
            state.roster_joins.insert(user_id.clone(), joined_at_ms);

            let upgradable = state.presence.get(&user_id).is_some_and(|entry| {
                entry.location == location
                    && entry.source == InstanceDwellSource::Observed
                    && joined_at_ms < entry.since_ms
            });
            if upgradable {
                state.presence.insert(
                    user_id,
                    InstanceDwellEntry {
                        location: location.clone(),
                        since_ms: joined_at_ms,
                        source: InstanceDwellSource::GameLog,
                    },
                );
            }
        }
    }

    pub fn observe_presence_location(
        &self,
        user_id: &str,
        location: &str,
        observed_ms: i64,
    ) -> i64 {
        let user_id = normalized(user_id).to_string();
        let location = normalized(location).to_string();
        if user_id.is_empty() || location.is_empty() {
            return observed_ms;
        }

        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = state.presence.get(&user_id) {
            if entry.location == location {
                return entry.since_ms;
            }
        }

        let roster_join = if state.roster_location == location {
            state.roster_joins.get(&user_id).copied()
        } else {
            None
        };
        let (since_ms, source) = match roster_join {
            Some(joined_at_ms) => (joined_at_ms, InstanceDwellSource::GameLog),
            None => (observed_ms, InstanceDwellSource::Observed),
        };
        state.presence.insert(
            user_id,
            InstanceDwellEntry {
                location,
                since_ms,
                source,
            },
        );
        since_ms
    }

    pub fn dwell_since_ms(&self, user_id: &str, location: &str) -> Option<i64> {
        let user_id = normalized(user_id);
        let location = normalized(location);
        if user_id.is_empty() || location.is_empty() {
            return None;
        }
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = state.presence.get(user_id) {
            if entry.location == location {
                return Some(entry.since_ms);
            }
        }
        if state.roster_location != location {
            return None;
        }
        state.roster_joins.get(user_id).copied()
    }

    #[cfg(test)]
    pub fn tracked_count(&self) -> (usize, usize) {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        (state.presence.len(), state.roster_joins.len())
    }

    pub fn forget(&self, user_id: &str) {
        let user_id = normalized(user_id);
        if user_id.is_empty() {
            return;
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.presence.remove(user_id);
        state.roster_joins.remove(user_id);
    }

    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.presence.clear();
        state.roster_joins.clear();
        state.roster_location.clear();
    }
}

impl vrcx_0_application_contracts::InstanceRosterObserver for InstanceDwellRegistry {
    fn on_instance_roster(&self, snapshot: InstanceRosterSnapshot) {
        self.observe_roster(&snapshot);
    }

    fn on_game_running(&self, running: bool) {
        if !running {
            self.observe_roster(&InstanceRosterSnapshot::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vrcx_0_application_contracts::InstanceRosterMember;

    fn roster(location: &str, members: &[(&str, i64)]) -> InstanceRosterSnapshot {
        InstanceRosterSnapshot {
            location: location.to_string(),
            world_name: String::new(),
            destination: String::new(),
            entered_at: String::new(),
            members: members
                .iter()
                .map(|(user_id, joined_at_ms)| InstanceRosterMember {
                    user_id: (*user_id).to_string(),
                    display_name: String::new(),
                    joined_at_ms: Some(*joined_at_ms),
                })
                .collect(),
        }
    }

    #[test]
    fn game_log_join_time_becomes_the_dwell_start() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 1_000)]));
        assert_eq!(registry.dwell_since_ms("usr_a", "wrld_a:1"), Some(1_000));
    }

    #[test]
    fn presence_reuses_the_game_log_join_time_instead_of_the_observed_moment() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 1_000)]));
        assert_eq!(
            registry.observe_presence_location("usr_a", "wrld_a:1", 9_000),
            1_000
        );
    }

    #[test]
    fn a_second_presence_event_in_the_same_instance_does_not_restart_the_timer() {
        let registry = InstanceDwellRegistry::new();
        assert_eq!(
            registry.observe_presence_location("usr_a", "wrld_a:1", 5_000),
            5_000
        );
        assert_eq!(
            registry.observe_presence_location("usr_a", "wrld_a:1", 8_000),
            5_000
        );
    }

    #[test]
    fn game_log_upgrades_an_observed_estimate_to_the_earlier_real_join_time() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_presence_location("usr_a", "wrld_a:1", 5_000);
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 1_000)]));
        assert_eq!(registry.dwell_since_ms("usr_a", "wrld_a:1"), Some(1_000));
    }

    #[test]
    fn game_log_never_pushes_an_established_start_later() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_presence_location("usr_a", "wrld_a:1", 1_000);
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 5_000)]));
        assert_eq!(registry.dwell_since_ms("usr_a", "wrld_a:1"), Some(1_000));
    }

    #[test]
    fn moving_to_another_instance_restarts_the_timer() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_presence_location("usr_a", "wrld_a:1", 1_000);
        assert_eq!(
            registry.observe_presence_location("usr_a", "wrld_b:2", 7_000),
            7_000
        );
        assert_eq!(registry.dwell_since_ms("usr_a", "wrld_a:1"), None);
    }

    #[test]
    fn leaving_the_instance_drops_the_non_friend_start() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 1_000), ("usr_b", 2_000)]));
        registry.observe_roster(&roster("wrld_a:1", &[("usr_b", 2_000)]));
        assert_eq!(registry.dwell_since_ms("usr_a", "wrld_a:1"), None);
        assert_eq!(registry.dwell_since_ms("usr_b", "wrld_a:1"), Some(2_000));
    }

    #[test]
    fn friends_in_remote_instances_survive_a_local_roster_refresh() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_presence_location("usr_remote", "wrld_far:9", 3_000);
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 1_000)]));
        assert_eq!(
            registry.dwell_since_ms("usr_remote", "wrld_far:9"),
            Some(3_000)
        );
    }

    #[test]
    fn a_friend_left_behind_in_the_old_instance_keeps_their_start() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_roster(&roster("wrld_a:1", &[("usr_friend", 1_000)]));
        registry.observe_presence_location("usr_friend", "wrld_a:1", 9_000);
        registry.observe_roster(&roster("wrld_b:2", &[]));
        assert_eq!(
            registry.dwell_since_ms("usr_friend", "wrld_a:1"),
            Some(1_000)
        );
    }

    #[test]
    fn hopping_instances_does_not_accumulate_non_friend_starts() {
        let registry = InstanceDwellRegistry::new();
        for index in 0..50 {
            let location = format!("wrld_{index}:1");
            registry.observe_roster(&roster(
                &location,
                &[("usr_a", 1_000), ("usr_b", 2_000), ("usr_c", 3_000)],
            ));
        }
        assert_eq!(registry.tracked_count(), (0, 3));
    }

    #[test]
    fn leaving_the_world_clears_the_local_roster_and_keeps_presence_starts() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 1_000)]));
        registry.observe_presence_location("usr_remote", "wrld_far:9", 3_000);
        registry.observe_roster(&InstanceRosterSnapshot::default());
        assert_eq!(registry.dwell_since_ms("usr_a", "wrld_a:1"), None);
        assert_eq!(
            registry.dwell_since_ms("usr_remote", "wrld_far:9"),
            Some(3_000)
        );
        assert_eq!(registry.tracked_count(), (1, 0));
    }

    #[test]
    fn forgetting_an_offline_friend_frees_both_tables() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 1_000)]));
        registry.observe_presence_location("usr_a", "wrld_a:1", 9_000);
        registry.forget("usr_a");
        assert_eq!(registry.tracked_count(), (0, 0));
    }

    #[test]
    fn clear_drops_every_tracked_start() {
        let registry = InstanceDwellRegistry::new();
        registry.observe_roster(&roster("wrld_a:1", &[("usr_a", 1_000)]));
        registry.observe_presence_location("usr_b", "wrld_b:2", 2_000);
        registry.clear();
        assert_eq!(registry.tracked_count(), (0, 0));
    }
}
