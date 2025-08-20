use super::updates::{Side, Trader};

pub struct Order {
    pub trader: Trader,
    pub quantity: i32,
}

pub struct OrderBook {
    asks: [Vec<Order>;99],
    bids: [Vec<Order>;99],
}