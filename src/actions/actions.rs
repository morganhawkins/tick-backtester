use crate::time_keeping::timer::Timer;

pub enum Action {
    OrderPlace(),
    OrderCancel(),
}

pub struct ActionProducer {
    timer: Timer,
}