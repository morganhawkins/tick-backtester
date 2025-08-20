use super::updates::{Side, Trader};

pub struct Order {
    pub trader: Trader,
    pub quantity: i32,
}

pub struct BookSnapshot {
    pub asks: Vec<(u8, i32)>,
    pub bids: Vec<(u8, i32)>,
}

impl Order {
    pub fn new(trader: Trader, quantity: i32) -> Self {
        Order { trader, quantity }
    }
}
