use std::cell::RefCell;
use std::rc::Rc;

use super::construction::{BookSnapshot, Order};
use super::updates::{Side, Trader};
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
    // best bid ask stored since nearly all updates 
    // are order place/cancels
    bid: RefCell<u8>,
    ask: RefCell<u8>,

}

impl std::fmt::Debug for OrderBook {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for (price_idx,ask_level) in self.asks.iter().enumerate().rev() {
            let price = price_idx+1;
            let mut quantity = 0;
            for order in ask_level.borrow().iter() {
                quantity += *order.quantity.borrow();
            }
            if quantity != 0{
                println!("{price} - {quantity}");
            }
        }
        println!("-------");
        for (price_idx,bid_level) in self.bids.iter().enumerate().rev() {
            let price = price_idx+1;
            let mut quantity = 0;
            for order in bid_level.borrow().iter() {
                quantity += *order.quantity.borrow();
            }
            if quantity != 0{
                println!("{price} - {quantity}");
            }
        }

        return Ok(())
    }
}

impl OrderBook {
    fn create_blank_ladders() -> ([Rc<RefCell<Vec<Order>>>; 99], [Rc<RefCell<Vec<Order>>>; 99]) {
        let ask_ladder: [Rc<RefCell<Vec<Order>>>; 99] =
            std::array::from_fn(|_| Rc::new(RefCell::new(Vec::new())));
        let bid_ladder: [Rc<RefCell<Vec<Order>>>; 99] =
            std::array::from_fn(|_| Rc::new(RefCell::new(Vec::new())));
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
        let (me_ask_liquidity, me_bid_liquidity, other_ask_liquidity, other_bid_liquidity) =
            OrderBook::create_blank_liquidity();

        return Self {
            asks: ask_ladder,
            bids: bid_ladder,
            me_ask_liquidity: RefCell::new(me_ask_liquidity),
            me_bid_liquidity: RefCell::new(me_bid_liquidity),
            other_ask_liquidity: RefCell::new(other_ask_liquidity),
            other_bid_liquidity: RefCell::new(other_bid_liquidity),
            bid: RefCell::new(0),
            ask: RefCell::new(100),
        }

    }

    pub fn from_snapshot(asks: Vec<(u8, i32)>, bids: Vec<(u8, i32)>, other_trader: &Rc<Trader>) -> Self {
        let (ask_ladder, bid_ladder) = OrderBook::create_blank_ladders();
        let (me_ask_liquidity, me_bid_liquidity, mut other_ask_liquidity, mut other_bid_liquidity) =
            OrderBook::create_blank_liquidity();

        // Filling in ask ladder
        let mut best_ask = 100u8;
        for (ask_price, ask_quantity) in asks {
            // push order
            let price_idx = (ask_price - 1) as usize;
            let order = Order::new(other_trader, ask_quantity, Side::Sell, ask_price);
            ask_ladder[price_idx].borrow_mut().push(order);
            // add liquidity
            other_ask_liquidity[price_idx] += ask_quantity;
            if (ask_price < best_ask) && (ask_quantity > 0){
                best_ask = ask_price;
            }
        }
        
        // Filling in bid ladder
        let mut best_bid = 0u8;
        for (bid_price, bid_quantity) in bids {
            // push order
            let price_idx = (bid_price - 1) as usize;
            let order = Order::new(other_trader, bid_quantity, Side::Buy, bid_price);
            bid_ladder[price_idx].borrow_mut().push(order);
            // add liquidity
            other_bid_liquidity[price_idx] += bid_quantity;
            if (bid_price > best_bid) && (bid_quantity > 0){
                best_bid = bid_price;
            }
        }

        return Self {
            asks: ask_ladder,
            bids: bid_ladder,
            me_ask_liquidity: RefCell::new(me_ask_liquidity),
            me_bid_liquidity: RefCell::new(me_bid_liquidity),
            other_ask_liquidity: RefCell::new(other_ask_liquidity),
            other_bid_liquidity: RefCell::new(other_bid_liquidity),
            bid: RefCell::new(best_bid),
            ask: RefCell::new(best_ask),
        }
    }

