use std::cell::RefCell;
use std::rc::Rc;
use std::error::Error;

use super::construction::{BookSnapshot, Order};
use super::updates::{BookUpdate, Side, Trader};
use crate::actions::actions::Action;

pub struct OrderBook {
    // Granular vector of orders to preserve time order
    asks: [Rc<RefCell<Vec<Order>>>; 99],
    bids: [Rc<RefCell<Vec<Order>>>; 99],
    // Order amounts aggregated by side and trader
    // Used to quickly find matches
    me_ask_liquidity: RefCell<[i32; 99]>,
    me_bid_liquidity: RefCell<[i32; 99]>,
    other_ask_liquidity: RefCell<[i32; 99]>,
    other_bid_liquidity: RefCell<[i32; 99]>,
}

impl OrderBook {
    fn create_blank_ladders() -> ([Rc<RefCell<Vec<Order>>>; 99], [Rc<RefCell<Vec<Order>>>; 99]) {
        let ask_ladder: [Rc<RefCell<Vec<Order>>>; 99] = std::array::from_fn(|_| Rc::new(RefCell::new(Vec::new())));
        let bid_ladder: [Rc<RefCell<Vec<Order>>>; 99] = std::array::from_fn(|_| Rc::new(RefCell::new(Vec::new())));
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
            asks: ask_ladder,
            bids: bid_ladder,
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
            ask_ladder[price_idx].borrow_mut().push(order);
            // add liquidity
            other_ask_liquidity[price_idx] += ask_quantity;
        }

        // Filling in bid ladder
        for (bid_price, bid_quantity) in bids {
            // push order
            let price_idx = (bid_price - 1) as usize;
            let order = Order::new(Trader::Other, bid_quantity);
            bid_ladder[price_idx].borrow_mut().push(order);
            // add liquidity
            other_bid_liquidity[price_idx] += bid_quantity;
        }

        return Self {
            asks: ask_ladder,
            bids: bid_ladder,
            me_ask_liquidity: RefCell::new(me_ask_liquidity),
            me_bid_liquidity: RefCell::new(me_bid_liquidity),
            other_ask_liquidity: RefCell::new(other_ask_liquidity),
            other_bid_liquidity: RefCell::new(other_bid_liquidity),
        };
    }

    fn get_orders(&self, price:u8, side: &Side) -> Rc<RefCell<Vec<Order>>> {
        let price_idx = (price-1u8) as usize;
        // selecting relevant side of orderbook
        let ladder = match side {
            Side::Buy => Rc::clone(&self.bids[price_idx]),
            Side::Sell => Rc::clone(&self.asks[price_idx]),
        };
        ladder
    }

    // Subtract order quantity from `trader`'s order starting with the FIRST order
    // until the amount to subtract is satisfied
    // If quantity to subtract exceeds to total amount at price level placed by trader,
    // then the remaining un-subtracted amount is discarded
    fn sub_front(&self, price: u8, quantity: i32, side: Side, trader: Trader) {
        // quantity to track progress on cancellations
        let mut quant_to_subtract = quantity;
        // iterate through orders at price level in order of oldest -> newest
        for order in self.get_orders(price, &side).borrow_mut().iter_mut(){
            // make sure that trade maker is same as person modifying
            if order.trader.is_same(&trader) {
                if quant_to_subtract > order.quantity {
                    // if we still need to subtract more quantity
                    // subtract order quant from quant left to subtract
                    quant_to_subtract -= order.quantity;
                    // 0-out order quantity
                    order.quantity = 0_i32;
                    
                } else {
                    // if our subtraction is satisfied on this order
                    order.quantity -= quant_to_subtract;
                    break
                    
                }
            }
        };
    }
    
    // Subtract order quantity from `trader`'s order starting with the LAST order
    // until the amount to subtract is satisfied
    // If quantity to subtract exceeds to total amount at price level placed by trader,
    // then the remaining un-subtracted amount is discarded
    fn sub_back(&self, price: u8, quantity: i32, side: Side, trader: Trader) {
        // quantity to track progress on cancellations
        let mut quant_to_subtract = quantity;
        // iterate through orders at price level in order of newest -> oldest
        for order in self.get_orders(price, &side).borrow_mut().iter_mut().rev(){
            // make sure that trade maker is same as person modifying
            if order.trader.is_same(&trader) {
                if quant_to_subtract > order.quantity {
                    // if we still need to subtract more quantity
                    // subtract order quant from quant left to subtract
                    quant_to_subtract -= order.quantity;
                    // 0-out order quantity
                    order.quantity = 0_i32;
                    
                } else {
                    // if our subtraction is satisfied on this order
                    order.quantity -= quant_to_subtract;
                    break
                    
                }
            }
        };
    }
    
    // Add order quantity to the back of the order book
    // if most recent order is from same trader, increase the quantity by `quantity`
    // if most recent order is from different trader, push a new order onto the book
    fn add_back(&self, price: u8, quantity: i32, side: Side, trader: Trader) {
        let can_modify =  match self.get_orders(price, &side).borrow().last() {
            Some(order) => order.trader.is_same(&trader),
            None => false
        };
        if can_modify {
            // if we can modify, we know there is a last Order
            // and it is the same trader and `trader` arguement
            self.get_orders(price, &side).borrow_mut().last_mut().unwrap().quantity += quantity;
        } else {
            // if we can't modify, we need to create a new Order
            // and push it into price level Vec
            let new_order = Order::new(trader, quantity);
            self.get_orders(price, &side).borrow_mut().push(new_order);
        }
        
    }

    // NOTE: there is no `Self::add_front` method because no one can skip when
    // placing orer

    fn digest_update(&self, update: BookUpdate) {
        match update {
            BookUpdate::OrderbookDelta(delta) => {

            },
            BookUpdate::OrderTake(take) => {

            },
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

    fn update_from_order_cancel(
        &self,
        ts: f64,
        price: u8,
        side: Side,
    ) -> Vec<BookUpdate> {
        let mut updates: Vec<BookUpdate> = Vec::new();

        updates
    }

    fn update_from_trade_take(
        &self,
        ts: f64,
        price: u8,
        quant: i32,
        side: Side,
    ) -> Vec<BookUpdate> {
        let mut updates: Vec<BookUpdate> = Vec::new();

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
