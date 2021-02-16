use std::collections::HashSet;

///
/// Entity type, as seen by the user.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Entity {
    /// id of Entity
    pub id: u32,
}

impl Entity {
    ///
    /// Create a new `Entity`
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

///
/// Entity Factory.
///
/// # Example
///
/// * Allocation & free Entity
/// ```rust
/// use entity_system::EntityAllocator;
///
/// // create allocator
/// let mut allocator = EntityAllocator::new();
///
/// // alloc a new entity
/// let entity1 = allocator.alloc();
/// let id1 = entity1.id;
///
/// // free the entity
/// allocator.free(entity1);
///
/// // re-alloc an entity
/// let entity2 = allocator.alloc();
/// let id2 = entity2.id;
/// assert!(id2 == id1);
/// ```
///
/// * Iter
/// ```rust
/// use entity_system::{EntityAllocator, Entity};
///
/// let mut allocator = EntityAllocator::new();
///
/// // alloc 5 entity
/// for _ in 0..5 {
///     allocator.alloc();
/// }
///
/// // free 2 entities
/// allocator.free(Entity::new(1));
/// allocator.free(Entity::new(3));
///
/// // iter over allocated entity
/// for entity in allocator.iter() {
///     println!("{}", entity.id);
/// }
/// ```
/// Output will be :
/// ```text
/// 0
/// 2
/// 4
/// ```
pub struct EntityAllocator {
    next: Entity,
    free: HashSet<Entity>,
}

impl EntityAllocator {
    ///
    /// Create a new `EntityAllocator`
    pub fn new() -> Self {
        Self {
            next: Entity::new(0),
            free: HashSet::new(),
        }
    }

    ///
    /// Alloc a new `Entity`
    pub fn alloc(&mut self) -> Entity {
        match self.free.iter().next() {
            Some(&value) => {
                self.free.remove(&value);
                Entity::new(value.id)
            }
            None => {
                let value = self.next;
                self.next = Entity::new(self.next.id + 1);
                value
            }
        }
    }

    ///
    /// Free an `Entity`. `Entity` id could be re-used
    pub fn free(&mut self, entity: Entity) {
        self.free.insert(entity);
    }

    ///
    /// Iter over allocated `Entity`
    pub fn iter(&self) -> EntityAllocatorIterator {
        EntityAllocatorIterator::new(self)
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

///
/// Iterator over [`EntityAllocator`].
///
/// Cf [`EntityAllocator`] to have an example
pub struct EntityAllocatorIterator<'a> {
    allocator: &'a EntityAllocator,
    current: Entity,
}

impl<'a> EntityAllocatorIterator<'a> {
    ///
    /// Create an Iterator
    pub fn new(allocator: &'a EntityAllocator) -> Self {
        let mut it = Self {
            allocator,
            current: Entity::new(0),
        };
        it.next_free_entity();
        it
    }

    fn next_free_entity(&mut self) {
        while self.allocator.free.contains(&self.current) {
            self.current = Entity::new(self.current.id + 1);
            if self.current == self.allocator.next {
                break;
            }
        }
    }
}

impl<'a> Iterator for EntityAllocatorIterator<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        if self.current == self.allocator.next {
            None
        } else {
            let current = self.current;
            self.current = Entity::new(self.current.id + 1);
            self.next_free_entity();
            Some(current)
        }
    }
}
