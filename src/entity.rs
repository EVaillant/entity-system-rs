use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Entity {
    pub id: u32,
}

impl Entity {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

pub struct EntityAllocator {
    next: Entity,
    free: HashSet<Entity>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            next: Entity::new(0),
            free: HashSet::new(),
        }
    }

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

    pub fn free(&mut self, entity: Entity) {
        self.free.insert(entity);
    }

    pub fn iter(&self) -> EntityAllocatorIterator {
        EntityAllocatorIterator::new(self)
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EntityAllocatorIterator<'a> {
    allocator: &'a EntityAllocator,
    current: Entity,
}

impl<'a> EntityAllocatorIterator<'a> {
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
