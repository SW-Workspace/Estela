use estela_model::{ElementId, Event, EventKind, Time, TimeRange, Zoom, ZoomMode};

const GROUP_GAP: i64 = 2_500_000_000;
const ANTICIPATION: i64 = 600_000_000;
const HOLD: i64 = 1_500_000_000;
const MERGE_GAP: i64 = 1_000_000_000;
const GROUP_DISTANCE: f64 = 0.25;

pub fn generate(events: &[Event], size: (u32, u32), duration: Time) -> Vec<Zoom> {
    let mut zooms: Vec<Zoom> = Vec::new();
    let mut position = (0, 0);
    let mut previous_click = None;
    let diagonal = libm::hypot(f64::from(size.0), f64::from(size.1));
    for event in events {
        match event.kind {
            EventKind::Move { x, y } => position = (x, y),
            EventKind::Down { .. } => {
                let start = (event.t.0 - ANTICIPATION).max(0);
                let end = (event.t.0 + HOLD).min(duration.0);
                let nearby = previous_click.is_some_and(|(time, x, y): (i64, i32, i32)| {
                    let distance =
                        libm::hypot(f64::from(position.0 - x), f64::from(position.1 - y));
                    event.t.0 - time <= GROUP_GAP && distance <= diagonal * GROUP_DISTANCE
                });
                if let Some(last) = zooms
                    .last_mut()
                    .filter(|last| nearby || start - last.range.end.0 < MERGE_GAP)
                {
                    last.range.end = Time(end);
                } else if end > start {
                    zooms.push(Zoom {
                        id: ElementId::new(),
                        range: TimeRange {
                            start: Time(start),
                            end: Time(end),
                        },
                        level: 2.0,
                        mode: ZoomMode::FollowCursor,
                    });
                }
                previous_click = Some((event.t.0, position.0, position.1));
            }
            _ => {}
        }
    }
    zooms
}

pub fn generate_from_dwell(events: &[Event], size: (u32, u32), duration: Time) -> Vec<Zoom> {
    use estela_model::Button;

    const DWELL: i64 = 600_000_000;
    const TRAVEL: f64 = 80.0;
    const STABLE_RADIUS: f64 = 20.0;
    const COOLDOWN: i64 = 3_000_000_000;
    let mut anchor = None;
    let mut last_focus = None;
    let mut focused_at = -COOLDOWN;
    let mut synthetic = Vec::new();
    for event in events {
        let EventKind::Move { x, y } = event.kind else {
            continue;
        };
        synthetic.push(event.clone());
        if anchor.is_none() {
            last_focus = Some((x, y));
        }
        let (since, origin_x, origin_y) = *anchor.get_or_insert((event.t.0, x, y));
        if libm::hypot(f64::from(x - origin_x), f64::from(y - origin_y)) > STABLE_RADIUS {
            anchor = Some((event.t.0, x, y));
            continue;
        }
        let moved = last_focus.is_none_or(|(last_x, last_y)| {
            libm::hypot(f64::from(x - last_x), f64::from(y - last_y)) >= TRAVEL
        });
        if moved && event.t.0 - since >= DWELL && event.t.0 - focused_at >= COOLDOWN {
            synthetic.push(Event {
                t: event.t,
                kind: EventKind::Down {
                    button: Button::Left,
                },
            });
            last_focus = Some((x, y));
            focused_at = event.t.0;
        }
    }
    generate(&synthetic, size, duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use estela_model::Button;

    #[test]
    fn groups_clicks_and_clamps_to_recording() {
        let events = vec![
            Event {
                t: Time(0),
                kind: EventKind::Move { x: 30, y: 40 },
            },
            Event {
                t: Time(200_000_000),
                kind: EventKind::Down {
                    button: Button::Left,
                },
            },
            Event {
                t: Time(1_000_000_000),
                kind: EventKind::Down {
                    button: Button::Left,
                },
            },
            Event {
                t: Time(6_000_000_000),
                kind: EventKind::Down {
                    button: Button::Left,
                },
            },
        ];
        let zooms = generate(&events, (1920, 1080), Time(7_000_000_000));
        assert_eq!(zooms.len(), 2);
        assert_eq!(
            zooms[0].range,
            TimeRange {
                start: Time(0),
                end: Time(2_500_000_000)
            }
        );
        assert_eq!(zooms[1].range.end, Time(7_000_000_000));
    }

    #[test]
    fn movement_alone_does_not_invent_clicks() {
        let events = [Event {
            t: Time(0),
            kind: EventKind::Move { x: 1, y: 1 },
        }];
        assert!(generate(&events, (100, 100), Time(1_000_000_000)).is_empty());
    }

    #[test]
    fn dwell_requires_movement_and_a_stable_cursor() {
        let events = [
            Event {
                t: Time(0),
                kind: EventKind::Move { x: 10, y: 10 },
            },
            Event {
                t: Time(1_000_000_000),
                kind: EventKind::Move { x: 10, y: 10 },
            },
            Event {
                t: Time(2_000_000_000),
                kind: EventKind::Move { x: 300, y: 300 },
            },
            Event {
                t: Time(2_700_000_000),
                kind: EventKind::Move { x: 301, y: 300 },
            },
        ];
        assert!(generate_from_dwell(&events[..2], (1920, 1080), Time(4_000_000_000)).is_empty());
        let zooms = generate_from_dwell(&events, (1920, 1080), Time(4_000_000_000));
        assert_eq!(zooms.len(), 1);
        assert_eq!(zooms[0].range.start, Time(2_100_000_000));
        assert!(events
            .iter()
            .all(|event| !matches!(event.kind, EventKind::Down { .. })));
    }
}
