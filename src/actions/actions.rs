use crate::order_book::updates::{Side, Trader};

pub enum Action {
    // Change your order quantity at a price level
    // This can be seen as an orderbook_delta
    OrderPlace(f64, u8, i32, Side, Trader),
    // Cancel all orders at a price level
    // This action will only really be performed by a strategy
    // This can be seen as placing an order in the negative amount of the
    // total order amount at a price level
    OrderCancel(f64, u8, Side, Trader),
    // This will be performed by the user or the simulation
    // This the same as placing an order except unmatched amount will
    // not go into book
    // This can be seen as an immediate-or-cancel order
    TradeTake(f64, u8, i32, Side, Trader),
}

impl Action {
    pub fn get_ts(&self) -> f64 {
        match self {
            Action::OrderPlace(ts, _, _, _, _) => *ts,
            Action::OrderCancel(ts, _, _, _) => *ts,
            Action::TradeTake(ts, _, _, _, _) => *ts,
        }
    }
}
