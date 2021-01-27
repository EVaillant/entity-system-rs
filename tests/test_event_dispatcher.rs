use entity_system::create_dispatcher;

struct Event1(i32);
struct Event2(i32);
create_dispatcher!(MyDispatcher1 { Event1, Event2 });

use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Receiver1 {
    event1: u32,
    event2: u32,
}

impl Receiver1 {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            event1: 0,
            event2: 0,
        }))
    }
}

impl entity_system::EventHandler<Event1> for Receiver1 {
    fn on_event(&mut self, _event: &Event1) {
        self.event1 += 1;
    }
}

impl entity_system::EventHandler<Event2> for Receiver1 {
    fn on_event(&mut self, _event: &Event2) {
        self.event2 += 1;
    }
}

struct Receiver2 {
    event1: u32,
    event2: u32,
    connection1: entity_system::Connection<MyDispatcher1, Receiver2, Event1>,
    connection2: entity_system::Connection<MyDispatcher1, Receiver2, Event2>,
    dispatcher: Weak<MyDispatcher1>,
}

impl Receiver2 {
    fn new(dispatcher: &Rc<MyDispatcher1>) -> Rc<RefCell<Self>> {
        let instance = Rc::new(RefCell::new(Self {
            event1: 0,
            event2: 0,
            connection1: entity_system::Connection::empty(),
            connection2: entity_system::Connection::empty(),
            dispatcher: Rc::downgrade(dispatcher),
        }));

        let mut ref_instance = instance.borrow_mut();
        ref_instance.connection1 = dispatcher.create_connection(&instance);
        ref_instance.connection2 = dispatcher.create_connection(&instance);
        ref_instance.connection1.connect();
        drop(ref_instance);

        instance
    }
}

impl entity_system::EventHandler<Event1> for Receiver2 {
    fn on_event(&mut self, _event: &Event1) {
        self.event1 += 1;
        self.connection1.disconnect();
        self.connection2.connect();
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.push(Event2 { 0: 0 });
        }
    }
}

impl entity_system::EventHandler<Event2> for Receiver2 {
    fn on_event(&mut self, _event: &Event2) {
        self.event2 += 1;
        self.connection2.disconnect();
    }
}

#[test]
fn test_event_dispatcher_01() {
    let dispatcher = MyDispatcher1::new();
    let receiver = Receiver1::new();
    dispatcher.create_connection::<Receiver1, Event1>(&receiver);
    dispatcher.push(Event1 { 0: 0 });
    dispatcher.dispatch();
    assert_eq!(receiver.borrow().event1, 0);
    assert_eq!(receiver.borrow().event2, 0);
}

#[test]
fn test_event_dispatcher_02() {
    let dispatcher = MyDispatcher1::new();
    let receiver = Receiver1::new();
    let connection = dispatcher.create_connection::<Receiver1, Event1>(&receiver);
    connection.connect();
    dispatcher.push(Event1 { 0: 0 });
    dispatcher.dispatch();
    assert_eq!(receiver.borrow().event1, 1);
    assert_eq!(receiver.borrow().event2, 0);
    connection.disconnect();
    dispatcher.push(Event1 { 0: 0 });
    dispatcher.dispatch();
    assert_eq!(receiver.borrow().event1, 1);
    assert_eq!(receiver.borrow().event2, 0);
}

#[test]
fn test_event_dispatcher_03() {
    let dispatcher = MyDispatcher1::new();
    let receiver = Receiver1::new();
    let connection1 = dispatcher.create_connection::<Receiver1, Event1>(&receiver);
    connection1.connect();
    let connection2 = dispatcher.create_connection::<Receiver1, Event2>(&receiver);
    connection2.connect();
    dispatcher.push(Event2 { 0: 0 });
    dispatcher.push(Event1 { 0: 0 });
    dispatcher.dispatch();
    assert_eq!(receiver.borrow().event1, 1);
    assert_eq!(receiver.borrow().event2, 1);
    connection1.disconnect();
    dispatcher.push(Event2 { 0: 0 });
    dispatcher.push(Event2 { 0: 0 });
    dispatcher.dispatch();
    assert_eq!(receiver.borrow().event1, 1);
    assert_eq!(receiver.borrow().event2, 3);
}

#[test]
fn test_event_dispatcher_04() {
    let dispatcher = MyDispatcher1::new();
    let receiver = Receiver2::new(&dispatcher);
    dispatcher.push(Event1 { 0: 0 });
    dispatcher.dispatch();
    assert_eq!(receiver.borrow().event1, 1);
    assert_eq!(receiver.borrow().event2, 1);
}
