
use crate::constants::*;
use sfml::graphics::Color;

#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: Vec2,
    prev_pos: Vec2,

    pub radius: f64,
    pub color: Color,
    prev_dt: f64,
}

impl Particle {
    pub fn new(pos: Vec2, vel: Vec2, radius: f64, color: Color) -> Particle {
        Particle {
            pos: pos,
            prev_pos: pos - vel,
            radius: radius.abs(),
            color: color,
            prev_dt: 1.0
        }
    }

    pub fn update_verlet(&mut self, dt: Secs, accel: Vec2) {
        let pos = self.pos;
        let vel = (self.pos - self.prev_pos) / self.prev_dt;
        let accel = accel - vel * 0.0;

        self.pos = pos + vel * (dt) + accel * (dt * dt);
        self.prev_pos = pos;
        self.prev_dt = dt;
    }

    pub fn update_collision(&mut self, other: &mut Particle) {
        let delta = self.pos - other.pos;
        let space = self.radius + other.radius;

        let distance = delta.length_sq().sqrt().max(1.0);

        if distance < space {
            let push = delta * (((space / distance) - 1.0) * 0.5 * K_COLLISION_PRESSURE).max(0.0);
            self.pos += push;
            other.pos -= push;
        }
    }
}
