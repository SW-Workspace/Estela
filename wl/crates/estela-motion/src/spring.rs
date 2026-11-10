const STEPS_PER_SECOND: f64 = 960.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    pub position: f64,
    pub velocity: f64,
}
impl Spring {
    pub fn step(&mut self, target: f64, stiffness: f64, damping: f64) {
        let acceleration = stiffness * (target - self.position) - damping * self.velocity;
        self.velocity += acceleration / STEPS_PER_SECOND;
        self.position += self.velocity / STEPS_PER_SECOND;
    }
}
#[test]
fn deterministic_reference() {
    let mut spring = Spring {
        position: 0.0,
        velocity: 0.0,
    };
    spring.step(1.0, 960.0, 0.0);
    assert_eq!(spring.velocity.to_bits(), 1.0_f64.to_bits());
    assert_eq!(
        spring.position.to_bits(),
        (1.0_f64 / STEPS_PER_SECOND).to_bits()
    );
}
