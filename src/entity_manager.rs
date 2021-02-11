use crate::entity::{Entity, EntityAllocator, EntityAllocatorIterator};
use crate::storage::Storage;
use std::cell::{Ref, RefMut};

///
/// Create EntityManagerComponent
/// 
/// # Arguments
/// * `name`  name of EntityManagerComponent class
/// * `component` list of component
/// 
/// # Examples
/// ```rust
/// use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
///
/// #[derive(Default)]
/// pub struct Position {
///     pub x: f32,
///     pub y: f32,
/// }
///
/// impl Component for Position {
///     type Storage = BasicVecStorage<Self>;
/// }
/// #[derive(Default)]
/// pub struct Velocity {
///     pub x: f32,
///     pub y: f32,
/// }
///
/// impl Component for Velocity {
///     type Storage = BasicVecStorage<Self>;
/// }
///
/// create_entity_manager_component!(EMC { Position, Velocity});
/// type EntityManager = entity_system::EntityManager<EMC>;
/// type Query = entity_system::Query<EMC>;
///```
#[macro_export]
macro_rules! create_entity_manager_component {
    ($name:ident { $($component:ident),* }) => {
        paste::paste! {
            pub struct $name {
                $(
                [<cpt $component:snake>]: std::cell::RefCell<<$component as entity_system::Component>::Storage>,
                )*
            }

            impl entity_system::EntityManagerComponent for $name {
                fn free(&mut self, entity: entity_system::Entity) {
                    use entity_system::Storage;
                    $(
                    self.[<cpt $component:snake>].borrow_mut().free(entity);
                    )*
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $(
                        [<cpt $component:snake>]: std::cell::RefCell::new(Default::default()),
                        )*
                    }
                }
            }

            $(
            impl entity_system::StorageAccess<$component> for $name
            where
                $component : entity_system::Component,
                <$component as entity_system::Component>::Storage : entity_system::Storage<$component>,
            {
                fn get(&self) -> std::cell::Ref<<$component as entity_system::Component>::Storage> {
                    self.[<cpt $component:snake>].borrow()
                }

                fn get_mut(&self) -> std::cell::RefMut<<$component as entity_system::Component>::Storage> {
                    self.[<cpt $component:snake>].borrow_mut()
                }
            }
            )*
        }
    };
}

///
/// Abstract component type.
///
/// ## Storages
///
/// Components are stored in separated collections for maximum
/// cache efficiency. The `Storage` associated type allows
/// to specify which collection should be used.
/// Depending on how many entities have this component and how
/// often it is accessed, you will want different storages.
///
/// The most common ones are `BasicVecStorage`.
///
/// ## Examples
/// ```rust
/// use entity_system::Component;
/// use entity_system::BasicVecStorage;
///
/// #[derive(Default)]
/// pub struct Position {
///     pub x: f32,
///     pub y: f32,
/// }
///
/// impl Component for Position {
///     type Storage = BasicVecStorage<Self>;
/// }
/// ```
pub trait Component {
    type Storage;
}

///
/// Abstract access to storge by component type.
pub trait StorageAccess<T>
where
    T: Component,
    T::Storage: Storage<T>,
{
    ///
    /// Return ref on Storage
    ///
    /// # Panics
    ///
    /// If Storage could be borrow.
    fn get(&self) -> Ref<T::Storage>;

    ///
    /// Return ref mut on Storage
    ///
    /// # Panics
    ///
    /// If Storage could be borrow mut.
    fn get_mut(&self) -> RefMut<T::Storage>;
}

///
/// Abstract entity manager component type.
pub trait EntityManagerComponent {
    ///
    /// Free all components for entity.
    fn free(&mut self, entity: Entity);
}

///
/// Manage (create, delete, update, iter...) Entities.
///
/// ## Examples
/// ```rust
/// use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
///
/// #[derive(Default)]
/// pub struct Position {
///     pub x: f32,
///     pub y: f32,
/// }
///
/// impl Component for Position {
///     type Storage = BasicVecStorage<Self>;
/// }
///
/// create_entity_manager_component!(EMC { Position });
/// type EntityManager = entity_system::EntityManager<EMC>;
/// ```
#[derive(Default)]
pub struct EntityManager<EntityManagerComponentType>
where
    EntityManagerComponentType: EntityManagerComponent + Default,
{
    components: EntityManagerComponentType,
    allocator: EntityAllocator,
}

