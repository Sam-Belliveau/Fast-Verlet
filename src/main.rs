#![feature(drain_filter)]

extern crate sfml;

pub mod constants;

pub mod stopwatch;
pub mod particle;

pub mod simulation;

use std::f64::consts::PI;

use constants::*;

use simulation::Simulator;
use particle::Particle;
use stopwatch::StopWatch;

use sfml::graphics::{RenderWindow, RenderTarget, Color};
use sfml::window::{Style, Event, mouse};

fn color(t: &mut f64) -> Color {
    *t += 3.88322208;
    Color::rgb(
        (127.0 + 127.0 * (*t + 0.0 * PI / 3.0).sin()) as u8,
        (127.0 + 127.0 * (*t + 2.0 * PI / 3.0).sin()) as u8,
        (127.0 + 127.0 * (*t + 4.0 * PI / 3.0).sin()) as u8,
    )
}

fn main() {
    let mut window = RenderWindow::new(
        (3600, 2400),
        "SFML Example",
        Style::CLOSE,
        &Default::default(),
    );

    const FPS: u32 = 120;

    window.set_framerate_limit(FPS);

    let mut simulation = Simulator::new(Vec2::new(3600.0, 2400.0), Box::new(|_|{
        // let offset = particle.pos - Vec2::new(1800.0, 1200.0);

        // offset / offset.length_sq().max(10.0) * -10000000.0
        Vec2::new(0.0, 500.0)
    }));

    let mut timer = StopWatch::default();
    let mut fc = 0.0;
    let mut p = 0;
    let mut physics_avg = 0.00;
    let mut b = false;
    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed { .. } => {fc = 0.0; p = 0; simulation.clear();},
                _ => (),
            }
        }

        window.clear(sfml::graphics::Color::BLACK);


        timer.reset();
        simulation.step_substeps(1.0 / (FPS as f64), 8);
        let physics = timer.reset();
        physics_avg += (physics - physics_avg) * 0.5;
        simulation.draw(&mut window);
        timer.reset();

        if !b && mouse::Button::Left.is_pressed() {
            b = true;
            let m : Vec2 = window.mouse_position().as_other();
            for x in -10..=10 {
                for y in -10..=10 {
                    fc += 0.001;
                    p += 1;
                    let r = 6.0;
                    simulation.add(Box::new(
                        Particle::new(
                            m + Vec2::new(x as f64, -y as f64) * (2.0 * r),
                            Vec2::new(100.0, 0.0) * r,
                            r,
                            color(&mut (1.0 * fc))
                        )
                    )
                    );
                }
            }
        } else {
            b = mouse::Button::Left.is_pressed();
        }

        window.set_title(&format!("{} | {:5.1}", p, 1.0 / physics_avg));

        window.display();
    }
}
