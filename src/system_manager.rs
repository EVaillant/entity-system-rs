use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq)]
pub enum RefreshPeriod {
    EveryTime,
    At(Instant),
    Stop,
}

pub trait System {
    fn name(&self) -> &'static str;
    fn run(&mut self) -> RefreshPeriod;
}

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

    pub fn add_system<S>(&mut self, system: Rc<RefCell<S>>)
    where
        S: System + 'static,
    {
        self.names
            .insert(system.borrow().name(), self.systems.len());
        self.systems.push(system);
        self.refresh.push(RefCell::new(RefreshPeriod::EveryTime));
    }

    pub fn set_refresh(&self, name: &str, value: RefreshPeriod) {
        if let Some(id) = self.names.get(&name) {
            self.set_refresh_by_pos(*id, value);
        }
    }

    fn set_refresh_by_pos(&self, id: usize, value: RefreshPeriod) {
        let mut status = self.refresh.get(id).unwrap().borrow_mut();
        *status = value;
    }

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