impl<EntityManagerComponentType> EntityManager<EntityManagerComponentType>
where
    EntityManagerComponentType: EntityManagerComponent + Default,
{
    ///
    /// Create new instance.
    /// ## Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let entity_manager = EntityManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            components: Default::default(),
            allocator: Default::default(),
        }
    }

    ///
    /// Create new entity.
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();
    /// ```
    pub fn create_entity(&mut self) -> Entity {
        self.allocator.alloc()
    }

    ///
    /// Delete an entity.
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();
    ///
    /// //
    /// // lot of things
    /// //
    ///
    /// entity_manager.delete_entity(entity);
    /// ```
    pub fn delete_entity(&mut self, entity: Entity) {
        self.allocator.free(entity);
        self.components.free(entity);
    }

    ///
    /// Add component to an entity. The component is initialized with default value.
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();
    ///
    /// entity_manager.add_component::<Position>(entity);
    /// ```
    pub fn add_component<T>(&self, entity: Entity)
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
    {
        self.get_storage_mut().alloc(entity);
    }

    ///
    /// Add component to an entity and initialize with closure f.
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();
    ///
    /// entity_manager.add_component_with::<Position, _>(entity, |position|{
    ///   position.x = 20.0;
    ///   position.y = 50.0;
    /// });
    /// ```
    pub fn add_component_with<T, F>(&self, entity: Entity, f: F)
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
        F: FnOnce(&mut T),
    {
        self.get_storage_mut().alloc(entity);
        self.update_component_with(entity, f);
    }

    ///
    /// Remove component to an entity.
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();
    /// entity_manager.add_component::<Position>(entity);
    ///
    /// //
    /// // lot of things
    /// //
    ///
    /// entity_manager.remove_component::<Position>(entity);
    /// ```
    pub fn remove_component<T>(&self, entity: Entity)
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
    {
        self.get_storage_mut().free(entity)
    }

    ///
    /// Check if an entity has a component.
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();
    ///
    /// assert!(!entity_manager.has_component::<Position>(entity));
    /// entity_manager.add_component::<Position>(entity);
    /// assert!(entity_manager.has_component::<Position>(entity));
    /// ```
    pub fn has_component<T>(&self, entity: Entity) -> bool
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
    {
        self.get_storage().has(entity)
    }

    ///
    /// Get a ref of component from an entity.
    ///
    /// # Panics
    ///
    /// if entity has not the component
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();    
    /// entity_manager.add_component::<Position>(entity);
    ///
    /// let position = entity_manager.get_component::<Position>(entity);
    /// println!("{}, {}", position.x, position.y);
    /// ```
    pub fn get_component<T>(&self, entity: Entity) -> Ref<T>
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
    {
        Ref::map(self.get_storage(), |storage| storage.get(entity))
    }

    ///
    /// Get a mut ref of component from an entity.
    ///
    /// # Panics
    ///
    /// if entity has not the component
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();    
    /// entity_manager.add_component::<Position>(entity);
    ///
    /// let mut position = entity_manager.get_component_mut::<Position>(entity);
    /// position.x = 5.0;
    /// position.y = 5.0;
    /// ```
    pub fn get_component_mut<T>(&self, entity: Entity) -> RefMut<T>
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
    {
        RefMut::map(self.get_storage_mut(), |storage| storage.get_mut(entity))
    }

    ///
    /// Update a component from an entity via closure f.
    ///
    /// # Panics
    ///
    /// if entity has not the component
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let entity = entity_manager.create_entity();    
    /// entity_manager.add_component::<Position>(entity);
    ///
    /// entity_manager.update_component_with::<Position, _>(entity, |position|{;
    ///     position.x = 5.0;
    ///     position.y = 5.0;
    /// });
    /// ```
    pub fn update_component_with<T, F>(&self, entity: Entity, f: F)
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
        F: FnOnce(&mut T),
    {
        f(&mut *self.get_component_mut::<T>(entity));
    }

    ///
    /// Iterate on Entity that match the query.
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component, Query};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let e1 = entity_manager.create_entity();
    /// entity_manager.add_component_with::<Position, _>(e1, |position|{;
    ///     position.x = 5.0;
    ///     position.y = 5.0;
    /// });
    /// let e2 = entity_manager.create_entity();
    /// entity_manager.add_component_with::<Position, _>(e2, |position|{;
    ///     position.x = 6.0;
    ///     position.y = 6.0;
    /// });
    ///
    /// let mut query = Query::new();
    /// query.check_component_by::<Position, _>(|position| -> bool {position.x > 5.5});
    ///
    /// println!("e1:{} e2:{}", e1.id, e2.id);
    ///
    /// for entity in entity_manager.iter(&query) {
    ///     println!("entity:{}", entity.id);
    /// }
    /// ```
    ///
    /// The output will be :
    /// ```text
    /// e1:0 e2:1
    /// entity:1
    /// ```
    pub fn iter<'a>(
        &'a self,
        query: &'a Query<EntityManagerComponentType>,
    ) -> EntityIterator<EntityManagerComponentType> {
        EntityIterator::new(query, self)
    }

    ///
    /// Iterate over all Entities.
    ///
    /// # Examples
    /// ```rust
    /// # use entity_system::{Component, BasicVecStorage, create_entity_manager_component, Query};
    /// #
    /// # #[derive(Default)]
    /// # pub struct Position {
    /// #     pub x: f32,
    /// #     pub y: f32,
    /// # }
    /// #
    /// # impl Component for Position {
    /// #     type Storage = BasicVecStorage<Self>;
    /// # }
    /// #
    /// # create_entity_manager_component!(EMC { Position });
    /// # type EntityManager = entity_system::EntityManager<EMC>;
    /// #
    /// let mut entity_manager = EntityManager::new();
    /// let e1 = entity_manager.create_entity();
    /// let e2 = entity_manager.create_entity();
    ///
    /// println!("e1:{} e2:{}", e1.id, e2.id);
    ///
    /// for entity in entity_manager.iter_all() {
    ///     println!("entity:{}", entity.id);
    /// }
    /// ```
    ///
    /// The output will be:
    /// ```text
    /// e1:0 e2:1
    /// entity:0
    /// entity:1
    /// ```
    pub fn iter_all(&self) -> EntityAllocatorIterator {
        self.allocator.iter()
    }

    fn get_storage<T>(&self) -> Ref<<T as Component>::Storage>
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
    {
        self.components.get()
    }

    fn get_storage_mut<T>(&self) -> RefMut<<T as Component>::Storage>
    where
        EntityManagerComponentType: StorageAccess<T>,
        T: Component,
        T::Storage: Storage<T>,
    {
        self.components.get_mut()
    }
}

