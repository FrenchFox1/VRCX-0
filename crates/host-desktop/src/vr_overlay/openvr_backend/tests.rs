use super::*;
use vrcx_0_vr_overlay::OverlaySize;

#[test]
fn visible_surface_releases_pending_frame_when_upload_interval_elapses() {
    let started_at = Instant::now();
    let frame = RgbaFrame::new(OverlaySize::new(1, 1), vec![255, 255, 255, 255]);
    let fingerprint = frame_fingerprint(&frame);
    let mut surface = test_main_surface(frame.clone(), started_at);

    assert!(surface
        .take_pending_frame_if_due(
            started_at + MAIN_VISIBLE_FRAME_UPLOAD_INTERVAL - Duration::from_millis(1)
        )
        .is_none());
    assert_eq!(
        surface
            .pending_frame
            .as_ref()
            .map(|pending_frame| &pending_frame.frame),
        Some(&frame)
    );

    let (_, released_pending_frame) = surface
        .take_pending_frame_if_due(started_at + MAIN_VISIBLE_FRAME_UPLOAD_INTERVAL)
        .expect("release pending frame at the upload deadline");
    assert_eq!(released_pending_frame.frame, frame);
    assert_eq!(released_pending_frame.fingerprint, fingerprint);
    assert!(surface.pending_frame.is_none());
    assert_eq!(
        surface.last_visible_frame_upload_at,
        Some(started_at + MAIN_VISIBLE_FRAME_UPLOAD_INTERVAL)
    );
}

#[test]
fn pending_main_frame_requests_high_frequency_tick_until_released() {
    let started_at = Instant::now();
    let frame = RgbaFrame::new(OverlaySize::new(1, 1), vec![255, 255, 255, 255]);
    let surface_id = OverlaySurfaceId::new(MAIN_SURFACE_ID);
    let mut backend = OpenVrOverlayBackend::new();
    backend
        .surfaces
        .insert(surface_id.clone(), test_main_surface(frame, started_at));

    assert!(backend.needs_high_frequency_tick());
    backend
        .surfaces
        .get_mut(&surface_id)
        .expect("main surface")
        .take_pending_frame_if_due(started_at + MAIN_VISIBLE_FRAME_UPLOAD_INTERVAL)
        .expect("release pending frame");
    assert!(!backend.needs_high_frequency_tick());
}

#[test]
fn raw_completions_keep_high_frequency_tick_until_all_submissions_finish() {
    let surface_id = OverlaySurfaceId::new(MAIN_SURFACE_ID);
    let mut backend = OpenVrOverlayBackend::new();
    backend.outstanding_raw_frames = 2;

    assert!(backend.needs_high_frequency_tick());
    assert!(!backend.handle_overlay_event(
        &surface_id,
        EventInfo {
            tracked_device_index: tracked_device_index::INVALID,
            age: 0.01,
            event: Event::ImageLoaded,
        },
    ));
    assert_eq!(backend.outstanding_raw_frames, 1);
    assert!(backend.needs_high_frequency_tick());

    assert!(!backend.handle_overlay_event(
        &surface_id,
        EventInfo {
            tracked_device_index: tracked_device_index::INVALID,
            age: 0.02,
            event: Event::ImageFailed,
        },
    ));
    assert_eq!(backend.outstanding_raw_frames, 0);
    assert!(!backend.needs_high_frequency_tick());
}

#[test]
fn overlay_quit_event_requests_runtime_shutdown() {
    let mut backend = OpenVrOverlayBackend::new();

    assert!(backend.handle_overlay_event(
        &OverlaySurfaceId::new(MAIN_SURFACE_ID),
        EventInfo {
            tracked_device_index: tracked_device_index::INVALID,
            age: 0.0,
            event: Event::Quit(openvr::system::event::Process {
                pid: 0,
                old_pid: 0,
                forced: false,
            }),
        },
    ));
}

fn test_main_surface(frame: RgbaFrame, last_uploaded_at: Instant) -> OpenVrSurface {
    let fingerprint = frame_fingerprint(&frame);
    OpenVrSurface {
        handle: OverlayHandle(1),
        config: OverlaySurfaceConfig {
            surface_id: OverlaySurfaceId::new(MAIN_SURFACE_ID),
            size: frame.size,
            physical_width_meters: 1.0,
            placement: OverlayPlacement::TrackedDeviceRelative {
                device_hint: "hmd".to_string(),
            },
            activation_button: OverlayActivationButton::Grip,
            interactive: false,
        },
        transform_device: None,
        policy: WristVisibilityPolicy::default(),
        visible: true,
        active: true,
        pending_frame: Some(PendingFrame { frame, fingerprint }),
        last_uploaded_frame_fingerprint: None,
        last_visible_frame_upload_at: Some(last_uploaded_at),
        current_alpha: 1.0,
        target_alpha: 1.0,
        fade: None,
        hide_after_fade: false,
    }
}

#[test]
fn openvr_context_lease_blocks_concurrent_owners_until_release() {
    let first = OpenVrContextLease::acquire().expect("acquire first OpenVR context lease");
    let error = std::thread::spawn(OpenVrContextLease::acquire)
        .join()
        .expect("join competing lease thread")
        .expect_err("reject a second OpenVR context owner");

    assert_eq!(
        error,
        BackendStartError::transient(OPENVR_CONTEXT_IN_USE_MESSAGE)
    );

    drop(first);
    std::thread::spawn(OpenVrContextLease::acquire)
        .join()
        .expect("join replacement lease thread")
        .expect("acquire OpenVR context lease after release");
}

#[test]
fn panel_summon_uses_fixed_right_hand_friends_grip_hold() {
    assert_eq!(PANEL_SUMMON_HAND, OverlayHand::Right);
    assert_eq!(PANEL_SUMMON_PANEL_ID, FRIENDS_PANEL_ID);
    assert_eq!(SUMMON_HOLD_DURATION, Duration::from_secs(2));
}

#[test]
fn friends_panel_input_path_is_disabled_by_default() {
    const { assert!(!FRIENDS_PANEL_INPUT_ENABLED) };
}

#[test]
fn validate_frame_rejects_mismatched_rgba_length() {
    let frame = RgbaFrame::new(OverlaySize::new(2, 2), vec![0; 15]);

    assert!(validate_frame(&frame).is_err());
}

#[test]
fn panel_summon_hold_emits_once_and_resets_after_release() {
    let started = Instant::now();
    let mut state = PanelSummonGestureState::default();

    assert!(!update_panel_summon_hold(&mut state, false, started));
    assert!(!update_panel_summon_hold(&mut state, true, started));
    assert!(!update_panel_summon_hold(
        &mut state,
        true,
        started + SUMMON_HOLD_DURATION - Duration::from_millis(1)
    ));
    assert!(update_panel_summon_hold(
        &mut state,
        true,
        started + SUMMON_HOLD_DURATION
    ));
    assert!(!update_panel_summon_hold(
        &mut state,
        true,
        started + SUMMON_HOLD_DURATION + Duration::from_secs(1)
    ));

    assert!(!update_panel_summon_hold(
        &mut state,
        false,
        started + SUMMON_HOLD_DURATION + Duration::from_secs(2)
    ));
    let restarted = started + SUMMON_HOLD_DURATION + Duration::from_secs(3);
    assert!(!update_panel_summon_hold(&mut state, true, restarted));
    assert!(update_panel_summon_hold(
        &mut state,
        true,
        restarted + SUMMON_HOLD_DURATION
    ));
}
