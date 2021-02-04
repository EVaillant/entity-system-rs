use entity_system::{
    create_entity_manager_composant, BasicVecStorage, Composant, EntityManager, Query,
};

#[derive(Default)]
struct Position {
    x: u32,
    y: u32,
}

impl Composant for Position {
    type Storage = BasicVecStorage<Position>;
}

#[derive(Default)]
struct Velocity {
    x: i32,
    y: i32,
}

impl Composant for Velocity {
    type Storage = BasicVecStorage<Velocity>;
}

create_entity_manager_composant!(EMC { Position, Velocity });
type MyEntityManager = EntityManager<EMC>;

#[test]
fn test_entity_manager_01() {
    let mut entity_manager = MyEntityManager::new();
    let e = entity_manager.create_entity();
    assert!(!entity_manager.has_composant::<Position>(e));
    assert!(!entity_manager.has_composant::<Velocity>(e));
    entity_manager.add_composant::<Position>(e);
    assert!(entity_manager.has_composant::<Position>(e));
    assert!(!entity_manager.has_composant::<Velocity>(e));

    {
        let position = entity_manager.get_composant::<Position>(e);
        assert!(position.is_some());
        let velocity = entity_manager.get_composant::<Velocity>(e);
        assert!(velocity.is_none());
    }

    {
        let mut position = entity_manager.get_composant_mut::<Position>(e).unwrap();
        position.x = 5;
        position.y = 6;
    }

    {
        let position = entity_manager.get_composant::<Position>(e).unwrap();
        assert_eq!(position.x, 5);
        assert_eq!(position.y, 6);
    }

    entity_manager.remove_composant::<Position>(e);
    entity_manager.add_composant::<Velocity>(e);
    assert!(!entity_manager.has_composant::<Position>(e));
    assert!(entity_manager.has_composant::<Velocity>(e));

    {
        let position = entity_manager.get_composant::<Position>(e);
        assert!(position.is_none());
        let velocity = entity_manager.get_composant::<Velocity>(e);
        assert!(velocity.is_some());
    }

    {
        let mut velocity = entity_manager.get_composant_mut::<Velocity>(e).unwrap();
        velocity.x = 5;
        velocity.y = 6;
    }

    {
        let velocity = entity_manager.get_composant::<Velocity>(e).unwrap();
        assert_eq!(velocity.x, 5);
        assert_eq!(velocity.y, 6);
    }

    entity_manager.delete_entity(e);
    assert!(!entity_manager.has_composant::<Position>(e));
    assert!(!entity_manager.has_composant::<Velocity>(e));
}

#[test]
fn test_entity_manager_02() {
    let mut entity_manager = MyEntityManager::new();
    let e1 = entity_manager.create_entity();
    entity_manager.add_composant::<Position>(e1);
    let e2 = entity_manager.create_entity();
    entity_manager.add_composant::<Position>(e2);
    entity_manager.add_composant::<Velocity>(e2);

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
        query.check_composant::<Position>();
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
        query.check_composant::<Position>();
        query.check_composant::<Velocity>();
        let mut r = Vec::new();
        for entity in entity_manager.iter(&query) {
            if !r.contains(&entity) && (entity == e2) {
                r.push(entity);
            }
        }
        assert_eq!(r.len(), 1);
    }

    {
        let mut position = entity_manager.get_composant_mut::<Position>(e1).unwrap();
        position.x = 5;

        let mut query = Query::new();
        query.check_composant_by::<Position, _>(|position| -> bool { position.x > 2 });
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
        query.check_not_composant::<Velocity>();
        let mut r = Vec::new();
        for entity in entity_manager.iter(&query) {
            if !r.contains(&entity) && (entity == e1) {
                r.push(entity);
            }
        }
        assert_eq!(r.len(), 1);
    }
}
