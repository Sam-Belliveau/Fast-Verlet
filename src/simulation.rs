use crate::constants::*;
use crate::particle::*;

use crate::sfml::graphics::Drawable;

use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;

use sfml::graphics::{
    PrimitiveType, RenderStates, RenderWindow, Vertex, VertexBuffer, VertexBufferUsage,
};
use sfml::system::Vector2f;

const PARTITION_LENGTH: f64 = 24.0;

type Accelerator = Box<dyn Fn(&Particle) -> Vec2 + Send + Sync + 'static>;
type ParticleSet = Vec<Box<Particle>>;

pub struct Simulator {
    partitions: Vec<Vec<ParticleSet>>,
    partition_shape: (usize, usize),

    bounds: BoundingBox,
    accelerator: Accelerator,

    thread_pool: ThreadPool
}

impl Simulator {
    fn default_partition(shape: (usize, usize)) -> Vec<Vec<ParticleSet>> {
        vec![vec![Vec::with_capacity(16); shape.1]; shape.0]
    }

    pub fn new(size: Vec2, accelerator: Accelerator) -> Simulator {
        let shape = (
            (size.x / PARTITION_LENGTH) as usize,
            (size.y / PARTITION_LENGTH) as usize,
        );

        Simulator {
            partitions: Self::default_partition(shape),
            partition_shape: shape,

            bounds: BoundingBox::new(Vec2::new(0.0, 0.0), size, 1000.0),
            accelerator: accelerator,

            thread_pool: ThreadPoolBuilder::new().num_threads(12).build().unwrap()
        }
    }

    pub fn add(&mut self, particle: Box<Particle>) {
        let (x, y) = self.bounds.get_partition(&particle, self.partition_shape);
        self.partitions[x][y].push(particle);
    }
}

impl Simulator {
    fn update_partitions(&mut self) {
        let mut need_move = Vec::with_capacity(self.partition_shape.0 * self.partition_shape.1);

        for (cx, column) in self.partitions.iter_mut().enumerate() {
            for (cy, row) in column.iter_mut().enumerate() {
                need_move.extend(row.drain_filter(|particle| {
                    let (x, y) = self.bounds.get_partition(&particle, self.partition_shape);
                    x != cx || y != cy
                }));
            }
        }

        for particle in need_move {
            self.add(particle);
        }
    }

    fn update_set_collisions(a_set: &mut ParticleSet) {
        for i in 1..a_set.len() {
            let (left, right) = a_set.split_at_mut(i);
            if let Some(particle) = right.first_mut() {
                for other in left.iter_mut() {
                    particle.update_collision(other);
                }
            }
        }
    }

    fn update_set_to_set_collisions(a_set: &mut ParticleSet, b_set_opt: Option<&mut ParticleSet>) {
        if let Some(b_set) = b_set_opt {
            if b_set.is_empty() {
                return;
            }

            for a in a_set.iter_mut() {
                for b in b_set.iter_mut() {
                    a.update_collision(b);
                }
            }
        }
    }

    fn update_columns_collisions(
        left: &mut [ParticleSet],
        right: &mut [ParticleSet],
    ) {
        Self::update_column_collisions(left);

        for (i, set) in left.iter_mut().enumerate() {
            Self::update_set_to_set_collisions(set, right.get_mut(i - 1));
        }

        for (i, set) in left.iter_mut().enumerate() {
            Self::update_set_to_set_collisions(set, right.get_mut(i + 0));
        }

        for (i, set) in left.iter_mut().enumerate() {
            Self::update_set_to_set_collisions(set, right.get_mut(i + 1));
        }
    }

    fn update_column_collisions(
        col: &mut [ParticleSet]
    ) {
        for pair in col[0..].chunks_exact_mut(2) {
            if let [a, b] = pair {
                Self::update_set_to_set_collisions(a, Some(b));
            }
        }

        for pair in col[1..].chunks_exact_mut(2) {
            if let [a, b] = pair {
                Self::update_set_to_set_collisions(a, Some(b));
            }
        }
    }

