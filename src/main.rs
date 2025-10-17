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
        (WIDTH, HEIGHT),
        "SFML Example",
        Style::DEFAULT,
        &Default::default(),
    )
    .expect("Cannot create a new Render Window");

    const FPS: u32 = 120;

    window.set_framerate_limit(FPS);

    let mut simulation = Simulator::new(
        Vec2::new(WIDTH as f64, HEIGHT as f64),
        Box::new(|_| {
            Vec2::new(0.0, 0.05)
        }),
    );

    let mut particles: i32 = 0;
    let mut pressed = false;

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed { .. } => {
                    particles = 0;
                    simulation.clear();
                }
                _ => (),
            }
        }

        window.clear(sfml::graphics::Color::BLACK);

        simulation.step();
        simulation.draw(&mut window);

        if !pressed && mouse::Button::Left.is_pressed() {
            pressed = true;
            let m: Vec2 = window.mouse_position().as_other();
            for x in -10..=10 {
                for y in -10..=10 {
                    particles += 1;
                    let r = K_RADIUS;

                    simulation.add(Box::new(Particle::new(
                        m + Vec2::new(x as f64, -y as f64) * (2.0 * r),
                        Vec2::new(0.0, 0.0) * r,
                        r,
                        color(&mut (0.001 * (particles as f64))),
                    )));
                }
            }
        } else {
            pressed = mouse::Button::Left.is_pressed();
        }

        let simulation_dt = simulation.total_dt();
        let simulation_fps = 1.0 / simulation_dt;

        window.set_title(&format!("{} | Max FPS: {:5.1}", particles, simulation_fps));

        window.display();
    }
}
