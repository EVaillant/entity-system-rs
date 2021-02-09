use cgmath::{prelude::*, Matrix3, Rad, Vector2, Vector3};

extern crate sdl2;

use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use std::time::Duration;

struct Position {
    position: Vector2<f32>,
    angle: Rad<f32>,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            angle: Rad::zero(),
        }
    }
}

impl Position {
    pub fn get_matrix(&self) -> Matrix3<f32> {
        let tse_matrix = Matrix3::from_translation(self.position);
        let rot_matrix = Matrix3::from_angle_z(self.angle);
        tse_matrix * rot_matrix
    }
}

impl entity_system::Composant for Position {
    type Storage = entity_system::BasicVecStorage<Self>;
}

struct Velocity {
    position: Vector2<f32>,
    angle: Rad<f32>,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            angle: Rad::zero(),
        }
    }
}

impl entity_system::Composant for Velocity {
    type Storage = entity_system::BasicVecStorage<Self>;
}

enum Shape {
    Circle,
    Square,
    Triangle,
}

impl Default for Shape {
    fn default() -> Self {
        Self::Circle
    }
}

impl entity_system::Composant for Shape {
    type Storage = entity_system::BasicVecStorage<Self>;
}

entity_system::create_entity_manager_composant!(EMC {
    Position,
    Velocity,
    Shape
});
type EntityManager = entity_system::EntityManager<EMC>;
type Query = entity_system::Query<EMC>;

fn main() -> Result<(), String> {
    let mut entity_manager = EntityManager::new();
    for i in 0..10 {
        let entity = entity_manager.create_entity();

        entity_manager.add_composant_with::<Position, _>(entity, |position| {
            position.position.x = rand::random::<f32>() * 800.0;
            position.position.y = rand::random::<f32>() * 600.0;
            position.angle = Rad(rand::random::<f32>() * std::f32::consts::PI * 2.0);
        });

        entity_manager.add_composant_with::<Shape, _>(entity, |shape| {
            if i % 2 == 0 {
                *shape = Shape::Circle;
            } else {
                *shape = Shape::Square;
            }
        });
    }

    {
        let entity = entity_manager.create_entity();

        entity_manager.add_composant_with::<Position, _>(entity, |position| {
            position.position.x = 400.0;
            position.position.y = 300.0;
            position.angle = Rad::zero();
        });

        entity_manager.add_composant_with::<Shape, _>(entity, |shape| {
            *shape = Shape::Triangle;
        });

        entity_manager.add_composant_with::<Velocity, _>(entity, |velocity| {
            velocity.position.x = 4.0;
            velocity.position.y = 3.0;
            velocity.angle = Rad(1.0);
        });
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("asteroids demo", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut query_drawable = Query::new();
    query_drawable
        .check_composant::<Shape>()
        .check_composant::<Position>();

    let mut query_velocity = Query::new();
    query_velocity
        .check_composant::<Velocity>()
        .check_composant::<Position>();

    'mainloop: loop {
        if let Some(event) = event_pump.poll_event() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }

        for entity in entity_manager.iter(&query_velocity) {
            let velocity = entity_manager.get_composant::<Velocity>(entity);
            entity_manager.update_composant_with::<Position, _>(entity, |position| {
                position.position.x += velocity.position.x;
                position.position.y += velocity.position.y;
                position.angle += velocity.angle;
            });
        }

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        for entity in entity_manager.iter(&query_drawable) {
            let position = entity_manager.get_composant::<Position>(entity);
            let shape = entity_manager.get_composant::<Shape>(entity);
            match *shape {
                Shape::Circle => {
                    canvas.circle(
                        position.position.x as i16,
                        position.position.y as i16,
                        10,
                        Color::WHITE,
                    )?;
                }
                Shape::Square => {
                    let matrix = position.get_matrix();
                    let p1 = matrix * Vector3::new(0.0, 0.0, 1.0);
                    let p2 = matrix * Vector3::new(0.0, 20.0, 1.0);
                    let p3 = matrix * Vector3::new(20.0, 20.0, 1.0);
                    let p4 = matrix * Vector3::new(20.0, 0.0, 1.0);
                    canvas.polygon(
                        &[p1.x as i16, p2.x as i16, p3.x as i16, p4.x as i16],
                        &[p1.y as i16, p2.y as i16, p3.y as i16, p4.y as i16],
                        Color::WHITE,
                    )?;
                }
                Shape::Triangle => {
                    let matrix = position.get_matrix();
                    let p1 = matrix * Vector3::new(0.0, 0.0, 1.0);
                    let p2 = matrix * Vector3::new(10.0, 20.0, 1.0);
                    let p3 = matrix * Vector3::new(20.0, 0.0, 1.0);
                    canvas.polygon(
                        &[p1.x as i16, p2.x as i16, p3.x as i16],
                        &[p1.y as i16, p2.y as i16, p3.y as i16],
                        Color::RED,
                    )?;
                }
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
