extern crate sfml;

pub mod constants;

pub mod particle;
pub mod stopwatch;

pub mod simulation;

use std::f64::consts::PI;

use constants::*;

use particle::Particle;
use simulation::Simulator;

use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::window::{mouse, Event, Style};

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
        (800, 600),
        "SFML Example",
        Style::DEFAULT,
        &Default::default(),
    )
    .expect("Cannot create a new Render Window");

    const FPS: u32 = 120;

    window.set_framerate_limit(FPS);

    let mut simulation = Simulator::new(
        Vec2::new(window.size().x as f64, window.size().y as f64),
        Box::new(|_| {
            // let offset = particle.pos - Vec2::new(1800.0, 1200.0);
            // offset / offset.length_sq().max(10.0) * -10000000.0

            Vec2::new(0.0, 500.0)
        }),
    );

    let mut fc = 0.0;
    let mut p = 0;
    let mut b = false;
    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed { .. } => {
                    fc = 0.0;
                    p = 0;
                    simulation.clear();
                }
                _ => (),
            }
        }

        window.clear(sfml::graphics::Color::BLACK);

        simulation.step();
        simulation.draw(&mut window);

        if !b && mouse::Button::Left.is_pressed() {
            b = true;
            let m: Vec2 = window.mouse_position().as_other();
            for x in -10..=10 {
                for y in -10..=10 {
                    fc += 0.001;
                    p += 1;
                    let r = 6.0;
                    simulation.add(Box::new(Particle::new(
                        m + Vec2::new(x as f64, -y as f64) * (2.0 * r),
                        Vec2::new(100.0, 0.0) * r,
                        r,
                        color(&mut (1.0 * fc)),
                    )));
                }
            }
        } else {
            b = mouse::Button::Left.is_pressed();
        }

        window.set_title(&format!("{} | {:5.1}", p, 1.0 / simulation.total_dt()));

        window.display();
    }
}
