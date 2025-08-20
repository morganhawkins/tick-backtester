use std::cell::RefCell;

use super::updates::{Side, Trader};

pub struct Order {
    pub trader: Trader,
    pub quantity: i32,
}

pub struct OrderBook {
    asks: RefCell<[Vec<Order>;99]>,
    bids: RefCell<[Vec<Order>;99]>,
}

impl OrderBook {
    fn create_blank_ladders() -> ([Vec<Order>; 99], [Vec<Order>; 99]) {
        let ask_ladder: [Vec<Order>; 99] = std::array::from_fn(|_| Vec::new());
        let bid_ladder: [Vec<Order>; 99] = std::array::from_fn(|_| Vec::new());
        (ask_ladder, bid_ladder)
    }   

    pub fn new_blank() -> Self{
        let (ask_ladder, bid_ladder) = OrderBook::create_blank_ladders();
        return Self { asks: RefCell::new(ask_ladder), bids: RefCell::new(bid_ladder)}
    }
    
    pub fn from_snapshot(asks: Vec<(u8, i32)>, bids: Vec<(u8, i32)>) -> Self {
        let (mut ask_ladder, mut bid_ladder) = OrderBook::create_blank_ladders();
        
        // TODO: fill in ladders
        return Self { asks: RefCell::new(ask_ladder), bids: RefCell::new(bid_ladder)}
    }

}