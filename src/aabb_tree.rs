
use std::borrow::Borrow;
use std::sync::Arc;
use std::sync::Mutex;

use rayon::slice::ParallelSliceMut;

use crate::aabb::*;
use crate::particle::*;
use crate::constants::*;

// Add this to your existing file
#[derive(Debug, Clone)]
pub enum AABBTree {
    Leaf(Arc<Mutex<Particle>>),
    Internal {
        aabb: AABB,
        children: Box<(AABBTree, AABBTree)>,
    },
}

enum VecAxis {
    X(), Y()
}

impl VecAxis {
    fn max_axis(v: &Vec2) -> VecAxis {
        if v.x > v.y { VecAxis::X() }
        else { VecAxis::Y() }
    }

    fn min_axis(v: &Vec2) -> VecAxis {
        if v.x < v.y { VecAxis::X() }
        else { VecAxis::Y() }
    }

    fn get(&self, v: &Vec2) -> f64 {
        match self {
            X => v.x,
            Y => v.y,
        }
    }
}

impl AABBTree {

    pub fn new_leaf(particle: Arc<Mutex<Particle>>) -> AABBTree {
        Self::Leaf(particle)
    }

    pub fn build_tree(particles: &[Arc<Mutex<Particle>>]) -> Self {
        match particles.len() {
            0 => panic!("Cannot build an AABB tree with no particles."),
            1 => AABBTree::Leaf(particles[0].clone()),
            _ => {
                let i = particles[0].lock().unwrap().bound();
                let aabb = particles.iter().fold(i, |acc, p| {
                    acc.join(&*p.lock().unwrap())
                });

                let axis = aabb.max - aabb.min;
                let split_axis = VecAxis::max_axis(&axis);

                let mut sorted_particles = particles.to_vec();
                sorted_particles.par_sort_unstable_by(|a, b| {
                    let a_pos = a.lock().unwrap().pos.clone();
                    let b_pos = b.lock().unwrap().pos.clone();
                    split_axis.get(&a_pos).partial_cmp(&split_axis.get(&b_pos)).unwrap_or(std::cmp::Ordering::Equal)
                });

                let mid = sorted_particles.len() / 2;
                let (left, right) = sorted_particles.split_at(mid);

                AABBTree::Internal {
                    aabb,
                    children: Box::new((AABBTree::build_tree(left), AABBTree::build_tree(right))),
                }
            }
        }
    }

    pub fn refit(&mut self) {
        match self {
            Self::Leaf(_) => (),
            Self::Internal { ref mut aabb, ref mut children } => {
                children.0.refit();
                children.1.refit();
                *aabb = children.0.join(&children.1);
            }
        }
    }

    pub fn collect_particles(&self) -> Vec<Particle> {
        let mut particles = Vec::new();
        self.collect_particles_recursive(&mut particles);
        particles
    }

    fn collect_particles_recursive(&self, particles: &mut Vec<Particle>) {
        match self {
            AABBTree::Leaf(particle) => particles.push(particle.lock().unwrap().clone()),
            AABBTree::Internal { children, .. } => {
                children.0.collect_particles_recursive(particles);
                children.1.collect_particles_recursive(particles);
            }
        }
    }
}

impl Bounded for AABBTree {

    fn min(&self) -> Vec2 {
        match self {
            Self::Leaf(p) => p.lock().unwrap().min(),
            Self::Internal { aabb, .. } => aabb.min()
        }
    }

    fn max(&self) -> Vec2 {
        match self {
            Self::Leaf(p) => p.lock().unwrap().max(),
            Self::Internal { aabb, .. } => aabb.max()
        }
    }
}

pub struct AABBNodeIter<'a> {
    stack: Vec<&'a mut AABBTree>,
}

impl AABBTree {
    pub fn iter_mut(&mut self) -> AABBNodeIter {
        AABBNodeIter {
            stack: vec![self],
        }
    }
}

impl<'a> Iterator for AABBNodeIter<'a> {
    type Item = Arc<Mutex<Particle>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            match node {
                AABBTree::Leaf(particle) => return Some(particle.clone()),
                AABBTree::Internal { children, .. } => {
                    self.stack.push(&mut children.0);
                    self.stack.push(&mut children.1);
                }
            }
        }
        None
    }
}
