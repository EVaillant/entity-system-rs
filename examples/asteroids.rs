use cgmath::{prelude::*, Deg, Matrix3, Vector2, Vector3};

entity_system::create_event_adapters!(EventAdapters {});
type EventDispatcher = entity_system::EventDispatcher<EventAdapters>;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

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

impl entity_system::Component for Position {
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

impl entity_system::Component for Velocity {
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

impl entity_system::Component for Shape {
    type Storage = entity_system::BasicVecStorage<Self>;
}

entity_system::create_entity_manager_component!(EMC {
    Position,
    Velocity,
    Shape
});

type EntityManager = entity_system::EntityManager<EMC>;
type Query = entity_system::Query<EMC>;
type Entity = entity_system::Entity;

struct Move {
    entity_manager: Rc<RefCell<EntityManager>>,
    query_velocity: Query,
}

impl Move {
    fn new(entity_manager: Rc<RefCell<EntityManager>>) -> Self {
        let mut query_velocity = Query::new();
        query_velocity
            .check_component::<Velocity>()
            .check_component::<Position>();

        Self {
            entity_manager,
            query_velocity,
        }
    }
}

impl entity_system::System for Move {
    fn name(&self) -> &'static str {
        "move"
    }

    fn run(&mut self, _now: Instant) -> entity_system::RefreshPeriod {
        let entity_manager = self.entity_manager.borrow();
        for entity in entity_manager.iter(&self.query_velocity) {
            entity_manager.update_component_with::<Position, _>(entity, |position| {
                let velocity = entity_manager.get_component::<Velocity>(entity);

                position.position.x += velocity.position.x;
                position.position.y += velocity.position.y;
                position.angle += velocity.angle;
            });
        }

        entity_system::RefreshPeriod::At(Instant::now() + Duration::from_millis(20))
    }
}

struct Draw {
    entity_manager: Rc<RefCell<EntityManager>>,
    query_draw: Query,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl Draw {
    fn new(entity_manager: Rc<RefCell<EntityManager>>, sdl_context: &sdl2::Sdl) -> Self {
        let mut query_draw = Query::new();
        query_draw
            .check_component::<Shape>()
            .check_component::<Position>();

        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("asteroids demo", 800, 600)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        Self {
            entity_manager,
            query_draw,
            canvas: window
                .into_canvas()
                .build()
                .map_err(|e| e.to_string())
                .unwrap(),
        }
    }
}

impl entity_system::System for Draw {
    fn name(&self) -> &'static str {
        "draw"
    }

