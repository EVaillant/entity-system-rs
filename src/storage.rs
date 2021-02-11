use crate::entity::Entity;

///
/// Trait must be implemented to store [`crate::Composant`]
pub trait Storage<T> {
    ///
    /// Allocation an item in the storage
    /// 
    /// # Panics
    /// 
    /// If allocation failed
    fn alloc(&mut self, entity: Entity);

    ///
    /// Free the item in the storage
    fn free(&mut self, entity: Entity);

    ///
    /// Get item from storage
    /// 
    /// # Panics
    /// 
    /// If no allocation has be done before
    fn get(&self, entity: Entity) -> &T;

    ///
    /// Get item from storage (mutable version)
    /// 
    /// # Panics
    /// 
    /// If no allocation has be done before
    fn get_mut(&mut self, entity: Entity) -> &mut T;

    ///
    /// Check if allocatio has been done    
    fn has(&self, entity: Entity) -> bool;
}

///
/// Implementation of Storage<T> with a [`Vec`] as underlying.
/// 
/// # Example
/// ```rust
///     use entity_system::{Entity, Storage, BasicVecStorage};
/// 
///     let mut storage : BasicVecStorage<u32> = Default::default();
///     let entity = Entity::new(0);
/// 
///     // allocation (default value is 0)
///     storage.alloc(entity);
/// 
///     // read the value
///     let val = storage.get(entity);
///     assert!(*val == 0);
/// 
///     // update the value
///     let val = storage.get_mut(entity);
///     *val = 5;
/// 
///     // free
///     storage.free(entity); 
/// ```
#[derive(Default)]
pub struct BasicVecStorage<T>
where
    T: Default,
{
    datas: Vec<T>,
    alloc: Vec<bool>,
}

impl<T> Storage<T> for BasicVecStorage<T>
where
    T: Default,
{
    fn alloc(&mut self, entity: Entity) {
        let pos = entity.id as usize;
        if pos >= self.datas.len() {
            self.datas.resize_with(pos + 1, Default::default);
            self.alloc.resize_with(pos + 1, Default::default);
        }
        self.alloc[pos] = true;
    }

    fn free(&mut self, entity: Entity) {
        let pos = entity.id as usize;
        if pos < self.datas.len() && self.alloc[pos] {
            self.datas[pos] = Default::default();
            self.alloc[pos] = false;
        }
    }

    fn get(&self, entity: Entity) -> &T {
        let pos = entity.id as usize;
        if pos < self.datas.len() && self.alloc[pos] {
            self.datas.get(pos).unwrap()
        } else {
            panic!("index is out of bounds or not allocated");
        }
    }

    fn get_mut(&mut self, entity: Entity) -> &mut T {
        let pos = entity.id as usize;
        if pos < self.datas.len() && self.alloc[pos] {
            self.datas.get_mut(pos).unwrap()
        } else {
            panic!("index is out of bounds or not allocated");
        }
    }

    fn has(&self, entity: Entity) -> bool {
        let pos = entity.id as usize;
        pos < self.datas.len() && self.alloc[pos]
    }
}
