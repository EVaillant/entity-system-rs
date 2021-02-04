use crate::entity::Entity;

pub trait Storage<T> {
    fn alloc(&mut self, entity: Entity);
    fn free(&mut self, entity: Entity);
    fn get(&self, entity: Entity) -> Option<&T>;
    fn get_mut(&mut self, entity: Entity) -> Option<&mut T>;
    fn has(&self, entity: Entity) -> bool;
}

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

    fn get(&self, entity: Entity) -> Option<&T> {
        let pos = entity.id as usize;
        if pos < self.datas.len() && self.alloc[pos] {
            self.datas.get(pos)
        } else {
            None
        }
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let pos = entity.id as usize;
        if pos < self.datas.len() && self.alloc[pos] {
            self.datas.get_mut(pos)
        } else {
            None
        }
    }

    fn has(&self, entity: Entity) -> bool {
        let pos = entity.id as usize;
        pos < self.datas.len() && self.alloc[pos]
    }
}
