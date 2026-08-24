pub(crate) fn is_real_instance(location: &str) -> bool {
    let location = location.trim().to_ascii_lowercase();
    if location.is_empty() || location.starts_with("local") {
        return false;
    }
    !matches!(
        location.as_str(),
        ":" | "offline"
            | "offline:offline"
            | "traveling"
            | "traveling:traveling"
            | "private"
            | "private:private"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_local_and_sentinel_locations() {
        for location in [
            "",
            "local",
            "local:1234",
            ":",
            "offline",
            "offline:offline",
            "private",
            "private:private",
            "traveling",
            "traveling:traveling",
        ] {
            assert!(!is_real_instance(location), "{location}");
        }
    }

    #[test]
    fn accepts_world_instance_tags() {
        for location in ["wrld_a:12345", "wrld_a:12345~region(us)", "wrld_a"] {
            assert!(is_real_instance(location), "{location}");
        }
    }

    #[test]
    fn trims_whitespace_and_folds_case_before_matching_sentinels() {
        assert!(!is_real_instance("  offline  "));
        assert!(!is_real_instance("OFFLINE"));
        assert!(!is_real_instance("Private:Private"));
        assert!(!is_real_instance("  Traveling  "));
    }
}