    fn get_orders(&self, price: u8, side: &Side) -> Rc<RefCell<Vec<Order>>> {
        let price_idx = (price - 1u8) as usize;
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
    fn sub_front(&self, price: u8, quantity: i32, side: Side, trader: Rc<Trader>) {
        // quantity to track progress on cancellations
        let mut quant_to_subtract = quantity;
        // iterate through orders at price level in order of oldest -> newest
        for order in self.get_orders(price, &side).borrow_mut().iter_mut() {
            // make sure that trade maker is same as person modifying
            if order.trader.is_same(&trader) {
                if quant_to_subtract > *order.quantity.borrow() {
                    // if we still need to subtract more quantity
                    // subtract order quant from quant left to subtract
                    quant_to_subtract -= *order.quantity.borrow();
                    // 0-out order quantity
                    *order.quantity.borrow_mut() = 0_i32;
                } else {
                    // if our subtraction is satisfied on this order
                    *order.quantity.borrow_mut() -= quant_to_subtract;
                    break;
                }
            }
        }
    }

    // Subtract order quantity from `trader`'s order starting with the LAST order
    // until the amount to subtract is satisfied
    // If quantity to subtract exceeds to total amount at price level placed by trader,
    // then the remaining un-subtracted amount is discarded
    fn sub_back(&self, price: u8, quantity: i32, side: Side, trader: Rc<Trader>) {
        // quantity to track progress on cancellations
        let mut quant_to_subtract = quantity;
        // iterate through orders at price level in order of newest -> oldest
        for order in self.get_orders(price, &side).borrow_mut().iter_mut().rev() {
            // make sure that trade maker is same as person modifying
            if order.trader.is_same(&trader) {
                if quant_to_subtract > *order.quantity.borrow() {
                    // if we still need to subtract more quantity
                    // subtract order quant from quant left to subtract
                    quant_to_subtract -= *order.quantity.borrow();
                    // 0-out order quantity
                    *order.quantity.borrow_mut() = 0_i32;
                } else {
                    // if our subtraction is satisfied on this order
                    *order.quantity.borrow_mut() -= quant_to_subtract;
                    break;
                }
            }
        }
    }

    // Add order quantity to the back of the order book
    // if most recent order is from same trader, increase the quantity by `quantity`
    // if most recent order is from different trader, push a new order onto the book
    fn add_back(&self, price: u8, quantity: i32, side: Side, trader: Rc<Trader>) {
        let can_modify = match self.get_orders(price, &side).borrow().last() {
            Some(order) => order.trader.is_same(&trader),
            None => false,
        };
        if can_modify {
            // if we can modify, we know there is a last Order, so we can unwrap
            // and it is the same trader and `trader` arguement
            *self.get_orders(price, &side)
                .borrow_mut()
                .last_mut()
                .unwrap()
                .quantity
                .borrow_mut() += quantity;
        } else {
            // if we can't modify, we need to create a new Order
            // and push it into price level Vec
            let orders = self.get_orders(price, &side);
            let new_order = Order::new(&trader, quantity, side, price);
            orders.borrow_mut().push(new_order);
        }
    }

    ///
    fn match_order(&self, take_order: &Order){
        let price_increment = match take_order.side {
            Side::Buy => 1,
            Side::Sell => -1,
        };

        let mut best_price = match take_order.side {
            Side::Buy => *self.ask.borrow() as i32,
            Side::Sell => *self.bid.borrow() as i32,
        };
        // worst price taker will accept
        let price_ceil = (take_order.price as i32) * price_increment;

        // while we have shares left to match and the price
        let mut orders;
        while ((best_price*price_increment) <= price_ceil) && (best_price <= 99) && (best_price >= 1){
            orders = self.get_orders(best_price as u8, &take_order.side.opposite());

            for order in orders.borrow_mut().iter_mut() {
                // if the order's trader is opposite from
                order.fill(&take_order);

                if *take_order.quantity.borrow() == 0{
                    break
                }
            }
            
            if *take_order.quantity.borrow() == 0{
                break
            }

            best_price += price_increment;
        }
    }

    /// Create `BookUpdate`s from actions and book state
    /// Does not process updates, just produces
    ///
    ///
    /// # Arguments 
    /// 
    /// 
    /// # Returns
    /// The number of shares that were matched with another trade
    fn digest_order_place(
        &self,
        _ts: f64,
        price: u8,
        quant: i32,
        side: Side,
        trader: Rc<Trader>,
    ) -> i32 {
        if quant > 0 {
            let take_order = Order::new(&trader, quant, side.clone(), price);
            self.match_order(&take_order);
            self.add_back(price, *take_order.quantity.borrow(), side, trader);
        } else if quant < 0 {
            if trader.is_me() {
                self.sub_back(price, quant, side, trader);
            } else {
                self.sub_front(price, quant, side, trader);
            }
        }

        0i32
    }



    ///
    ///
    /// # Arguments 
    /// 
    /// 
    /// # Returns
    /// The number of shares that were matched successfully
    fn digest_trade_take(
        &self,
        _ts: f64,
        price: u8,
        quant: i32,
        side: Side,
        trader: Rc<Trader>,
    ) -> i32{
        let take_order = Order::new(&trader, quant, side.clone(), price);
        self.match_order(&take_order);
        return quant - *take_order.quantity.borrow();
    }
    
    ///
    ///
    /// # Arguments 
    /// 
    /// 
    /// # Returns
    /// Unit
    fn digest_order_cancel(&self, _ts: f64, price: u8, side: Side, trader: Rc<Trader>){
        let price_idx = (price as usize) - 1;
        let orders = match side {
            Side::Buy =>  self.bids[price_idx].clone(),
            Side::Sell => self.asks[price_idx].clone(),
        };
        
        // only retain orders from traders who are not the cancellign trader
        orders.borrow_mut().retain(|order| !order.trader.is_same(&trader));
        
    }
    
    // fn update_from_action(&self, action: Action) -> BookUpdate {
    ///
    ///
    /// # Arguments 
    /// 
    /// 
    /// # Returns
    /// 
    pub fn digest_action(&self, action: Action) -> () {
        match action {
            Action::OrderPlace(ts, price, quant, side, trader) => {
                self.digest_order_place(ts, price, quant, side, trader);
            }
            Action::TradeTake(ts, price, quant, side, trader) => {
                self.digest_trade_take(ts, price, quant, side, trader);
            }
            Action::OrderCancel(ts, price, side, trader) => {
                self.digest_order_cancel(ts, price, side, trader);
            }
        }
    }
}

