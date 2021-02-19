use crate::event_dispatcher::EventDispatcher;
use std::cell::RefCell;
use std::cmp::{max, Ord, Ordering};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

///
/// Definie the system execution period.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RefreshPeriod {
    ///
    /// Each time.
    EveryTime,
    ///
    /// After a date.
    At(Instant),
    ///
    /// Stop to refresh.
    Stop,
}

impl Ord for RefreshPeriod {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (RefreshPeriod::EveryTime, RefreshPeriod::EveryTime) => Ordering::Equal,
            (RefreshPeriod::EveryTime, _) => Ordering::Greater,
            (RefreshPeriod::At(_), RefreshPeriod::EveryTime) => Ordering::Less,
            (RefreshPeriod::At(self_time), RefreshPeriod::At(other_time)) => {
                self_time.cmp(other_time)
            }
            (RefreshPeriod::At(_), RefreshPeriod::Stop) => Ordering::Greater,
            (RefreshPeriod::Stop, RefreshPeriod::Stop) => Ordering::Equal,
            (RefreshPeriod::Stop, _) => Ordering::Less,
        }
    }
}

impl PartialOrd for RefreshPeriod {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

///
/// Abstract system type.
///
/// It will be executed by [`SystemManager`]
/// # Example
/// ```rust
/// use entity_system::{System, RefreshPeriod};
/// use std::time::Instant;
///
/// struct MoveSystem {
/// }
///
/// impl System for MoveSystem {
///     fn name(&self) -> &'static str {
///         "move"
///     }
///
///     fn run(&mut self, now : Instant) -> RefreshPeriod {
///         //
///         // Do lot of thing
///         //
///
///         RefreshPeriod::EveryTime
///     }         
/// }
/// ```
pub trait System {
    ///
    /// Get the system name.
    ///
    /// The name must be unique and constant.
    fn name(&self) -> &'static str;

    ///
    /// Execute the system.
    ///
    /// # Arguments
    /// * `now` system rum time.
    ///
    /// # Return
    ///
    /// The next execution time.
    fn run(&mut self, now: Instant) -> RefreshPeriod;
}

///
/// Manage & Execute [`System`]\(s)
///
/// # Example
/// ```rust
/// use std::cell::RefCell;
/// use std::rc::Rc;
/// use entity_system::{SystemManager, System, RefreshPeriod};
/// use std::time::Instant;
///
/// entity_system::create_event_adapters!(EventAdapters {});
/// type EventDispatcher = entity_system::EventDispatcher<EventAdapters>;
///
/// struct MoveSystem {
/// }
///
/// impl System for MoveSystem {
///     fn name(&self) -> &'static str {
///         "move"
///     }
///
///     fn run(&mut self, now : Instant) -> RefreshPeriod {
///         //
///         // Do lot of thing
///         //
///
///         RefreshPeriod::EveryTime
///     }         
/// }
///
/// let event_dispatcher = EventDispatcher::new();
/// let mut system_manager = SystemManager::new();
/// system_manager.add_system(Rc::new(RefCell::new(MoveSystem {})));
/// system_manager.update(&event_dispatcher);
/// ```
pub struct SystemManager {
    systems: Vec<Rc<RefCell<dyn System>>>,
    refresh: Vec<RefCell<RefreshPeriod>>,
    names: HashMap<&'static str, usize>,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            refresh: Vec::new(),
            names: HashMap::new(),
        }
    }

    ///
    /// Add a system
    pub fn add_system<S>(&mut self, system: Rc<RefCell<S>>)
    where
        S: System + 'static,
    {
        self.names
            .insert(system.borrow().name(), self.systems.len());
        self.systems.push(system);
        self.refresh.push(RefCell::new(RefreshPeriod::EveryTime));
    }

    ///
    /// Update refresh time for a system
    pub fn set_refresh(&self, name: &str, value: RefreshPeriod) {
        if let Some(id) = self.names.get(&name) {
            self.set_refresh_by_pos(*id, value);
        }
    }

    fn set_refresh_by_pos(&self, id: usize, value: RefreshPeriod) {
        let mut status = self.refresh.get(id).unwrap().borrow_mut();
        *status = value;
    }

    ///
    /// Execute all systems
    pub fn update<EventAdapters>(
        &self,
        event_dispatcher: &Rc<EventDispatcher<EventAdapters>>,
    ) -> RefreshPeriod
    where
        EventAdapters: Default,
    {
        let mut ret = RefreshPeriod::Stop;
        let now = Instant::now();
        for ((id, system), refresh) in self.systems.iter().enumerate().zip(self.refresh.iter()) {
            let refresh = *refresh.borrow();
            ret = max(ret, refresh);
            if RefreshPeriod::At(now) < refresh {
                let mut system = system.borrow_mut();
                let new_refresh = system.run(now);
                if new_refresh != refresh {
                    self.set_refresh_by_pos(id, new_refresh);
                }
                event_dispatcher.dispatch();
            }
        }
        ret
    }
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}
