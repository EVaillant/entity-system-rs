use entity_system::{Entity, EntityAllocator};

#[test]
fn test_entity_01() {
    let mut ea = EntityAllocator::new();

    // alloc : 0 1 2 3 4 5 6 7 8 9
    for i in 0..10 {
        assert_eq!(ea.alloc().id, i);
    }

    // free : 0 2 4 6 8
    for i in 0..5 {
        ea.free(Entity::new(i * 2));
    }

    // alloc : 0 2 4 6 8 10 11
    let mut v1 = Vec::new();
    for _ in 0..7 {
        v1.push(ea.alloc().id);
    }
    v1.sort_unstable();
    assert_eq!(v1, [0, 2, 4, 6, 8, 10, 11]);

    // iter
    let mut v2 = Vec::new();
    for e in ea.iter() {
        v2.push(e.id);
    }
    v2.sort_unstable();
    assert_eq!(v2, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
}
