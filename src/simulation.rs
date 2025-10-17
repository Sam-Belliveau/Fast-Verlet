use crate::constants::*;
use crate::particle::*;

use crate::sfml::graphics::Drawable;
use crate::stopwatch::StopWatch;

use rayon::prelude::*;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;

use sfml::graphics::{
    PrimitiveType, RenderStates, RenderWindow, Vertex, VertexBuffer, VertexBufferUsage,
};
use sfml::system::Vector2f;

const PARTITION_LENGTH: f64 = 32.0;

type Accelerator = Box<dyn Fn(&Particle) -> Vec2 + Send + Sync + 'static>;
type ParticleSet = Vec<Box<Particle>>;

pub struct Simulator {
    partitions: Vec<Vec<ParticleSet>>,
    partition_shape: (usize, usize),

    bounds: BoundingBox,
    accelerator: Accelerator,

    thread_pool: ThreadPool,

    step_size: Secs,

    partition_dt_avg: Secs,
    physics_dt_avg: Secs,
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
            accelerator,

            thread_pool: ThreadPoolBuilder::new().num_threads(16).build().unwrap(),
            
            step_size: 1.0 / 120.0,

            partition_dt_avg: 0.0,
            physics_dt_avg: 0.0,
        }
    }

    pub fn add(&mut self, particle: Box<Particle>) {
        let (x, y) = self.bounds.get_partition(&particle, self.partition_shape);
        self.partitions[x][y].push(particle);
    }
}

impl Simulator {
    fn update_partitions(&mut self) {
        let mut col_updated = Vec::new();

        for (cx, column) in self.partitions.iter_mut().enumerate() {
            for (cy, row) in column.iter_mut().enumerate() {
                col_updated.extend(row.extract_if(.., |particle| {
                    let (x, y) = self.bounds.get_partition(particle, self.partition_shape);
                    x != cx || y != cy
                }))
            }
        }

        for particle in col_updated {
            self.add(particle);
        }
    }

    fn collide_set(dt: Secs, set: &mut ParticleSet) {
        for i in 1..set.len() {
            let (left, right) = set.split_at_mut(i);
            if let Some(particle) = right.first_mut() {
                for other in left.iter_mut() {
                    particle.update_collision(dt, other);
                }
            }
        }
    }

    fn collide_set_with_set(step_size: Secs, a_set: &mut ParticleSet, b_set: &mut ParticleSet) {
        if b_set.is_empty() {
            return;
        }

        for a in a_set.iter_mut() {
            for b in b_set.iter_mut() {
                a.update_collision(step_size, b);
            }
        }
    }

    fn collide_column(step_size: Secs, col: &mut [ParticleSet]) {
        for set in col.iter_mut() {
            Self::collide_set(step_size, set);
        }

        for pair in col[0..].chunks_exact_mut(2) {
            if let [a, b] = pair {
                Self::collide_set_with_set(step_size, a, b);
            }
        }

        for pair in col[1..].chunks_exact_mut(2) {
            if let [a, b] = pair {
                Self::collide_set_with_set(step_size, a, b);
            }
        }
    }

    fn collide_column_with_column(
        step_size: Secs,
        left: &mut [ParticleSet],
        right: &mut [ParticleSet],
    ) {
        for (left_set, right_set) in left[1..].iter_mut().zip(right[0..].iter_mut()) {
            Self::collide_set_with_set(step_size, left_set, right_set);
        }

        for (left_set, right_set) in left[0..].iter_mut().zip(right[0..].iter_mut()) {
            Self::collide_set_with_set(step_size, left_set, right_set);
        }

        for (left_set, right_set) in left[0..].iter_mut().zip(right[1..].iter_mut()) {
            Self::collide_set_with_set(step_size, left_set, right_set);
        }
    }

    fn physics_step_column(
        step_size: Secs,
        accelerator: &Accelerator,
        bounds: &BoundingBox,
        column: &mut [ParticleSet],
    ) {
        for set in column.iter_mut() {
            for particle in set.iter_mut() {
                particle.update_verlet(step_size, accelerator(particle));
                bounds.update_bounds(particle);
            }
        }
    }

    fn update_physics(&mut self) {
        let step_size = self.step_size;
        let chunk_updater = |columns: &mut [Vec<ParticleSet>]| {
            for col in columns.iter_mut() {
                Self::physics_step_column(step_size, &self.accelerator, &self.bounds, col);
                Self::collide_column(step_size, col);
            }

            if let [left, right] = columns {
                Self::collide_column_with_column(step_size, left, right);
            }
        };

        self.thread_pool.install(|| {
            self.partitions[0..]
                .par_chunks_mut(2)
                .for_each(chunk_updater);
            self.partitions[1..]
                .par_chunks_mut(2)
                .for_each(chunk_updater);
        });
    }

    pub fn step(&mut self) {
        let mut timer = StopWatch::new();
        self.update_partitions();
        let partition = timer.reset();
        self.update_physics();
        let physics = timer.reset();

        let average_rc = 1.0;
        let dt = physics + partition;
        let alpha = -f64::exp_m1(-dt / average_rc);

        self.partition_dt_avg += alpha * (partition - self.partition_dt_avg);
        self.physics_dt_avg += alpha * (physics - self.physics_dt_avg);

        println!("+------------------+");
        println!("|Partition: {:7.5}|", self.partition_dt_avg);
        println!("|Physics:   {:7.5}|", self.physics_dt_avg);
        println!("+------------------+\n");
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
            vertex_buffer.len(),
            VertexBufferUsage::STREAM,
        )
        .expect("Could not allocate vertex buffer");

        buffer
            .update(&vertex_buffer, 0)
            .expect("SFML Update Failed");
        buffer.draw(window, &RenderStates::DEFAULT);
    }

    pub fn clear(&mut self) -> &mut Self {
        self.partitions = Self::default_partition(self.partition_shape);
        self
    }

    pub fn total_dt(&self) -> Secs {
        self.partition_dt_avg + self.physics_dt_avg
    }
}

#[derive(Debug, Clone, Copy)]
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
            min,
            max,
            area: max - min,
            r,
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
