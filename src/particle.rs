
use crate::constants::*;
use crate::aabb::*;
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

        self.pos = (vel + accel * dt) * dt + pos;
        self.prev_pos = pos;
        self.prev_dt = dt;
    }

    pub fn update_collision(&mut self, other: &mut Particle) {
        let delta = self.pos - other.pos;
        let space = self.radius + other.radius;

        if delta.y.abs() > self.radius + other.radius { return; }

        let distance = delta.length_sq();
        let ratio = space * space / distance;

        let push = delta * ((ratio.sqrt() - 1.0) * 0.5 * kCollisionPressure).max(0.0);
        self.pos += push;
        other.pos -= push;
    }
}

unsafe impl Sync for Particle {}
unsafe impl Send for Particle {}

impl Bounded for Particle {
    fn min(&self) -> Vec2 {
        self.pos - Vec2::new(self.radius, self.radius)
    }

    fn max(&self) -> Vec2 {
        self.pos + Vec2::new(self.radius, self.radius)
    }
}