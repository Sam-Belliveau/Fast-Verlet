extern crate sfml;

pub mod constants;

pub mod stopwatch;
pub mod particle;
pub mod simulation;
pub mod aabb;
pub mod aabb_tree;

use std::f64::consts::PI;

use constants::*;

use simulation::Simulator;
use particle::Particle;
use stopwatch::StopWatch;

use sfml::graphics::{RenderWindow, RenderTarget, Color};
use sfml::window::{Style, Event};

fn color(t: f64) -> Color {
    Color::rgb(
        (127.0 + 127.0 * (t + 0.0 * PI / 3.0).sin()) as u8,
        (127.0 + 127.0 * (t + 2.0 * PI / 3.0).sin()) as u8,
        (127.0 + 127.0 * (t + 4.0 * PI / 3.0).sin()) as u8,
    )
}

fn main() {
    let mut window = RenderWindow::new(
        (3600, 1200),
        "SFML Example",
        Style::CLOSE,
        &Default::default(),
    );

    // window.set_framerate_limit(20);

    let mut simulation = Simulator::new(Vec2::new(3600.0, 1200.0));
    let mut timer = StopWatch::new();
    let mut ctimer = StopWatch::new();
    let mut ptimer = StopWatch::new();
    let mut f = 0;
    let mut p = 0;
    let mut b = false;
    let mut mx = 0.0;
    let mut my = 0.0;
    let mut render = 0.02;
    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::MouseButtonPressed { button, x, y } => {
                    mx = x as f64;
                    my = y as f64;
                    b = true;
                    f = 1000;
                },
                Event::MouseButtonReleased { button, x, y } => b = false,
                Event::KeyPressed { code, alt, ctrl, shift, system } => {p = 0; simulation.clear();},
                _ => (),
            }
        }

        window.clear(sfml::graphics::Color::BLACK);


        timer.reset();
        simulation.step_substeps(0.01, 8);
        let physics = timer.reset();
        simulation.draw(&mut window);
        render = timer.reset();

        if b { f += 1; }
        if f > 1 {
            for x in -0..1 {
                for y in -10..10 {
                    p += 1;
                    simulation.add(
                        Particle::new(
                            Vec2::new(mx + 50.0 * (x as f64) + 0.0 * 20.0 * (((y as i32).abs() % 2) as f64), my + 10.0 * (y as f64)),
                            Vec2::new(1000.0, 500.0),
                            4.0,
                            color(ctimer.time())
                        )
                    );
                }
            }
            f = 0;
        }

        window.set_title(&format!("{} | {:6.4}", p, physics));

        window.display();
    }
}
