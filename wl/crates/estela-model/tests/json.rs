use estela_model::*;

#[test]
fn unknown_fields_survive_roundtrip() {
    let value = serde_json::json!({
        "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "format": {"major": 1, "minor": 0},
        "recording": {
            "size": {"width": 1920.0, "height": 1080.0}, "scale": 1.0,
            "color": {"primaries": "Bt709", "range": "Limited"},
            "audio_offset": 0, "duration": 0, "recorded_at": "2026-11-09T00:00:00Z",
            "platform": "linux", "video_codec": "h264", "has_audio": false,
            "app_version": "0.1.0"
        },
        "meta": {"title": "Test", "tags": ["Example"]},
        "edit": {
            "clips": [], "zooms": [], "cursor": {"visible": true, "scale": 1.0},
            "background": {"Solid": [0.0, 0.0, 0.0, 1.0]},
            "export": {
                "size": {"width": 1920.0, "height": 1080.0},
                "fps": 60.0, "bitrate_kbps": 8000
            },
            "future_edit": {"enabled": true}
        },
        "future_project": [1, 2, 3]
    });
    let project: Project = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(project).unwrap(), value);
}

#[test]
fn all_event_variants_roundtrip() {
    let kinds = [
        EventKind::Move { x: -10, y: 20 },
        EventKind::Down {
            button: Button::Left,
        },
        EventKind::Up {
            button: Button::Other(4),
        },
        EventKind::CursorShape { id: CursorId(1) },
        EventKind::Dropped,
    ];
    for kind in kinds {
        let event = Event {
            t: Time(i64::MAX),
            kind,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(serde_json::from_str::<Event>(&json).unwrap(), event);
    }
}

#[test]
fn enum_states_preserve_boolean_json() {
    for value in [false, true] {
        let visibility: Visibility = serde_json::from_value(value.into()).unwrap();
        let audio: AudioPresence = serde_json::from_value(value.into()).unwrap();
        assert_eq!(serde_json::to_value(visibility).unwrap(), value);
        assert_eq!(serde_json::to_value(audio).unwrap(), value);
        assert_eq!(visibility == Visibility::Visible, value);
        assert_eq!(audio == AudioPresence::Present, value);
    }
}
