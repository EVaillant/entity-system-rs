use entity_system::{
    create_entity_manager_component, BasicVecStorage, Component, EntityManager, Query,
};

#[derive(Default)]
struct Position {
    x: u32,
    y: u32,
}

impl Component for Position {
    type Storage = BasicVecStorage<Position>;
}

#[derive(Default)]
struct Velocity {
    x: i32,
    y: i32,
}

impl Component for Velocity {
    type Storage = BasicVecStorage<Velocity>;
}

create_entity_manager_component!(EMC { Position, Velocity });
type MyEntityManager = EntityManager<EMC>;

#[test]
fn test_entity_manager_01() {
    let mut entity_manager = MyEntityManager::new();
    let e = entity_manager.create_entity();
    assert!(!entity_manager.has_component::<Position>(e));
    assert!(!entity_manager.has_component::<Velocity>(e));
    entity_manager.add_component::<Position>(e);
    assert!(entity_manager.has_component::<Position>(e));
    assert!(!entity_manager.has_component::<Velocity>(e));

    entity_manager.update_component_with::<Position, _>(e, |position| {
        position.x = 5;
        position.y = 6;
    });

    {
        let position = entity_manager.get_component::<Position>(e);
        assert_eq!(position.x, 5);
        assert_eq!(position.y, 6);
    }

    entity_manager.remove_component::<Position>(e);
    entity_manager.add_component::<Velocity>(e);
    assert!(!entity_manager.has_component::<Position>(e));
    assert!(entity_manager.has_component::<Velocity>(e));

    entity_manager.update_component_with::<Velocity, _>(e, |velocity| {
        velocity.x = 5;
        velocity.y = 6;
    });

    {
        let velocity = entity_manager.get_component::<Velocity>(e);
        assert_eq!(velocity.x, 5);
        assert_eq!(velocity.y, 6);
    }

    entity_manager.delete_entity(e);
    assert!(!entity_manager.has_component::<Position>(e));
    assert!(!entity_manager.has_component::<Velocity>(e));
}

#[test]
fn test_entity_manager_02() {
    let mut entity_manager = MyEntityManager::new();
    let e1 = entity_manager.create_entity();
    entity_manager.add_component::<Position>(e1);
    let e2 = entity_manager.create_entity();
    entity_manager.add_component::<Position>(e2);
    entity_manager.add_component::<Velocity>(e2);

    {
        let mut r = Vec::new();
        for entity in entity_manager.iter_all() {
            if !r.contains(&entity) && (entity == e1 || entity == e2) {
                r.push(entity);
            }
        }
        assert_eq!(r.len(), 2);
    }

    {
        let mut query = Query::new();
        query.check_component::<Position>();
        let mut r = Vec::new();
        for entity in entity_manager.iter(&query) {
            if !r.contains(&entity) && (entity == e1 || entity == e2) {
                r.push(entity);
            }
        }
        assert_eq!(r.len(), 2);
    }

    {
        let mut query = Query::new();
        query.check_component::<Position>();
        query.check_component::<Velocity>();
        let mut r = Vec::new();
        for entity in entity_manager.iter(&query) {
            if !r.contains(&entity) && (entity == e2) {
                r.push(entity);
            }
        }
        assert_eq!(r.len(), 1);
    }

    {
        entity_manager.update_component_with::<Position, _>(e1, |position| {
            position.x = 5;
        });

        let mut query = Query::new();
        query.check_component_by::<Position, _>(|position| -> bool { position.x > 2 });
        let mut r = Vec::new();
        for entity in entity_manager.iter(&query) {
            if !r.contains(&entity) && (entity == e1) {
                r.push(entity);
            }
        }
        assert_eq!(r.len(), 1);
    }

    {
        let mut query = Query::new();
        query.check_not_component::<Velocity>();
        let mut r = Vec::new();
        for entity in entity_manager.iter(&query) {
            if !r.contains(&entity) && (entity == e1) {
                r.push(entity);
            }
        }
        assert_eq!(r.len(), 1);
    }
}
