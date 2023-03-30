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
            pos,
            prev_pos: pos - vel,
            radius: radius.abs(),
            color,
            prev_dt: 1.0,
        }
    }

    pub fn update_verlet(&mut self, dt: Secs, accel: Vec2) {
        let pos = self.pos;
        let vel = (self.pos - self.prev_pos) / self.prev_dt;
        let accel = accel - vel * K_FRICTION;

        self.pos = pos + vel * (dt) + accel * (dt * dt);
        self.prev_pos = pos;
        self.prev_dt = dt;
    }

    fn force(d: f64, dt: Secs) -> f64 {
        if 0.0 <= d {
            1.0
        } else {
            let k = dt * dt * K_STICKINESS * K_STICKINESS;
            k / (k + d * d)
        }
    }

    pub fn update_collision(&mut self, dt: Secs, other: &mut Particle) {
        let delta = self.pos - other.pos;
        let mut space = self.radius + other.radius;

        let distance = delta.length_sq().sqrt().max(1.0);

        // if the particle was previously on the other side,
        // the collision should treat as if we were colliding from that side.
        if delta.dot(self.prev_pos - other.prev_pos) < 0.0 {
            space = -space;
        }

        let d = space - distance;
        let force = Self::force(d, dt);

        if K_MIN_FORCE <= force {
            let push = delta * (d * force * 0.5 * K_COLLISION_PRESSURE / distance);
            self.pos += push;
            other.pos -= push;
        }

    }
}
