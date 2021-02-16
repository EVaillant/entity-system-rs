use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

///
/// Definie the system execution period.
#[derive(Clone, Copy, PartialEq)]
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

///
/// Abstract system type.
///
/// It will be executed by [`SystemManager`]
/// # Example
/// ```rust
/// use entity_system::{System, RefreshPeriod};
///
/// struct MoveSystem {
/// }
///
/// impl System for MoveSystem {
///     fn name(&self) -> &'static str {
///         "move"
///     }
///
///     fn run(&mut self) -> RefreshPeriod {
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
    /// # Return
    ///
    /// The next execution time.
    fn run(&mut self) -> RefreshPeriod;
}

///
/// Manage & Execute [`System`]\(s)
///
/// # Example
/// ```rust
/// use std::cell::RefCell;
/// use std::rc::Rc;
/// use entity_system::{SystemManager, System, RefreshPeriod};
///
/// struct MoveSystem {
/// }
///
/// impl System for MoveSystem {
///     fn name(&self) -> &'static str {
///         "move"
///     }
///
///     fn run(&mut self) -> RefreshPeriod {
///         //
///         // Do lot of thing
///         //
///
///         RefreshPeriod::EveryTime
///     }         
/// }
///
/// let mut system_manager = SystemManager::new();
/// system_manager.add_system(Rc::new(RefCell::new(MoveSystem {})));
/// system_manager.update();
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
    pub fn update(&self) {
        let now = Instant::now();
        for ((id, system), refresh) in self.systems.iter().enumerate().zip(self.refresh.iter()) {
            let refresh = *refresh.borrow();
            match refresh {
                RefreshPeriod::At(time) => {
                    if now < time {
                        continue;
                    }
                }
                RefreshPeriod::EveryTime => {}
                RefreshPeriod::Stop => {
                    continue;
                }
            }
            let mut system = system.borrow_mut();
            let new_refresh = system.run();
            if new_refresh != refresh {
                self.set_refresh_by_pos(id, new_refresh);
            }
        }
    }
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}