    fn run(&mut self, _now: Instant) -> entity_system::RefreshPeriod {
        let entity_manager = self.entity_manager.borrow();

        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        for entity in entity_manager.iter(&self.query_draw) {
            let position = entity_manager.get_component::<Position>(entity);
            let shape = entity_manager.get_component::<Shape>(entity);
            match *shape {
                Shape::Circle => {
                    self.canvas
                        .circle(
                            position.position.x as i16,
                            position.position.y as i16,
                            10,
                            Color::WHITE,
                        )
                        .unwrap();
                }
                Shape::Bullet => {
                    self.canvas
                        .pixel(
                            position.position.x as i16,
                            position.position.y as i16,
                            Color::WHITE,
                        )
                        .unwrap();
                }
                Shape::Square => {
                    let matrix = position.get_matrix();
                    let p1 = matrix * Vector3::new(-10.0, -10.0, 1.0);
                    let p2 = matrix * Vector3::new(-10.0, 10.0, 1.0);
                    let p3 = matrix * Vector3::new(10.0, 10.0, 1.0);
                    let p4 = matrix * Vector3::new(10.0, -10.0, 1.0);
                    self.canvas
                        .polygon(
                            &[p1.x as i16, p2.x as i16, p3.x as i16, p4.x as i16],
                            &[p1.y as i16, p2.y as i16, p3.y as i16, p4.y as i16],
                            Color::WHITE,
                        )
                        .unwrap();
                }
                Shape::Triangle => {
                    let matrix = position.get_matrix();
                    let p1 = matrix * Vector3::new(0.0, 10.0, 1.0);
                    let p2 = matrix * Vector3::new(-8.0, -6.0, 1.0);
                    let p3 = matrix * Vector3::new(8.0, -6.0, 1.0);
                    self.canvas
                        .polygon(
                            &[p1.x as i16, p2.x as i16, p3.x as i16],
                            &[p1.y as i16, p2.y as i16, p3.y as i16],
                            Color::RED,
                        )
                        .unwrap();
                }
            }
        }
        self.canvas.present();
        entity_system::RefreshPeriod::At(Instant::now() + Duration::new(0, 1_000_000_000u32 / 30))
    }
}

struct Hit {
    entity_manager: Rc<RefCell<EntityManager>>,
    query_bullet: Query,
    query_target: Query,
}

impl Hit {
    fn new(entity_manager: Rc<RefCell<EntityManager>>) -> Self {
        let mut query_bullet = Query::new();
        query_bullet
            .check_component_by::<Shape, _>(|shape| -> bool { *shape == Shape::Bullet })
            .check_component::<Position>();

        let mut query_target = Query::new();
        query_target
            .check_component_by::<Shape, _>(|shape| -> bool {
                *shape == Shape::Square || *shape == Shape::Circle
            })
            .check_component::<Position>();

        Self {
            entity_manager,
            query_bullet,
            query_target,
        }
    }
}

impl entity_system::System for Hit {
    fn name(&self) -> &'static str {
        "hit"
    }

    fn run(&mut self, _now: Instant) -> entity_system::RefreshPeriod {
        let entity_manager = self.entity_manager.borrow();

        let mut delete_entities = Vec::new();
        for bullet_entity in entity_manager.iter(&self.query_bullet) {
            let bullet_position = entity_manager
                .get_component::<Position>(bullet_entity)
                .position;
            for target_entity in entity_manager.iter(&self.query_target) {
                let target_position = entity_manager
                    .get_component::<Position>(target_entity)
                    .position;
                if (target_position - bullet_position).magnitude() < 10.0 {
                    delete_entities.push(target_entity);
                    delete_entities.push(bullet_entity);
                }
            }
        }
        drop(entity_manager);

        let mut entity_manager = self.entity_manager.borrow_mut();
        delete_entities.iter().for_each(|entity| {
            entity_manager.delete_entity(*entity);
        });

        entity_system::RefreshPeriod::EveryTime
    }
}

struct Keyboard {
    entity_manager: Rc<RefCell<EntityManager>>,
    starship_entity: Entity,
    event_pump: sdl2::EventPump,
    stop: bool,
}

impl Keyboard {
    fn new(
        entity_manager: Rc<RefCell<EntityManager>>,
        starship_entity: Entity,
        sdl_context: &sdl2::Sdl,
    ) -> Self {
        Self {
            entity_manager,
            starship_entity,
            event_pump: sdl_context.event_pump().unwrap(),
            stop: false,
        }
    }

    fn quit(&self) -> bool {
        self.stop
    }

    fn update_velocity_angle(&self, delta_angle: Deg<f32>) {
        let entity_manager = self.entity_manager.borrow();
        entity_manager.update_component_with::<Velocity, _>(self.starship_entity, |velocity| {
            velocity.angle += delta_angle;
        });
    }

    fn update_velocity_position(&self, delta_position: Vector2<f32>) {
        let entity_manager = self.entity_manager.borrow();
        let position = entity_manager.get_component::<Position>(self.starship_entity);
        let delta = (Matrix3::from_angle_z(position.angle) * delta_position.extend(1.0)).truncate();
        entity_manager.update_component_with::<Velocity, _>(self.starship_entity, |velocity| {
            velocity.position += delta;
            velocity.position.x = velocity.position.x.min(5.0).max(-5.0);
            velocity.position.y = velocity.position.y.min(5.0).max(-5.0);
        });
    }
}

