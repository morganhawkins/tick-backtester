use std::cell::RefCell;
use std::error::Error;

use super::construction::{BookSnapshot, Order};
use super::updates::{BookUpdate, Side, Trader};
use crate::actions::actions::Action;

pub struct OrderBook {
    // Granular vector of orders to preserve time order
    asks: RefCell<[Vec<Order>; 99]>,
    bids: RefCell<[Vec<Order>; 99]>,
    // Order amounts aggregated by side and trader
    // Used to quickly find matches
    me_ask_liquidity: RefCell<[i32; 99]>,
    me_bid_liquidity: RefCell<[i32; 99]>,
    other_ask_liquidity: RefCell<[i32; 99]>,
    other_bid_liquidity: RefCell<[i32; 99]>,
}

impl OrderBook {
    fn create_blank_ladders() -> ([Vec<Order>; 99], [Vec<Order>; 99]) {
        let ask_ladder: [Vec<Order>; 99] = std::array::from_fn(|_| Vec::new());
        let bid_ladder: [Vec<Order>; 99] = std::array::from_fn(|_| Vec::new());
        (ask_ladder, bid_ladder)
    }

    fn create_blank_liquidity() -> ([i32; 99], [i32; 99], [i32; 99], [i32; 99]) {
        let me_ask_liquidity: [i32; 99] = std::array::from_fn(|_| 0_i32);
        let me_bid_liquidity: [i32; 99] = std::array::from_fn(|_| 0_i32);
        let other_ask_liquidity: [i32; 99] = std::array::from_fn(|_| 0_i32);
        let other_bid_liquidity: [i32; 99] = std::array::from_fn(|_| 0_i32);
        (
            me_ask_liquidity,
            me_bid_liquidity,
            other_ask_liquidity,
            other_bid_liquidity,
        )
    }

    pub fn new_blank() -> Self {
        let (ask_ladder, bid_ladder) = OrderBook::create_blank_ladders();
        let (
            me_ask_liquidity, 
            me_bid_liquidity, 
            other_ask_liquidity, 
            other_bid_liquidity
        ) = OrderBook::create_blank_liquidity();

        return Self {
            asks: RefCell::new(ask_ladder),
            bids: RefCell::new(bid_ladder),
            me_ask_liquidity: RefCell::new(me_ask_liquidity),
            me_bid_liquidity: RefCell::new(me_bid_liquidity),
            other_ask_liquidity: RefCell::new(other_ask_liquidity),
            other_bid_liquidity: RefCell::new(other_bid_liquidity),
        };
    }

    pub fn from_snapshot(asks: Vec<(u8, i32)>, bids: Vec<(u8, i32)>) -> Self {
        let (mut ask_ladder, mut bid_ladder) = OrderBook::create_blank_ladders();
        let (
            me_ask_liquidity,
            me_bid_liquidity,
            mut other_ask_liquidity,
            mut other_bid_liquidity,
        ) = OrderBook::create_blank_liquidity();

        // Filling in ask ladder
        for (ask_price, ask_quantity) in asks {
            // push order
            let price_idx = (ask_price - 1) as usize;
            let order = Order::new(Trader::Other, ask_quantity);
            ask_ladder[price_idx].push(order);
            // add liquidity
            other_ask_liquidity[price_idx] += ask_quantity;
        }

        // Filling in bid ladder
        for (bid_price, bid_quantity) in bids {
            // push order
            let price_idx = (bid_price - 1) as usize;
            let order = Order::new(Trader::Other, bid_quantity);
            bid_ladder[price_idx].push(order);
            // add liquidity
            other_bid_liquidity[price_idx] += bid_quantity;
        }

        return Self {
            asks: RefCell::new(ask_ladder),
            bids: RefCell::new(bid_ladder),
            me_ask_liquidity: RefCell::new(me_ask_liquidity),
            me_bid_liquidity: RefCell::new(me_bid_liquidity),
            other_ask_liquidity: RefCell::new(other_ask_liquidity),
            other_bid_liquidity: RefCell::new(other_bid_liquidity),
        };
    }

    // Create `BookUpdate`s from actions and book state
    // Does not process updates, just produces
    fn update_from_order_place(
        &self,
        ts: f64,
        price: u8,
        quant: i32,
        side: Side,
    ) -> Vec<BookUpdate> {
        let mut updates: Vec<BookUpdate> = Vec::new();
        
        // creating order take actions
        // decrease quaniity by matches amount and create ordebook delta

        updates
    }

    fn update_from_action(&self, action: Action) -> BookUpdate {
        match action {
            Action::OrderPlace(ts, price, quant, side, trader) => {

            },
            Action::TradeTake(ts, price, quant, side, trader) => {

            },
            Action::OrderCancel(ts, price, side, trader) => {

            },
        };
    }
}
