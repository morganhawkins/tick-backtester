use std::rc::Rc;

use super::actions::Action;
use crate::order_book::updates::{Side, Trader};
use crate::time_keeping::timer::Timer;

pub struct ActionProducer {
    timer: Rc<Timer>,      // used to track simulation time
    latency_constant: f64, // represents the client -> matching engine message send latency
}

impl ActionProducer {
    pub fn new(timer: &Rc<Timer>, latency_constant: f64) -> Self {
        Self {
            timer: Rc::clone(timer),
            latency_constant: latency_constant,
        }
    }

    pub fn order_place(&self, price: u8, quantity: i32, side: Side) -> Action {
        let ts = self.timer.get_time() + self.latency_constant;
        Action::OrderPlace(ts, price, quantity, side, Trader::Me)
    }

    pub fn order_cancel(&self, price: u8, side: Side) -> Action {
        let ts = self.timer.get_time() + self.latency_constant;
        Action::OrderCancel(ts, price, side, Trader::Me)
    }

    pub fn trade_take(&self, price: u8, quantity: i32, side: Side) -> Action {
        let ts = self.timer.get_time() + self.latency_constant;
        Action::TradeTake(ts, price, quantity, side, Trader::Me)
    }
}
