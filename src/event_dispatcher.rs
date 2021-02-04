use std::cell::RefCell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

pub trait EventHandler<EventType> {
    fn on_event(&mut self, event: &EventType);
}

pub trait Dispatcher<EventAdapters> {
    fn connect<EventHandlerType, EventType>(
        self: &Rc<Self>,
        handler: Rc<RefCell<EventHandlerType>>,
    ) where
        EventHandlerType: EventHandler<EventType> + 'static,
        EventAdapters: AccessEventAdapter<EventType>,
        EventType: 'static;

    fn disconnect<EventHandlerType, EventType>(
        self: &Rc<Self>,
        handler: Rc<RefCell<EventHandlerType>>,
    ) where
        EventHandlerType: EventHandler<EventType> + 'static,
        EventAdapters: AccessEventAdapter<EventType>,
        EventType: 'static;
}

pub trait AccessEventAdapter<EventType> {
    fn get(&self) -> &RefCell<Adapter<EventType>>;
}

impl<DispatcherType, EventAdapters, EventHandlerType, EventType> Default
    for Connection<DispatcherType, EventAdapters, EventHandlerType, EventType>
where
    EventType: 'static,
    EventHandlerType: EventHandler<EventType> + 'static,
    DispatcherType: Dispatcher<EventAdapters>,
    EventAdapters: AccessEventAdapter<EventType>,
{
    fn default() -> Self {
        Self::empty()
    }
}

pub struct Connection<DispatcherType, EventAdapters, EventHandlerType, EventType>
where
    EventType: 'static,
    EventHandlerType: EventHandler<EventType> + 'static,
    DispatcherType: Dispatcher<EventAdapters>,
    EventAdapters: AccessEventAdapter<EventType>,
{
    dispatcher: Weak<DispatcherType>,
    handler: Weak<RefCell<EventHandlerType>>,
    event: PhantomData<EventType>,
    adapters: PhantomData<EventAdapters>,
}

impl<DispatcherType, EventAdapters, EventHandlerType, EventType>
    Connection<DispatcherType, EventAdapters, EventHandlerType, EventType>
where
    EventType: 'static,
    EventHandlerType: EventHandler<EventType> + 'static,
    DispatcherType: Dispatcher<EventAdapters>,
    EventAdapters: AccessEventAdapter<EventType>,
{
    pub fn new(dispatcher: &Rc<DispatcherType>, handler: &Rc<RefCell<EventHandlerType>>) -> Self {
        Self {
            dispatcher: Rc::downgrade(dispatcher),
            handler: Rc::downgrade(handler),
            event: PhantomData,
            adapters: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            dispatcher: Weak::new(),
            handler: Weak::new(),
            event: PhantomData,
            adapters: PhantomData,
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

#[derive(Default)]
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

type EventCallbackType<S> = VecDeque<Box<dyn FnMut(&Rc<S>)>>;

pub struct EventDispatcher<EventAdapters>
where
    EventAdapters: Default,
{
    pendings: RefCell<EventCallbackType<Self>>,
    adapters: EventAdapters,
}

impl<EventAdapters> EventDispatcher<EventAdapters>
where
    EventAdapters: Default,
{
    pub fn new() -> std::rc::Rc<Self> {
        std::rc::Rc::new(Self {
            pendings: RefCell::new(EventCallbackType::new()),
            adapters: Default::default(),
        })
    }

    pub fn create_connection<EventHandlerType, EventType>(
        self: &Rc<Self>,
        handler: &Rc<RefCell<EventHandlerType>>,
    ) -> Connection<Self, EventAdapters, EventHandlerType, EventType>
    where
        EventHandlerType: EventHandler<EventType>,
        EventAdapters: AccessEventAdapter<EventType>,
        EventType: 'static,
    {
        Connection::new(self, handler)
    }

    pub fn push<EventType>(self: &Rc<Self>, event: EventType)
    where
        EventAdapters: AccessEventAdapter<EventType>,
        EventType: 'static,
    {
        self.pendings
            .borrow_mut()
            .push_back(Box::new(move |dispatch| {
                let adapter = (&dispatch.adapters as &dyn AccessEventAdapter<EventType>).get();
                adapter.borrow_mut().invoke(&event);
            }));
    }

    pub fn dispatch(self: &Rc<Self>) {
        while let Some(mut event) = self.pop_event_() {
            (event)(&self);
        }
    }

    fn pop_event_(&self) -> Option<Box<dyn FnMut(&Rc<Self>)>> {
        let mut events = self.pendings.borrow_mut();
        events.pop_front()
    }
}

impl<EventAdapters> Dispatcher<EventAdapters> for EventDispatcher<EventAdapters>
where
    EventAdapters: Default,
{
    fn connect<EventHandlerType, EventType>(self: &Rc<Self>, handler: Rc<RefCell<EventHandlerType>>)
    where
        EventHandlerType: EventHandler<EventType> + 'static,
        EventAdapters: AccessEventAdapter<EventType>,
        EventType: 'static,
    {
        self.pendings
            .borrow_mut()
            .push_back(Box::new(move |dispatch| {
                let adapter = (&dispatch.adapters as &dyn AccessEventAdapter<EventType>).get();
                adapter.borrow_mut().connect(handler.clone());
            }));
    }

    fn disconnect<EventHandlerType, EventType>(
        self: &Rc<Self>,
        handler: Rc<RefCell<EventHandlerType>>,
    ) where
        EventHandlerType: EventHandler<EventType> + 'static,
        EventAdapters: AccessEventAdapter<EventType>,
        EventType: 'static,
    {
        self.pendings
            .borrow_mut()
            .push_back(Box::new(move |dispatch| {
                let adapter = (&dispatch.adapters as &dyn AccessEventAdapter<EventType>).get();
                adapter.borrow_mut().disconnect(handler.clone());
            }));
    }
}

#[macro_export]
macro_rules! create_event_adapters {
    ($name:ident { $($event:ident),* }) => {
        paste::paste! {
            pub struct $name {
                $(
                [<adp $event:snake>] : std::cell::RefCell<entity_system::Adapter<$event>>,
                )*
            }

            impl $name {
                pub fn new() -> Self {
                    Self {
                    $(
                    [<adp $event:snake>] : std::cell::RefCell::new(entity_system::Adapter::new()),
                    )*
                    }
                }
            }

            $(
            impl entity_system::AccessEventAdapter<$event> for $name {
                fn get(&self) -> &std::cell::RefCell<entity_system::Adapter<$event>> {
                    &self.[<adp $event:snake>]
                }
            }
            )*

            impl Default for $name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    };
}
