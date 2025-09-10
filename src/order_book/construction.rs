use std::cell::RefCell;
use std::rc::Rc;

use super::updates::{Side, Trader};

pub struct Order {
    pub trader: Rc<Trader>,
    pub quantity: RefCell<i32>,
    pub side: Side,
    pub price: u8,
}

pub struct BookSnapshot {
    pub asks: Vec<(u8, i32)>,
    pub bids: Vec<(u8, i32)>,
}

impl Order {
    pub fn new(trader: &Rc<Trader>, quantity: i32, side: Side, price: u8) -> Self {
        Order {
            trader: trader.clone(),
            quantity: RefCell::new(quantity),
            side,
            price,
        }
    }

    /// Match two orders. Will
    ///
    /// # Arguments
    ///
    /// # Returns
    /// the number of shares that were successfully matched
    pub fn fill(&self, order: &Order) -> i32 {
        let matched_amount = order.quantity.borrow().min(*self.quantity.borrow());
        let matched_cents = matched_amount * (self.price as i32);

        if matched_amount == 0 {
            return 0;
        }

        match self.side {
            Side::Buy => {
                // add shares, subtract money
                self.trader.delta_shares(matched_amount);
                order.trader.delta_shares(-matched_amount);
                self.trader.delta_cents(-matched_cents);
                order.trader.delta_cents(matched_cents);
            }
            Side::Sell => {
                // subtract shares, add money
                self.trader.delta_shares(-matched_amount);
                order.trader.delta_shares(matched_amount);
                self.trader.delta_cents(matched_cents);
                order.trader.delta_cents(-matched_cents);
            }
        }
        *self.quantity.borrow_mut() -= matched_amount;
        *order.quantity.borrow_mut() -= matched_amount;
        matched_amount
    }
}
