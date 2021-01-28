use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

pub trait EventHandler<EventType> {
    fn on_event(&mut self, event: &EventType);
}

pub trait Dispatcher {
    fn connect<EventHandlerType, EventType>(
        self: &Rc<Self>,
        handler: Rc<RefCell<EventHandlerType>>,
    ) where
        EventHandlerType: EventHandler<EventType> + 'static,
        Self: AccessEventAdapter<EventType>,
        EventType: 'static;

    fn disconnect<EventHandlerType, EventType>(
        self: &Rc<Self>,
        handler: Rc<RefCell<EventHandlerType>>,
    ) where
        EventHandlerType: EventHandler<EventType> + 'static,
        Self: AccessEventAdapter<EventType>,
        EventType: 'static;
}

pub trait AccessEventAdapter<EventType> {
    fn get(&self) -> &RefCell<Adapter<EventType>>;
}

pub struct Connection<DispatcherType, EventHandlerType, EventType>
where
    EventType: 'static,
    EventHandlerType: EventHandler<EventType> + 'static,
    DispatcherType: Dispatcher,
{
    dispatcher: Weak<DispatcherType>,
    handler: Weak<RefCell<EventHandlerType>>,
    event: PhantomData<EventType>,
}

impl<DispatcherType, EventHandlerType, EventType>
    Connection<DispatcherType, EventHandlerType, EventType>
where
    EventType: 'static,
    EventHandlerType: EventHandler<EventType> + 'static,
    DispatcherType: Dispatcher + AccessEventAdapter<EventType>,
{
    pub fn new(dispatcher: &Rc<DispatcherType>, handler: &Rc<RefCell<EventHandlerType>>) -> Self {
        Self {
            dispatcher: Rc::downgrade(dispatcher),
            handler: Rc::downgrade(handler),
            event: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            dispatcher: Weak::new(),
            handler: Weak::new(),
            event: PhantomData,
        }
    }

    pub fn connect(&self) {
        if let (Some(dispatcher), Some(handler)) =
            (self.dispatcher.upgrade(), self.handler.upgrade())
        {
            dispatcher.connect::<EventHandlerType, EventType>(handler);
        }
    }

    pub fn disconnect(&self) {
        if let (Some(dispatcher), Some(handler)) =
            (self.dispatcher.upgrade(), self.handler.upgrade())
        {
            dispatcher.disconnect::<EventHandlerType, EventType>(handler);
        }
    }
}

pub struct Adapter<EventType> {
    handlers: Vec<Rc<RefCell<dyn EventHandler<EventType>>>>,
}

impl<EventType> Adapter<EventType> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn connect(&mut self, handler: Rc<RefCell<dyn EventHandler<EventType>>>) {
        self.handlers.push(handler);
    }

    pub fn disconnect(&mut self, handler: Rc<RefCell<dyn EventHandler<EventType>>>) {
        if let Some(pos) = self
            .handlers
            .iter()
            .position(|x| std::ptr::eq(x.as_ptr() as *const (), handler.as_ptr() as *const ()))
        {
            self.handlers.remove(pos);
        }
    }

    pub fn invoke(&mut self, event: &EventType) {
        for handler in self.handlers.iter() {
            handler.borrow_mut().on_event(event);
        }
    }
}

impl<EventType> Default for Adapter<EventType> {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! create_dispatcher {
    ($name:ident { $($event:ident),* }) => {
        paste::paste! {
            pub struct $name {
                pendings: std::cell::RefCell<std::collections::VecDeque<Box<dyn FnMut(&std::rc::Rc<$name>)>>>,
                    $(
                        [<adp $event:snake>] : std::cell::RefCell<entity_system::Adapter<$event>>,
                    )*
            }

            impl $name {
                pub fn new() -> std::rc::Rc<Self> {
                    std::rc::Rc::new(Self {
                        pendings: std::cell::RefCell::new(std::collections::VecDeque::new()),
                        $(
                            [<adp $event:snake>] : std::cell::RefCell::new(entity_system::Adapter::new()),
                        )*
                    })
                }

                pub fn create_connection<EventHandlerType, EventType>(
                    self: &std::rc::Rc<Self>,
                    handler: &std::rc::Rc<std::cell::RefCell<EventHandlerType>>,
                ) -> entity_system::Connection<Self, EventHandlerType, EventType>
                where
                    EventHandlerType: entity_system::EventHandler<EventType>,
                    Self: entity_system::AccessEventAdapter<EventType>,
                    EventType: 'static,
                {
                    entity_system::Connection::new(self, handler)
                }

                pub fn push<EventType>(self: &std::rc::Rc<Self>, event: EventType)
                where
                    Self: entity_system::AccessEventAdapter<EventType>,
                    EventType: 'static,
                {
                    self.pendings
                        .borrow_mut()
                        .push_back(Box::new(move |dispatch| {
                            let adapter = (&**dispatch as &dyn entity_system::AccessEventAdapter<EventType>).get();
                            adapter.borrow_mut().invoke(&event);
                        }));
                }

                pub fn dispatch(self: &std::rc::Rc<Self>) {
                    while let Some(mut event) = self.pop_event_() {
                        (event)(&self);
                    }
                }

                fn pop_event_(&self) -> Option<Box<dyn FnMut(&std::rc::Rc<Self>)>> {
                    let mut events = self.pendings.borrow_mut();
                    events.pop_front()
                }
            }

            $(
                impl entity_system::AccessEventAdapter<$event> for $name {
                    fn get(&self) -> &std::cell::RefCell<entity_system::Adapter<$event>> {
                        &self.[<adp $event:snake>]
                    }
                }
            )*
        }

        impl entity_system::Dispatcher for $name {
            fn connect<EventHandlerType, EventType>(self: &std::rc::Rc<Self>, handler: std::rc::Rc<std::cell::RefCell<EventHandlerType>>)
            where
                EventHandlerType: entity_system::EventHandler<EventType> + 'static,
                Self: entity_system::AccessEventAdapter<EventType>,
                EventType: 'static,
            {
                self.pendings
                    .borrow_mut()
                    .push_back(Box::new(move |dispatch| {
                        let adapter = (&**dispatch as &dyn entity_system::AccessEventAdapter<EventType>).get();
                        adapter.borrow_mut().connect(handler.clone());
                    }));
            }

            fn disconnect<EventHandlerType, EventType>(
                self: &std::rc::Rc<Self>,
                handler: std::rc::Rc<std::cell::RefCell<EventHandlerType>>,
            ) where
                EventHandlerType: entity_system::EventHandler<EventType> + 'static,
                Self: entity_system::AccessEventAdapter<EventType>,
                EventType: 'static,
            {
                self.pendings
                    .borrow_mut()
                    .push_back(Box::new(move |dispatch| {
                        let adapter = (&**dispatch as &dyn entity_system::AccessEventAdapter<EventType>).get();
                        adapter.borrow_mut().disconnect(handler.clone());
                    }));
            }
        }
    };
}