///
/// EntityIterator over EntityManager.
/// cf [`EntityManager`] to have an example
pub struct EntityIterator<'a, EntityManagerComponentType>
where
    EntityManagerComponentType: EntityManagerComponent + Default,
{
    query: &'a Query<EntityManagerComponentType>,
    entity_manager: &'a EntityManager<EntityManagerComponentType>,
    all_it: EntityAllocatorIterator<'a>,
}

impl<'a, EntityManagerComponentType> EntityIterator<'a, EntityManagerComponentType>
where
    EntityManagerComponentType: EntityManagerComponent + Default,
{
    ///
    /// Create an Iterator
    pub fn new(
        query: &'a Query<EntityManagerComponentType>,
        entity_manager: &'a EntityManager<EntityManagerComponentType>,
    ) -> Self {
        Self {
            query,
            entity_manager,
            all_it: entity_manager.iter_all(),
        }
    }
}

impl<'a, EntityManagerComponentType> Iterator for EntityIterator<'a, EntityManagerComponentType>
where
    EntityManagerComponentType: EntityManagerComponent + Default,
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

type Filter<EntityManagerComponentType> =
    Box<dyn Fn(&EntityManager<EntityManagerComponentType>, Entity) -> bool>;

///
/// Query to select some Entity from EntityManager.
///
/// ## Examples
/// ```rust
/// use entity_system::{Component, BasicVecStorage, create_entity_manager_component};
///
/// #[derive(Default)]
/// pub struct Position {
///     pub x: f32,
///     pub y: f32,
/// }
///
/// impl Component for Position {
///     type Storage = BasicVecStorage<Self>;
/// }
/// #[derive(Default)]
/// pub struct Velocity {
///     pub x: f32,
///     pub y: f32,
/// }
///
/// impl Component for Velocity {
///     type Storage = BasicVecStorage<Self>;
/// }
///
/// create_entity_manager_component!(EMC { Position, Velocity});
/// type EntityManager = entity_system::EntityManager<EMC>;
/// type Query = entity_system::Query<EMC>;
///
/// let mut entity_manager = EntityManager::new();
///
/// //
/// // select entity with component Position
/// let mut query = Query::new();
/// query.check_component::<Position>();
///
/// //
/// // select entity without component Position
/// let mut query = Query::new();
/// query.check_not_component::<Position>();
///
/// //
/// // select entity without component Position and position.x > 5.5
/// let mut query = Query::new();
/// query.check_component_by::<Position, _>(|position| -> bool {position.x > 5.5});
///
/// //
/// // select entity with component Position and Velocity and position.x > 5.5 and velocity.y < 5
/// let mut query = Query::new();
/// query.check_component_by::<Position, _>(|position| -> bool {position.x > 5.5});
/// query.check_component_by::<Velocity, _>(|velocity| -> bool {velocity.y < 5.0});
///
/// //
/// // select entity with component Position and Velocity and position.x + velocity.x > 5
/// let mut query = Query::new();
/// query.check_component::<Position>();
/// query.check_component::<Velocity>();
/// query.check_global(|entity_manager, entity| -> bool {
///     let pos_x = entity_manager.get_component::<Position>(entity).x;
///     let vel_x = entity_manager.get_component::<Velocity>(entity).x;
///     
///     pos_x + vel_x > 5.0
/// });
/// ```
#[derive(Default)]
pub struct Query<EntityManagerComponentType>
where
    EntityManagerComponentType: EntityManagerComponent + Default,
{
    filters: Vec<Filter<EntityManagerComponentType>>,
}

