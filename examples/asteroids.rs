use cgmath::{prelude::*, Deg, Matrix3, Vector2, Vector3};

extern crate sdl2;

use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use std::time::Duration;

struct Position {
    position: Vector2<f32>,
    angle: Deg<f32>,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            angle: Deg::zero(),
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
    angle: Deg<f32>,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            angle: Deg::zero(),
        }
    }
}

impl entity_system::Composant for Velocity {
    type Storage = entity_system::BasicVecStorage<Self>;
}

#[derive(PartialEq)]
enum Shape {
    Circle,
    Square,
    Triangle,
    Bullet,
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
type Entity = entity_system::Entity;

fn update_velocity_angle(
    entity_manager: &mut EntityManager,
    entity: Entity,
    delta_angle: Deg<f32>,
) {
    entity_manager.update_composant_with::<Velocity, _>(entity, |velocity| {
        velocity.angle += delta_angle;
    });
}

fn update_velocity_position(
    entity_manager: &mut EntityManager,
    entity: Entity,
    delta_position: Vector2<f32>,
) {
    let position = entity_manager.get_composant::<Position>(entity);
    let delta = (Matrix3::from_angle_z(position.angle) * delta_position.extend(1.0)).truncate();
    entity_manager.update_composant_with::<Velocity, _>(entity, |velocity| {
        velocity.position += delta;
        velocity.position.x = velocity.position.x.min(5.0).max(-5.0);
        velocity.position.y = velocity.position.y.min(5.0).max(-5.0);
    });
}

fn main() -> Result<(), String> {
    let mut entity_manager = EntityManager::new();
    for i in 0..20 {
        let entity = entity_manager.create_entity();

        entity_manager.add_composant_with::<Position, _>(entity, |position| {
            position.position.x = rand::random::<f32>() * 800.0;
            position.position.y = rand::random::<f32>() * 600.0;
            position.angle = Deg(rand::random::<f32>() * 360.0);
        });

        entity_manager.add_composant_with::<Shape, _>(entity, |shape| {
            if i % 2 == 0 {
                *shape = Shape::Circle;
            } else {
                *shape = Shape::Square;
            }
        });
    }

    let starship_entity = entity_manager.create_entity();

    entity_manager.add_composant_with::<Position, _>(starship_entity, |position| {
        position.position.x = 400.0;
        position.position.y = 300.0;
        position.angle = Deg::zero();
    });

    entity_manager.add_composant_with::<Shape, _>(starship_entity, |shape| {
        *shape = Shape::Triangle;
    });

    entity_manager.add_composant::<Velocity>(starship_entity);

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

    let mut query_bullet = Query::new();
    query_bullet
        .check_composant_by::<Shape, _>(|shape| -> bool { *shape == Shape::Bullet })
        .check_composant::<Position>();

    let mut query_target = Query::new();
    query_target
        .check_composant_by::<Shape, _>(|shape| -> bool {
            *shape == Shape::Square || *shape == Shape::Circle
        })
        .check_composant::<Position>();

    let mut query_velocity = Query::new();
    query_velocity
        .check_composant::<Velocity>()
        .check_composant::<Position>();

    'mainloop: loop {
        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    update_velocity_position(
                        &mut entity_manager,
                        starship_entity,
                        Vector2::new(0.0, 1.0),
                    );
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    update_velocity_position(
                        &mut entity_manager,
                        starship_entity,
                        Vector2::new(0.0, -1.0),
                    );
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    update_velocity_angle(&mut entity_manager, starship_entity, Deg(5.0));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    update_velocity_angle(&mut entity_manager, starship_entity, Deg(-5.0));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    let bullet = entity_manager.create_entity();
                    let matrix = entity_manager
                        .get_composant::<Position>(starship_entity)
                        .get_matrix();
                    let center_pos = (matrix * Vector3::new(0.0, 0.0, 1.0)).truncate();
                    let start_pos = (matrix * Vector3::new(0.0, 10.0, 1.0)).truncate();
                    entity_manager.add_composant_with::<Position, _>(bullet, |position| {
                        position.position = start_pos;
                    });

                    entity_manager.add_composant_with::<Velocity, _>(bullet, |velocity| {
                        velocity.position = (start_pos - center_pos).normalize() * 6.0;
                    });

                    entity_manager.add_composant_with::<Shape, _>(bullet, |shape| {
                        *shape = Shape::Bullet;
                    });
                }
                _ => {}
            }
        }

        for entity in entity_manager.iter(&query_velocity) {
            entity_manager.update_composant_with::<Position, _>(entity, |position| {
                let velocity = entity_manager.get_composant::<Velocity>(entity);

                position.position.x += velocity.position.x;
                position.position.y += velocity.position.y;
                position.angle += velocity.angle;
            });
        }

        let mut delete_entities = Vec::new();
        for bullet_entity in entity_manager.iter(&query_bullet) {
            let bullet_position = entity_manager
                .get_composant::<Position>(bullet_entity)
                .position;
            for target_entity in entity_manager.iter(&query_target) {
                let target_position = entity_manager
                    .get_composant::<Position>(target_entity)
                    .position;
                if (target_position - bullet_position).magnitude() < 10.0 {
                    delete_entities.push(target_entity);
                    delete_entities.push(bullet_entity);
                }
            }
        }

        delete_entities.iter().for_each(|entity| {
            entity_manager.delete_entity(*entity);
        });

        canvas.set_draw_color(Color::BLACK);
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
                Shape::Bullet => {
                    canvas.pixel(
                        position.position.x as i16,
                        position.position.y as i16,
                        Color::WHITE,
                    )?;
                }
                Shape::Square => {
                    let matrix = position.get_matrix();
                    let p1 = matrix * Vector3::new(-10.0, -10.0, 1.0);
                    let p2 = matrix * Vector3::new(-10.0, 10.0, 1.0);
                    let p3 = matrix * Vector3::new(10.0, 10.0, 1.0);
                    let p4 = matrix * Vector3::new(10.0, -10.0, 1.0);
                    canvas.polygon(
                        &[p1.x as i16, p2.x as i16, p3.x as i16, p4.x as i16],
                        &[p1.y as i16, p2.y as i16, p3.y as i16, p4.y as i16],
                        Color::WHITE,
                    )?;
                }
                Shape::Triangle => {
                    let matrix = position.get_matrix();
                    let p1 = matrix * Vector3::new(0.0, 10.0, 1.0);
                    let p2 = matrix * Vector3::new(-8.0, -6.0, 1.0);
                    let p3 = matrix * Vector3::new(8.0, -6.0, 1.0);
                    canvas.polygon(
                        &[p1.x as i16, p2.x as i16, p3.x as i16],
                        &[p1.y as i16, p2.y as i16, p3.y as i16],
                        Color::RED,
                    )?;
                }
            }
        }
        canvas.present();

        // 30 img/sec
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
