use std::rc::Rc;

use crate::order_book::updates::Side;
use crate::time_keeping::timer::Timer;
use super::actions::Action;

pub struct ActionProducer {
    timer: Rc<Timer>,
}

impl ActionProducer {
    pub fn new(timer: &Rc<Timer>) -> Self {
        Self{timer: Rc::clone(timer)}
    }

    pub fn order_place(&self, price: u8, quantity: i32, side: Side) -> Action {
        let ts= self.timer.get_time();
        Action::OrderPlace(ts, price, quantity, side)
    }

    pub fn order_cancel(&self, price: u8, quantity: i32, side: Side) -> Action {
        let ts= self.timer.get_time();
        Action::OrderCancel(ts, price, quantity, side)
    }
}