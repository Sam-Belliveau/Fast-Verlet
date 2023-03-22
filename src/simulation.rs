
use std::f64::consts::PI;
use std::sync::Arc;
use std::sync::Mutex;

use crate::aabb::*;
use crate::aabb_tree::*;
use crate::constants::*;
use crate::particle::*;

use crate::sfml::graphics::Drawable;

use rayon::ThreadPool;
use rayon::prelude::*;

use sfml::graphics::{
    Color, PrimitiveType, RenderStates, RenderWindow, Vertex, VertexBuffer, VertexBufferUsage,
};
use sfml::system::Vector2f;

fn color(t: &mut f64) -> Color {
    *t += 3.88322208;
    Color::rgb(
        (127.0 + 127.0 * (*t + 0.0 * PI / 3.0).sin()) as u8,
        (127.0 + 127.0 * (*t + 2.0 * PI / 3.0).sin()) as u8,
        (127.0 + 127.0 * (*t + 4.0 * PI / 3.0).sin()) as u8,
    )
}

#[derive(Debug)]
pub struct Simulator {
    pub size: usize,
    pub particles_tree: AABBTree,
    pub particles: Vec<Arc<Mutex<Particle>>>,
    pub bounds: BoundingBox,
    pub gravity: Vec2,

    thread_pool: ThreadPool
}


impl Simulator {

    fn default_particle() -> Arc<Mutex<Particle>> {
        Arc::new(Mutex::new(Particle::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            1.0,
            Color::WHITE,
        )))
    }

    pub fn new(size: Vec2) -> Simulator {
        Simulator {
            size: 1,
            particles_tree: AABBTree::new_leaf(Self::default_particle()),
            particles: vec![Self::default_particle()],
            bounds: BoundingBox::new(Vec2::new(0.0, 0.0), size, 400.0),
            gravity: Vec2::new(0.0, 1000.0),
            thread_pool: rayon::ThreadPoolBuilder::new().num_threads(16).build().unwrap()
        }
    }

    pub fn add(&mut self, particle: Particle) {
        self.size += 1;
        self.particles.push(Arc::new(Mutex::new(particle)));
        self.particles_tree = AABBTree::build_tree(&self.particles);
    }
}

impl Simulator {

    fn update_collisions(&mut self) {
        let mut stack = vec![&self.particles_tree];

        self.thread_pool.scope(|scope| {
            while let Some(p) = stack.pop() {
                match p {
                    AABBTree::Leaf(_) => (),
                    AABBTree::Internal { children, .. } => {
                        let (left, right) = &**children;

                        if left.overlap(right) {
                            scope.spawn(move |_| {
                                Self::check_collisions(left, right)
                            });
                        }

                        stack.push(left);
                        stack.push(right);
                    }
                }
            }
        });
    }

    fn check_collisions(left: &AABBTree, right: &AABBTree) {
        match (&left, &right) {
            (AABBTree::Leaf(p1), AABBTree::Leaf(p2)) => {
                p1.lock().unwrap().update_collision(&mut p2.lock().unwrap());
            }
            (AABBTree::Leaf(p), AABBTree::Internal { children, .. }) => {
                let (c1, c2) = &**children;
                if p.lock().unwrap().overlap(c1) {
                    Self::check_collisions(&left, &c1);
                }
                if p.lock().unwrap().overlap(c2) {
                    Self::check_collisions(&left, &c2);
                }
            }

            (AABBTree::Internal { children, .. }, AABBTree::Leaf(p)) => {
                let (c1, c2) = &**children;
                if p.lock().unwrap().overlap(c1) {
                    Self::check_collisions(&right, &c1);
                }
                if p.lock().unwrap().overlap(c2) {
                    Self::check_collisions(&right, &c2);
                }
            }

            (
                AABBTree::Internal { children: c1, .. },
                AABBTree::Internal { children: c2, .. },
            ) => {
                let (a1, a2) = &**c1;
                let (b1, b2) = &**c2;

                if a1.overlap(b1) {
                    Self::check_collisions(&a1, &b1);
                }
                if a1.overlap(b2) {
                    Self::check_collisions(&a1, &b2);
                }
                if a2.overlap(b1) {
                    Self::check_collisions(&a2, &b1);
                }
                if a2.overlap(b2) {
                    Self::check_collisions(&a2, &b2);
                }
            }
        }
    }

    pub fn step(&mut self, dt: Secs) {
        self.particles_tree.refit();
        self.update_collisions();

        self.particles
            .par_iter_mut()
            .for_each(|particle| {
                particle.lock().unwrap().update_verlet(dt, self.gravity);
                self.bounds.update_bounds(&mut particle.lock().unwrap());
            });
    }

    pub fn step_substeps(&mut self, dt: Secs, substeps: i32) {
        let substep_dt = dt / f64::from(substeps);

        for _ in 0..substeps {
            self.step(substep_dt);
        }

        self.particles_tree = AABBTree::build_tree(&self.particles);
    }

