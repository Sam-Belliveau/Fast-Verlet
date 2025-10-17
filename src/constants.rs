use sfml::system::{Vector2, Vector3};

pub type Vec2 = Vector2<f64>;
pub type Vec3 = Vector3<f64>;

pub type Secs = f64;

pub const K_COLLISION_PRESSURE : f64 = 0.25;
pub const K_RADIUS: f64 = 6.0;
pub const K_FRICTION: f64 = 0.0025;

pub const WIDTH: u32 = 1600;
pub const HEIGHT: u32 = 1200;
