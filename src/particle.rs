use crate::constants::*;
use sfml::graphics::Color;

#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: Vec2,
    prev_pos: Vec2,

    pub radius: f64,
    pub color: Color,
}

impl Particle {
    pub fn new(pos: Vec2, vel: Vec2, radius: f64, color: Color) -> Particle {
        Particle {
            pos,
            prev_pos: pos - vel,
            radius: radius.abs(),
            color
        }
    }

    pub fn update_verlet(&mut self, accel: Vec2) {
        let new_pos = self.pos * 2.0 - self.prev_pos + accel;
        self.prev_pos = self.pos;
        self.pos = new_pos;
    }

    pub fn update_collision(&mut self, other: &mut Particle) {
        let difference = self.pos - other.pos;
        let min_distance = self.radius + other.radius;

        let distance_sq = difference.length_sq();
        let min_distanec_sq = min_distance * min_distance;

        if distance_sq <= min_distanec_sq {
            let distance = distance_sq.sqrt();

            let delta = min_distance - distance;
            let correction = difference * (0.5 * delta / distance) * K_COLLISION_PRESSURE;

            self.pos += correction;
            other.pos -= correction;
        }
    }
}
