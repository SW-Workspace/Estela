use estela_model::{Event, EventKind, Time, Zoom};

const TRANSITION: f64 = 700_000_000.0;
const FOLLOW_TIME: f64 = 180_000_000.0;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

pub fn sample(zooms: &[Zoom], events: &[Event], time: Time, size: (u32, u32)) -> Camera {
    let mut camera = Camera {
        x: f64::from(size.0) / 2.0,
        y: f64::from(size.1) / 2.0,
        scale: 1.0,
    };
    let Some(zoom) = zooms
        .iter()
        .find(|zoom| time >= zoom.range.start && time <= zoom.range.end)
    else {
        return camera;
    };
    let enter = (time.0 - zoom.range.start.0) as f64 / TRANSITION;
    let leave = (zoom.range.end.0 - time.0) as f64 / TRANSITION;
    let progress = enter.min(leave).clamp(0.0, 1.0);
    let ease = progress * progress * (3.0 - 2.0 * progress);
    camera.scale = 1.0 + (zoom.level - 1.0) * ease;
    let end = events.partition_point(|event| event.t <= time);
    let begin = events[..end].partition_point(|event| event.t.0 < time.0 - 2_000_000_000);
    let mut last_time = events.get(begin).map_or(0, |event| event.t.0);
    if let Some(Event {
        kind: EventKind::Move { x, y },
        ..
    }) = events.get(begin)
    {
        camera.x = f64::from(*x);
        camera.y = f64::from(*y);
    }
    for event in &events[begin..end] {
        if let EventKind::Move { x, y } = event.kind {
            let delta = (event.t.0 - last_time).max(0) as f64;
            let weight = 1.0 - libm::exp(-delta / FOLLOW_TIME);
            camera.x += (f64::from(x) - camera.x) * weight;
            camera.y += (f64::from(y) - camera.y) * weight;
            last_time = event.t.0;
        }
    }
    let half_width = f64::from(size.0) / (2.0 * camera.scale);
    let half_height = f64::from(size.1) / (2.0 * camera.scale);
    camera.x = camera.x.clamp(half_width, f64::from(size.0) - half_width);
    camera.y = camera.y.clamp(half_height, f64::from(size.1) - half_height);
    camera
}