impl<EntityManagerComponentType> Query<EntityManagerComponentType>
where
    EntityManagerComponentType: EntityManagerComponent + Default,
{
    ///
    /// Create a new query
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    ///
    /// Check if query match the entity.
    pub fn check(
        &self,
        entity_manager: &EntityManager<EntityManagerComponentType>,
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

    ///
    /// Check entity has the component.
    pub fn check_component<C>(&mut self) -> &mut Self
    where
        EntityManagerComponentType: StorageAccess<C>,
        C: Component,
        C::Storage: Storage<C>,
    {
        self.filters
            .push(Box::new(|entity_manager, entity| -> bool {
                entity_manager.has_component::<C>(entity)
            }));
        self
    }

    ///
    /// Check entity has not the component.
    pub fn check_not_component<C>(&mut self) -> &mut Self
    where
        EntityManagerComponentType: StorageAccess<C>,
        C: Component,
        C::Storage: Storage<C>,
    {
        self.filters
            .push(Box::new(|entity_manager, entity| -> bool {
                !entity_manager.has_component::<C>(entity)
            }));
        self
    }

    ///
    /// Check entity has the component and the composant match the closure f.
    pub fn check_component_by<C, F>(&mut self, f: F) -> &mut Self
    where
        EntityManagerComponentType: StorageAccess<C>,
        C: Component,
        C::Storage: Storage<C>,
        F: Fn(&C) -> bool + 'static,
    {
        self.filters
            .push(Box::new(move |entity_manager, entity| -> bool {
                if entity_manager.has_component::<C>(entity) {
                    let compostant = entity_manager.get_component::<C>(entity);
                    f(&*compostant)
                } else {
                    false
                }
            }));
        self
    }

    ///
    /// Check if entity match the closure f.
    pub fn check_global<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&EntityManager<EntityManagerComponentType>, Entity) -> bool + 'static,
    {
        self.filters.push(Box::new(f));
        self
    }
}
