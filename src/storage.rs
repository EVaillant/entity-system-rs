use crate::entity::Entity;

pub trait Storage<T> {
    fn alloc(&mut self, entity: Entity);
    fn free(&mut self, entity: Entity);
    fn get(&self, entity: Entity) -> &T;
    fn get_mut(&mut self, entity: Entity) -> &mut T;
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