    pub fn draw(&mut self, window: &mut RenderWindow) {
        let mut vertex_buffer: Vec<Vertex> = Vec::new();

        let mut stack = vec![&self.particles_tree];

        let k = Vector2f::new(0.0, 0.0);
        let mut t = 0.0;
        while let Some(node) = stack.pop() {
            match node {
                AABBTree::Leaf(p_ref) => {
                    let p = p_ref.lock().unwrap().bound();
                    let p_color = color(&mut t);

                    vertex_buffer.push(Vertex::new(Vec2::new(p.min().x, p.min().y).as_other(), p_color, k));
                    vertex_buffer.push(Vertex::new(Vec2::new(p.min().x, p.max().y).as_other(), p_color, k));

                    vertex_buffer.push(Vertex::new(Vec2::new(p.min().x, p.max().y).as_other(), p_color, k));
                    vertex_buffer.push(Vertex::new(Vec2::new(p.max().x, p.max().y).as_other(), p_color, k));

                    vertex_buffer.push(Vertex::new(Vec2::new(p.max().x, p.max().y).as_other(), p_color, k));
                    vertex_buffer.push(Vertex::new(Vec2::new(p.max().x, p.min().y).as_other(), p_color, k));

                    vertex_buffer.push(Vertex::new(Vec2::new(p.max().x, p.min().y).as_other(), p_color, k));
                    vertex_buffer.push(Vertex::new(Vec2::new(p.min().x, p.min().y).as_other(), p_color, k));

                }
                AABBTree::Internal { children, .. } => {
                    let (left, right) = &**children;
                    let b = node.bound();
                    let b_color = color(&mut t);

                    vertex_buffer.push(Vertex::new(Vec2::new(b.min.x, b.min.y).as_other(), Color::RED, k));
                    vertex_buffer.push(Vertex::new(Vec2::new(b.min.x, b.max.y).as_other(), Color::RED, k));

                    vertex_buffer.push(Vertex::new(Vec2::new(b.min.x, b.max.y).as_other(), Color::CYAN, k));
                    vertex_buffer.push(Vertex::new(Vec2::new(b.max.x, b.max.y).as_other(), Color::CYAN, k));

                    vertex_buffer.push(Vertex::new(Vec2::new(b.max.x, b.max.y).as_other(), Color::MAGENTA, k));
                    vertex_buffer.push(Vertex::new(Vec2::new(b.max.x, b.min.y).as_other(), Color::MAGENTA, k));

                    vertex_buffer.push(Vertex::new(Vec2::new(b.max.x, b.min.y).as_other(), Color::GREEN, k));
                    vertex_buffer.push(Vertex::new(Vec2::new(b.min.x, b.min.y).as_other(), Color::GREEN, k));

                    stack.push(left);
                    stack.push(right);
                }
            }
        }
        // for particle in self.particles.iter_mut() {
        //     let p = particle;
        //     let s = 0.5_f64.sqrt();
        //     vertex_buffer.push(Vertex::new(
        //         (p.pos + Vec2::new(p.radius * s, p.radius * s)).as_other(),
        //         p.color,
        //         Vector2f::new(0.0, 0.0),
        //     ));
        //     vertex_buffer.push(Vertex::new(
        //         (p.pos + Vec2::new(p.radius * s, -p.radius * s)).as_other(),
        //         p.color,
        //         Vector2f::new(0.0, 0.0),
        //     ));
        //     vertex_buffer.push(Vertex::new(
        //         (p.pos + Vec2::new(-p.radius * s, -p.radius * s)).as_other(),
        //         p.color,
        //         Vector2f::new(0.0, 0.0),
        //     ));
        //     vertex_buffer.push(Vertex::new(
        //         (p.pos + Vec2::new(-p.radius * s, p.radius * s)).as_other(),
        //         p.color,
        //         Vector2f::new(0.0, 0.0),
        //     ));
        // }
        let mut buffer = VertexBuffer::new(
            PrimitiveType::LINES,
            vertex_buffer.len() as u32,
            VertexBufferUsage::STREAM,
        );

        buffer.update(&vertex_buffer, 0);

        buffer.draw(window, &RenderStates::DEFAULT);
    }

    pub fn clear(&mut self) {
        self.size = 1;
        self.particles = vec![Self::default_particle()];
        self.particles_tree = AABBTree::new_leaf(Self::default_particle());
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    min: Vec2,
    max: Vec2,
    r: f64,
}

fn bound(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

impl BoundingBox {
    pub fn new(a: Vec2, b: Vec2, r: f64) -> BoundingBox {
        BoundingBox {
            min: Vec2::new(a.x.min(b.x) + r, a.y.min(b.y) + r),
            max: Vec2::new(a.x.max(b.x) - r, a.y.max(b.y) - r),
            r: r,
        }
    }
}

impl BoundingBox {
    fn update_bounds(&self, other: &mut Particle) {
        let r = other.radius;
        let pos = Vec2::new(
            bound(other.pos.x, self.min.x + r, self.max.x - r),
            bound(other.pos.y, self.min.y + r, self.max.y - r),
        );

        let d = pos - other.pos;
        other.pos += d * (kCollisionPressure * (1.0 - self.r / d.length_sq().sqrt()).max(0.0));
    }
}
