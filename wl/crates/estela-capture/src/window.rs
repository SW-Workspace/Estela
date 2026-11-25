#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub radius: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowInput {
    Entire,
    Areas(Vec<InputArea>),
}

impl InputArea {
    pub fn rectangles(self, scale: f32) -> Vec<Self> {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        let x = (self.x as f32 * scale).round() as i32;
        let y = (self.y as f32 * scale).round() as i32;
        let width = (self.width as f32 * scale).round() as u32;
        let height = (self.height as f32 * scale).round() as u32;
        let radius = ((self.radius as f32 * scale).round() as u32)
            .min(width / 2)
            .min(height / 2);
        let mut rectangles = Vec::new();
        if width == 0 || height == 0 {
            return rectangles;
        }
        rectangles.push(Self {
            x,
            y: y + radius as i32,
            width,
            height: height - 2 * radius,
            radius: 0,
        });
        for row in 0..radius {
            let distance = radius - row - 1;
            let inset = radius
                - (u64::from(radius) * u64::from(radius)
                    - u64::from(distance) * u64::from(distance))
                .isqrt() as u32;
            for offset in [row, height - row - 1] {
                rectangles.push(Self {
                    x: x + inset as i32,
                    y: y + offset as i32,
                    width: width - 2 * inset,
                    height: 1,
                    radius: 0,
                });
            }
        }
        rectangles
    }
}