impl entity_system::System for Keyboard {
    fn name(&self) -> &'static str {
        "keyboard"
    }

    fn run(&mut self, _now: Instant) -> entity_system::RefreshPeriod {
        while let Some(event) = self.event_pump.poll_event() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => {
                    self.stop = true;
                    break;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    self.update_velocity_position(Vector2::new(0.0, 1.0));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    self.update_velocity_position(Vector2::new(0.0, -1.0));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    self.update_velocity_angle(Deg(5.0));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    self.update_velocity_angle(Deg(-5.0));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    let mut entity_manager = self.entity_manager.borrow_mut();
                    let bullet = entity_manager.create_entity();
                    let matrix = entity_manager
                        .get_component::<Position>(self.starship_entity)
                        .get_matrix();
                    let center_pos = (matrix * Vector3::new(0.0, 0.0, 1.0)).truncate();
                    let start_pos = (matrix * Vector3::new(0.0, 10.0, 1.0)).truncate();
                    entity_manager.add_component_with::<Position, _>(bullet, |position| {
                        position.position = start_pos;
                    });

                    entity_manager.add_component_with::<Velocity, _>(bullet, |velocity| {
                        velocity.position = (start_pos - center_pos).normalize() * 6.0;
                    });

                    entity_manager.add_component_with::<Shape, _>(bullet, |shape| {
                        *shape = Shape::Bullet;
                    });
                }
                _ => {}
            }
        }

        entity_system::RefreshPeriod::EveryTime
    }
}

fn create_target(count: usize, entity_manager: &mut EntityManager) {
    let entity = entity_manager.create_entity();

    entity_manager.add_component_with::<Position, _>(entity, |position| {
        position.position.x = rand::random::<f32>() * 800.0;
        position.position.y = rand::random::<f32>() * 600.0;
        position.angle = Deg(rand::random::<f32>() * 360.0);
    });

    entity_manager.add_component_with::<Shape, _>(entity, |shape| {
        if count % 2 == 0 {
            *shape = Shape::Circle;
        } else {
            *shape = Shape::Square;
        }
    });
}

fn create_starship(entity_manager: &mut EntityManager) -> Entity {
    let entity = entity_manager.create_entity();

    entity_manager.add_component_with::<Position, _>(entity, |position| {
        position.position.x = 400.0;
        position.position.y = 300.0;
        position.angle = Deg::zero();
    });

    entity_manager.add_component_with::<Shape, _>(entity, |shape| {
        *shape = Shape::Triangle;
    });

    entity_manager.add_component::<Velocity>(entity);
    entity
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let event_dispatcher = EventDispatcher::new();
    let entity_manager = Rc::new(RefCell::new(EntityManager::new()));

    for i in 0..20 {
        create_target(i, &mut *entity_manager.borrow_mut());
    }
    let starship_entity = create_starship(&mut *entity_manager.borrow_mut());

    let mut system_manager = entity_system::SystemManager::new();
    system_manager.add_system(Rc::new(RefCell::new(Move::new(Rc::clone(&entity_manager)))));
    system_manager.add_system(Rc::new(RefCell::new(Hit::new(Rc::clone(&entity_manager)))));
    system_manager.add_system(Rc::new(RefCell::new(Draw::new(
        Rc::clone(&entity_manager),
        &sdl_context,
    ))));
    let keyboard_system = Rc::new(RefCell::new(Keyboard::new(
        Rc::clone(&entity_manager),
        starship_entity,
        &sdl_context,
    )));
    system_manager.add_system(Rc::clone(&keyboard_system));

    while !keyboard_system.borrow().quit() {
        system_manager.update(&event_dispatcher);
    }

    Ok(())
}
