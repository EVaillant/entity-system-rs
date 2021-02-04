use crate::entity::{Entity, EntityAllocator, EntityAllocatorIterator};
use crate::storage::Storage;

#[macro_export]
macro_rules! create_entity_manager_composant {
    ($name:ident { $($composant:ident),* }) => {
        paste::paste! {
            #[derive(Default)]
            pub struct $name {
                $(
                [<cpt $composant:snake>]: <$composant as Composant>::Storage,
                )*
            }

            impl entity_system::EntityManagerComposant for $name {
                fn free(&mut self, entity: entity_system::Entity) {
                    use entity_system::Storage;
                    $(
                    self.[<cpt $composant:snake>].free(entity);
                    )*
                }
            }

            $(
            impl entity_system::StorageAccess<$composant> for $name {
                fn get(&self) -> &dyn entity_system::Storage<$composant> {
                    &self.[<cpt $composant:snake>]
                }

                fn get_mut(&mut self) -> &mut dyn entity_system::Storage<$composant> {
                    &mut self.[<cpt $composant:snake>]
                }
            }
            )*
        }
    };
}

pub trait Composant {
    type Storage;
}

pub trait StorageAccess<T> {
    fn get(&self) -> &dyn Storage<T>;
    fn get_mut(&mut self) -> &mut dyn Storage<T>;
}

pub trait EntityManagerComposant {
    fn free(&mut self, entity: Entity);
}

#[derive(Default)]
pub struct EntityManager<EntityManagerComposantType>
where
    EntityManagerComposantType: EntityManagerComposant + Default,
{
    composants: EntityManagerComposantType,
    allocator: EntityAllocator,
}

impl<EntityManagerComposantType> EntityManager<EntityManagerComposantType>
where
    EntityManagerComposantType: EntityManagerComposant + Default,
{
    pub fn new() -> Self {
        Self {
            composants: Default::default(),
            allocator: Default::default(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self.allocator.alloc()
    }

    pub fn delete_entity(&mut self, entity: Entity) {
        self.allocator.free(entity);
        self.composants.free(entity);
    }

    pub fn add_composant<T>(&mut self, entity: Entity)
    where
        EntityManagerComposantType: StorageAccess<T>,
    {
        self.get_storage_mut().alloc(entity)
    }

    pub fn remove_composant<T>(&mut self, entity: Entity)
    where
        EntityManagerComposantType: StorageAccess<T>,
    {
        self.get_storage_mut().free(entity)
    }

    pub fn has_composant<T>(&self, entity: Entity) -> bool
    where
        EntityManagerComposantType: StorageAccess<T>,
    {
        self.get_storage().has(entity)
    }

    pub fn get_composant<T>(&self, entity: Entity) -> Option<&T>
    where
        EntityManagerComposantType: StorageAccess<T>,
    {
        self.get_storage().get(entity)
    }

    pub fn get_composant_mut<T>(&mut self, entity: Entity) -> Option<&mut T>
    where
        EntityManagerComposantType: StorageAccess<T>,
    {
        self.get_storage_mut().get_mut(entity)
    }

    pub fn iter<'a>(
        &'a self,
        query: &'a Query<EntityManagerComposantType>,
    ) -> EntityIterator<EntityManagerComposantType> {
        EntityIterator::new(query, self)
    }

    pub fn iter_all(&self) -> EntityAllocatorIterator {
        self.allocator.iter()
    }

    fn get_storage<T>(&self) -> &dyn Storage<T>
    where
        EntityManagerComposantType: StorageAccess<T>,
    {
        self.composants.get()
    }

    fn get_storage_mut<T>(&mut self) -> &mut dyn Storage<T>
    where
        EntityManagerComposantType: StorageAccess<T>,
    {
        self.composants.get_mut()
    }
}

pub struct EntityIterator<'a, EntityManagerComposantType>
where
    EntityManagerComposantType: EntityManagerComposant + Default,
{
    query: &'a Query<EntityManagerComposantType>,
    entity_manager: &'a EntityManager<EntityManagerComposantType>,
    all_it: EntityAllocatorIterator<'a>,
}

impl<'a, EntityManagerComposantType> EntityIterator<'a, EntityManagerComposantType>
where
    EntityManagerComposantType: EntityManagerComposant + Default,
{
    pub fn new(
        query: &'a Query<EntityManagerComposantType>,
        entity_manager: &'a EntityManager<EntityManagerComposantType>,
    ) -> Self {
        Self {
            query,
            entity_manager,
            all_it: entity_manager.iter_all(),
        }
    }
}

impl<'a, EntityManagerComposantType> Iterator for EntityIterator<'a, EntityManagerComposantType>
where
    EntityManagerComposantType: EntityManagerComposant + Default,
{
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        loop {
            let entity = self.all_it.next();
            match entity {
                Some(entity) => {
                    if self.query.check(self.entity_manager, entity) {
                        return Some(entity);
                    } else {
                        continue;
                    }
                }
                None => break,
            }
        }
        None
    }
}

type Filter<EntityManagerComposantType> =
    Box<dyn Fn(&EntityManager<EntityManagerComposantType>, Entity) -> bool>;

#[derive(Default)]
pub struct Query<EntityManagerComposantType>
where
    EntityManagerComposantType: EntityManagerComposant + Default,
{
    filters: Vec<Filter<EntityManagerComposantType>>,
}

impl<EntityManagerComposantType> Query<EntityManagerComposantType>
where
    EntityManagerComposantType: EntityManagerComposant + Default,
{
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn check(
        &self,
        entity_manager: &EntityManager<EntityManagerComposantType>,
        entity: Entity,
    ) -> bool {
        let mut ret = true;
        for filter in self.filters.iter() {
            ret = (filter)(entity_manager, entity);
            if !ret {
                break;
            }
        }
        ret
    }

    pub fn check_composant<C>(&mut self) -> &mut Self
    where
        EntityManagerComposantType: StorageAccess<C>,
    {
        self.filters
            .push(Box::new(|entity_manager, entity| -> bool {
                entity_manager.has_composant::<C>(entity)
            }));
        self
    }

    pub fn check_not_composant<C>(&mut self) -> &mut Self
    where
        EntityManagerComposantType: StorageAccess<C>,
    {
        self.filters
            .push(Box::new(|entity_manager, entity| -> bool {
                !entity_manager.has_composant::<C>(entity)
            }));
        self
    }

    pub fn check_composant_by<C, F>(&mut self, f: F) -> &mut Self
    where
        EntityManagerComposantType: StorageAccess<C>,
        F: Fn(&C) -> bool + 'static,
    {
        self.filters
            .push(Box::new(move |entity_manager, entity| -> bool {
                match entity_manager.get_composant::<C>(entity) {
                    Some(composant) => (f)(composant),
                    None => false,
                }
            }));
        self
    }

    pub fn check_global<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&EntityManager<EntityManagerComposantType>, Entity) -> bool + 'static,
    {
        self.filters.push(Box::new(f));
        self
    }
}