    fn update_column(
        dt: Secs,
        accelerator: &Accelerator,
        bounds: &BoundingBox,
        column: &mut [ParticleSet],
    ) {
        for set in column.iter_mut() {
            for particle in set.iter_mut() {
                particle.update_verlet(dt, accelerator(&particle));
                bounds.update_bounds(particle);
            }

            Self::update_set_collisions(set);
        }
    }

    fn update_physics<'a>(&mut self, dt: Secs) {
        let chunk_updater = |columns: &mut [Vec<ParticleSet>]| {
            match columns {
                [left, right] => {
                    Self::update_column(dt, &self.accelerator, &self.bounds, left);
                    Self::update_columns_collisions(left, right);
                },

                [col] => {
                    Self::update_column(dt, &self.accelerator, &self.bounds, col);
                    Self::update_column_collisions(col);
                },

                _ => panic!("columns are deformed size!"),
            }
        };

        self.thread_pool.install(|| {
            self.partitions[0..].par_chunks_mut(2).for_each(chunk_updater);
            self.partitions[1..].par_chunks_mut(2).for_each(chunk_updater);
        });
    }

    pub fn step(&mut self, dt: Secs) {
        self.update_partitions();
        self.update_physics(dt);
    }

    pub fn step_substeps(&mut self, dt: Secs, substeps: i32) {
        let substep_dt = dt / f64::from(substeps);

        for _ in 0..substeps {
            self.step(substep_dt);
        }
    }

    pub fn draw(&mut self, window: &mut RenderWindow) {
        let mut vertex_buffer: Vec<Vertex> = Vec::new();

        let k = Vector2f::new(0.0, 0.0);
        for column in self.partitions.iter() {
            for row in column.iter() {
                for p in row.iter() {
                    let r = p.radius * 0.5_f64.sqrt();
                    let c = p.color;

                    vertex_buffer.extend_from_slice(&[
                        Vertex::new((p.pos + Vec2::new(0.0 + r, 0.0 + r)).as_other(), c, k),
                        Vertex::new((p.pos + Vec2::new(0.0 + r, 0.0 - r)).as_other(), c, k),
                        Vertex::new((p.pos + Vec2::new(0.0 - r, 0.0 - r)).as_other(), c, k),
                        Vertex::new((p.pos + Vec2::new(0.0 - r, 0.0 + r)).as_other(), c, k),
                    ]);
                }
            }
        }

        let mut buffer = VertexBuffer::new(
            PrimitiveType::QUADS,
            vertex_buffer.len() as u32,
            VertexBufferUsage::STREAM,
        );

        buffer.update(&vertex_buffer, 0);

        buffer.draw(window, &RenderStates::DEFAULT);
    }

    pub fn clear(&mut self) -> &mut Self {
        self.partitions = Self::default_partition(self.partition_shape);
        self
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    min: Vec2,
    max: Vec2,
    area: Vec2,
    r: f64,
}

impl BoundingBox {
    pub fn new(a: Vec2, b: Vec2, r: f64) -> BoundingBox {
        let min = Vec2::new(a.x.min(b.x), a.y.min(b.y));
        let max = Vec2::new(a.x.max(b.x), a.y.max(b.y));
        BoundingBox {
            min: min,
            max: max,
            area: max - min,
            r: r,
        }
    }

    fn update_bounds(&self, other: &mut Particle) {
        let r = self.r + other.radius;
        let pos = Vec2::new(
            other.pos.x.clamp(self.min.x + r, self.max.x - r),
            other.pos.y.clamp(self.min.y + r, self.max.y - r),
        );

        let d = pos - other.pos;
        other.pos += d * (K_COLLISION_PRESSURE * (1.0 - self.r / d.length_sq().sqrt()).max(0.0));
    }

    fn get_partition(&self, other: &Particle, shape: (usize, usize)) -> (usize, usize) {
        let partition_area = Vec2::new(shape.0 as f64, shape.1 as f64);
        let scaler = partition_area.cwise_div(self.area);

        let norm_pos = (other.pos - self.min).cwise_mul(scaler);

        (
            (norm_pos.x as isize).clamp(0, shape.0 as isize - 1) as usize,
            (norm_pos.y as isize).clamp(0, shape.1 as isize - 1) as usize,
        )
    }
}
