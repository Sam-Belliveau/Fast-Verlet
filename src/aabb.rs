use crate::constants::*;

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    fn min_vec(a: Vec2, b: Vec2) -> Vec2 {
        Vec2::new(a.x.min(b.x), a.y.min(b.y))
    }

    fn max_vec(a: Vec2, b: Vec2) -> Vec2 {
        Vec2::new(a.x.max(b.x), a.y.max(b.y))
    }

    pub fn new(min: Vec2, max: Vec2) -> Self {
        AABB {
            min: Self::min_vec(min, max),
            max: Self::max_vec(min, max),
        }
    }
}

impl Bounded for AABB {
    fn min(&self) -> Vec2 {
        self.min
    }
    fn max(&self) -> Vec2 {
        self.max
    }
}

pub trait Bounded {
    fn min(&self) -> Vec2;
    fn max(&self) -> Vec2;

    fn join<T: Bounded>(&self, other: &T) -> AABB {
        AABB::new(
            AABB::min_vec(self.min(), other.min()),
            AABB::max_vec(self.max(), other.max()),
        )
    }

    fn expand(&self, amount: f64) -> AABB {
        let amount_vec = Vec2::new(amount, amount);
        AABB::new(self.min() - amount_vec, self.max() + amount_vec)
    }

    fn overlap<T: Bounded>(&self, other: &T) -> bool {
        let s_min = self.min();
        let s_max = self.max();
        let o_min = other.min();
        let o_max = other.max();

        !((s_max.x <= o_min.x) || (o_max.x <= s_min.x) || (s_max.y <= o_min.y) || (o_max.y <= s_min.y))
    }

    fn surface_area(&self) -> f64 {
        2.0 * (self.max() - self.min()).length_sq()
    }

    fn includes(&self, t: Vec2) -> bool {
        let s_min = self.min();
        let s_max = self.max();
        (s_min.x <= t.x && t.x <= s_max.x) && (s_min.y <= t.y && t.y <= s_max.y)
    }

    fn bound(&self) -> AABB {
        AABB::new(self.min(), self.max())
    }
}
