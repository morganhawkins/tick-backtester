use std::cell::RefCell;
use std::rc::Rc;

use super::construction::Order;
use super::updates::{Side, Trader};
use crate::actions::actions::Action;

pub struct OrderBook {
    // Granular vector of orders to preserve time order
    asks: [Rc<RefCell<Vec<Order>>>; 99],
    bids: [Rc<RefCell<Vec<Order>>>; 99],
    // Order amounts aggregated by side and trader
    // Used to quickly find matches
    bid_liquidity: RefCell<[i32; 99]>,
    ask_liquidity: RefCell<[i32; 99]>,
    // best bid ask stored since nearly all updates
    // are order place/cancels
    bid: RefCell<u8>,
    ask: RefCell<u8>,
}

impl std::fmt::Debug for OrderBook {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for (price_idx, ask_level) in self.asks.iter().enumerate().rev() {
            let price = price_idx + 1;
            let mut quantity = 0;
            for order in ask_level.borrow().iter() {
                quantity += *order.quantity.borrow();
            }
            if quantity != 0 {
                println!("{price} - {quantity}");
            }
        }
        println!("-------");
        println!("{}/{}", self.bid.borrow(), self.ask.borrow());
        println!("-------");
        for (price_idx, bid_level) in self.bids.iter().enumerate().rev() {
            let price = price_idx + 1;
            let mut quantity = 0;
            for order in bid_level.borrow().iter() {
                quantity += *order.quantity.borrow();
            }
            if quantity != 0 {
                println!("{price} - {quantity}");
            }
        }

        return Ok(());
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

    fn create_blank_liquidity() -> ([i32; 99], [i32; 99]) {
        let ask_liquidity: [i32; 99] = std::array::from_fn(|_| 0_i32);
        let bid_liquidity: [i32; 99] = std::array::from_fn(|_| 0_i32);
        (ask_liquidity, bid_liquidity)
    }

    /// Create a new orderbook with no placed
    ///
    /// # Returns
    /// An orderbook object with no previous actions
    pub fn new_blank() -> Self {
        let (ask_ladder, bid_ladder) = OrderBook::create_blank_ladders();
        let (ask_liquidity, bid_liquidity) = OrderBook::create_blank_liquidity();

        return Self {
            asks: ask_ladder,
            bids: bid_ladder,
            ask_liquidity: RefCell::new(ask_liquidity),
            bid_liquidity: RefCell::new(bid_liquidity),
            bid: RefCell::new(0),
            ask: RefCell::new(100),
        };
    }

    /// Create an order book from a list of bid and ask (price,quantities) tuples.
    ///
    ///
    /// # Arguments
    /// * asks - vector of (price,quantity) tuples representing liquidity
    /// * bids - vector of (price,quantity) tuples representing liquidity
    /// * trader - the trader who placed all of the current orders
    ///
    /// # Returns
    /// An orderbook with resting limit orders
    pub fn from_snapshot(
        asks: Vec<(u8, i32)>,
        bids: Vec<(u8, i32)>,
        other_trader: &Rc<Trader>,
    ) -> Self {
        let (ask_ladder, bid_ladder) = OrderBook::create_blank_ladders();
        let (mut ask_liquidity, mut bid_liquidity) = OrderBook::create_blank_liquidity();

        // Filling in ask ladder
        let mut best_ask = 100u8;
        for (ask_price, ask_quantity) in asks {
            // push order
            let price_idx = (ask_price - 1) as usize;
            let order = Order::new(other_trader, ask_quantity, Side::Sell, ask_price);
            ask_ladder[price_idx].borrow_mut().push(order);
            // add liquidity
            ask_liquidity[price_idx] += ask_quantity;
            if (ask_price < best_ask) && (ask_quantity > 0) {
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
            bid_liquidity[price_idx] += bid_quantity;
            if (bid_price > best_bid) && (bid_quantity > 0) {
                best_bid = bid_price;
            }
        }

        return Self {
            asks: ask_ladder,
            bids: bid_ladder,
            ask_liquidity: RefCell::new(ask_liquidity),
            bid_liquidity: RefCell::new(bid_liquidity),
            bid: RefCell::new(best_bid),
            ask: RefCell::new(best_ask),
        };
    }

    fn set_bid(&self, value: u8) {
        *self.bid.borrow_mut() = value;
    }

    fn set_ask(&self, value: u8) {
        *self.ask.borrow_mut() = value;
    }

    /// Function to update/validate the best bid/ask after orderbook
    /// subtraction or order matching. Note: must update liquidity array before
    /// calling this method
    ///
    /// # Arguments
    /// * price - price at which the subtraction/match occured
    /// * side - side from the book that needs best price updated
    fn update_best_sub(&self, price: u8, _quantity: i32, side: &Side) {
        let mut best_price = match side {
            Side::Buy => *self.bid.borrow() as isize,
            Side::Sell => *self.ask.borrow() as isize,
        };
        if best_price != (price as isize) {
            return;
        }
        let price_increment = match side {
            Side::Buy => -1isize,
            Side::Sell => 1isize,
        };
        let ladder_ref = match side {
            Side::Buy => &self.bid_liquidity,
            Side::Sell => &self.ask_liquidity,
        };

        while (best_price <= 100) && (best_price >= 0) {
            if (best_price==100) || (best_price==0){
                // if best_price is 100 or 0 i.e. no orders at all
                break
            }
            if ladder_ref.borrow()[(best_price - 1) as usize] != 0 {
                // if non-zero liquidity found, break
                break;
            } else {
                // otherwise go to next best price
                best_price += price_increment;
            }
        }

        // update new best price
        match side {
            Side::Buy => {
                self.set_bid(best_price as u8);
            }
            Side::Sell => {
                self.set_ask(best_price as u8);
            }
        }
    }

    /// Function to update/validate the best bid/ask after orderbook
    /// addition
    ///
    /// # Arguments
    /// * price - the price at which an order was placed
    /// * quantity - quantity added to order book
    /// * side - side from the book that needs best price updated
    fn update_best_add(&self, price: u8, quantity: i32, side: &Side) {
        let best_price = match side {
            Side::Buy => *self.bid.borrow(),
            Side::Sell => *self.ask.borrow(),
        };
        match side {
            Side::Buy => {
                if (price > best_price) && (quantity > 0) {
                    self.set_bid(price);
                }
            }
            Side::Sell => {
                if (price < best_price) && (quantity > 0) {
                    self.set_ask(price);
                }
            }
        };
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

    /// Subtract order quantity from `trader`'s order starting with the FIRST order
    /// until the amount to subtract is satisfied
    /// If quantity to subtract exceeds to total amount at price level placed by trader,
    /// then the remaining un-subtracted amount is discarded
    ///
    ///
    /// # Arguments
    /// * price - price to subtract order quantity from
    /// * quantity - negative int representing amount to subtract
    /// * side - side of the order
    /// * trader - trader performing action
    ///
    fn sub_front(&self, price: u8, quantity: i32, side: Side, trader: Rc<Trader>) {
        if quantity == 0 {
            return ();
        } else if quantity > 0 {
            panic!("sub_front 'quantity' argument should never be positive");
        };
        // quantity to track progress on cancellations
        let mut quant_to_subtract = quantity;
        // iterate through orders at price level in order of oldest -> newest
        for order in self.get_orders(price, &side).borrow_mut().iter_mut() {
            // make sure that trade maker is same as person modifying
            if order.trader.is_same(&trader) {
                let order_quantity_delta = order.delta_quantity(quant_to_subtract);
                quant_to_subtract -= order_quantity_delta;
                if quant_to_subtract == 0 {
                    break;
                }
            }
        }

        let delta_liquidity = quantity - quant_to_subtract;
        // update liquidity array
        self.delta_liquidity(price, delta_liquidity, &side);
        // update best price
        self.update_best_sub(price, 0, &side);
    }

    /// Subtract order quantity from `trader`'s order starting with the LAST order
    /// until the amount to subtract is satisfied
    /// If quantity to subtract exceeds to total amount at price level placed by trader,
    /// then the remaining un-subtracted amount is discarded
    ///
    /// # Arguments
    /// * price - price to subtract order quantity from
    /// * quantity - negative int representing amount to subtract
    /// * side - side of the order
    /// * trader - trader performing action
    ///
    fn sub_back(&self, price: u8, quantity: i32, side: Side, trader: Rc<Trader>) {
        if quantity == 0 {
            return ();
        } else if quantity > 0 {
            panic!("sub_front 'quantity' argument should never be positive");
        };
        // quantity to track progress on cancellations
        let mut quant_to_subtract = quantity;
        // iterate through orders at price level in order of newest -> oldest
        for order in self.get_orders(price, &side).borrow_mut().iter_mut().rev() {
            // make sure that trade maker is same as person modifying
            if order.trader.is_same(&trader) {
                let order_quantity_delta = order.delta_quantity(quant_to_subtract);
                quant_to_subtract -= order_quantity_delta;
                if quant_to_subtract == 0 {
                    break;
                }
            }
        }

        let delta_liquidity = quantity - quant_to_subtract;
        // update liquidity array
        self.delta_liquidity(price, delta_liquidity, &side);
        // update best price
        self.update_best_sub(price, 0, &side);
    }

    // Add order quantity to the back of the order book
    // if most recent order is from same trader, increase the quantity by `quantity`
    // if most recent order is from different trader, push a new order onto the book
    ///
    /// # Arguments
    /// * price - price to add order quantity to
    /// * quantity - amount to add
    /// * side - side of the order
    /// * trader - trader performing action
    ///
    fn add_back(&self, price: u8, quantity: i32, side: Side, trader: Rc<Trader>) {
        if quantity == 0 {
            return ();
        } else if quantity < 0 {
            panic!("sub_front 'quantity' argument should never be positive");
        };
        // check if last order in book is of same trader
        let can_modify = match self.get_orders(price, &side).borrow().last() {
            Some(order) => order.trader.is_same(&trader),
            None => false,
        };
        if can_modify {
            // if we can modify, we know there is a last Order, so we can unwrap
            // and it is the same trader and `trader` arguement
            self.get_orders(price, &side)
                .borrow_mut()
                .last_mut()
                .unwrap()
                .delta_quantity(quantity);
        } else {
            // if we can't modify, we need to create a new Order
            // and push it into price level Vec
            let orders = self.get_orders(price, &side);
            let new_order = Order::new(&trader, quantity, side.clone(), price);
            orders.borrow_mut().push(new_order);
        }

        // update liquidity array
        self.delta_liquidity(price, quantity, &side);
        // update best price
        self.update_best_add(price, quantity, &side);
    }

    fn delta_liquidity(&self, price: u8, quantity: i32, side: &Side) {
        let price_idx = (price - 1) as usize;
        match side {
            Side::Buy => {
                self.bid_liquidity.borrow_mut()[price_idx] += quantity;
            }
            Side::Sell => {
                self.ask_liquidity.borrow_mut()[price_idx] += quantity;
            }
        }
    }

    /// Match a take order with the current order book
    ///  
    /// # Arguments
    /// * take_order - taking order to match with the order book
    fn match_order(&self, take_order: &Order) {
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
        let mut matched;
        while ((best_price * price_increment) <= price_ceil)
            && (best_price <= 99)
            && (best_price >= 1)
        {
            orders = self.get_orders(best_price as u8, &take_order.side.opposite());

            for order in orders.borrow_mut().iter_mut() {
                // if the order's trader is opposite from
                matched = order.fill(&take_order);
                self.delta_liquidity(best_price as u8, -matched, &take_order.side.opposite());

                if *take_order.quantity.borrow() == 0 {
                    break;
                }
            }

            if *take_order.quantity.borrow() == 0 {
                break;
            }

            best_price += price_increment;
        }

        // update best bid/ask if enough orders were taken
        let make_side = &take_order.side.opposite();
        let last_best = match make_side {
            Side::Buy => *self.bid.borrow(),
            Side::Sell => *self.ask.borrow(),
        };
        self.update_best_sub(last_best, 0, make_side);
    }

    /// Digest an order place action
    ///
    /// # Arguments
    /// * price - the price at which to place the limit order
    /// * quant - the quantity to change order by at the price level
    /// * side - buy or sell order
    /// * trader - the trader performing the action
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
    /// # Arguments
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
    ) -> i32 {
        let take_order = Order::new(&trader, quant, side.clone(), price);
        self.match_order(&take_order);
        return quant - *take_order.quantity.borrow();
    }

    ///
    /// # Arguments
    ///
    /// # Returns
    ///
    fn digest_order_cancel(&self, _ts: f64, price: u8, side: Side, trader: Rc<Trader>) {
        let price_idx = (price as usize) - 1;
        let orders = match side {
            Side::Buy => self.bids[price_idx].clone(),
            Side::Sell => self.asks[price_idx].clone(),
        };

        // only retain orders from traders who are not the cancellign trader
        orders
            .borrow_mut()
            .retain(|order| !order.trader.is_same(&trader));
    }

    ///
    /// # Arguments
    ///
    /// # Returns
    ///
    pub fn digest_action(&self, action: Action) {
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_keeping::timer::Timer;
    use crate::actions::action_producer::ActionProducer;

    fn test_entities() -> (Rc<Timer>, Rc<Trader>, Rc<Trader>, ActionProducer, ActionProducer) {
        let timer = Rc::new(Timer::new(1756428105.0, 0.5));
        let me = Rc::new(Trader::new_me());
        let other = Rc::new(Trader::new_other());
        let me_prod = ActionProducer::new(&timer, &me, 0.0);
        let other_prod = ActionProducer::new(&timer, &other, 0.0);
        (timer, me, other, me_prod, other_prod)
    }
    fn double_side_ob(other_trader: &Rc<Trader>) -> OrderBook{
        OrderBook::from_snapshot(vec![(51, 10)], vec![(49, 10)], other_trader)
    }
    fn one_side_ob_buy(other_trader: &Rc<Trader>) -> OrderBook {
        OrderBook::from_snapshot(vec![], vec![(49, 10)], other_trader)
    }
    fn one_side_ob_sell(other_trader: &Rc<Trader>) -> OrderBook {
        OrderBook::from_snapshot(vec![(51, 10)], vec![], other_trader)
    }
    fn test_books(other_trader: &Rc<Trader>) -> (OrderBook, OrderBook, OrderBook){
        (
            one_side_ob_buy(other_trader),
            one_side_ob_sell(other_trader),
            double_side_ob(other_trader),
        )
    }
    fn double_take_orders(me_producer: &ActionProducer, quant: i32) -> (Action, Action) {
        let take_1 = me_producer.trade_take(55, quant, Side::Buy);
        let take_2 = me_producer.trade_take(55, quant, Side::Buy);
        (take_1, take_2)
    }

    /// Test scenario where liquidity is 0 on one side and then a take order action is produced
    #[test]
    fn test_take_empty(){
        let (_, _, other, me_prod, _) = test_entities();
        let (one_side_b, one_side_s, two_side) = test_books(&other); 
        
        // take all orders and attempt to take another
        let(take_1, take_2) = double_take_orders(&me_prod, 10);
        two_side.digest_action(take_1); // take all orders
        two_side.digest_action(take_2); // take when no orders exist
        assert!(*two_side.ask.borrow()==100u8, "ask should be 100 when no sell orders exist");
        
        // take when no orders exist
        let(take_1, _) = double_take_orders(&me_prod, 10);
        one_side_b.digest_action(take_1); // take when no orders exist
        assert!(*one_side_b.ask.borrow()==100u8, "ask should be 100 when no sell orders exist");
        let(take_1, take_2) = double_take_orders(&me_prod, 10);
        one_side_s.digest_action(take_1); // take all orders
        one_side_s.digest_action(take_2); // take when no orders exist
        assert!(*one_side_b.ask.borrow()==100u8, "ask should be 100 when no sell orders exist");
    }

    /// test placing order at different places in book that match or don't match
    #[test]
    fn test_order_place(){
        let (_, _, other, me_prod, other_prod) = test_entities();
        let (one_side_b, one_side_s, two_side) = test_books(&other); 
        
        let act = me_prod.order_place(50, 10, Side::Buy);
        one_side_b.digest_action(act);
        assert!(*one_side_b.bid.borrow()==50u8);
        
        let act = me_prod.order_place(53, 20, Side::Buy);
        one_side_s.digest_action(act);
        assert!(*one_side_s.bid.borrow()==53u8);
        assert!(*one_side_s.ask.borrow()==100u8);
        
        let act = other_prod.order_place(53, 50, Side::Sell);
        two_side.digest_action(act);
        let act = me_prod.order_place(53, 20, Side::Buy);
        two_side.digest_action(act);
        assert!(*two_side.bid.borrow()==49u8);
        assert!(*two_side.ask.borrow()==53u8);
    }
    
    // test matched trade takes and unmatched trade take impact on book
    #[test]
    fn test_trade_take(){
        let (_, _, other, me_prod, other_prod) = test_entities();
        let (one_side_b, one_side_s, two_side) = test_books(&other); 
        
        let act = me_prod.trade_take(99, 100, Side::Buy);
        two_side.digest_action(act);
        assert!(*two_side.bid.borrow()==49u8);
        assert!(*two_side.ask.borrow()==100u8);
        
        let act = other_prod.trade_take(99, 100, Side::Buy);
        one_side_b.digest_action(act);
        assert!(*one_side_b.bid.borrow()==49u8);
        assert!(*one_side_b.ask.borrow()==100u8);
        
        let act = other_prod.trade_take(99, 100, Side::Buy);
        one_side_s.digest_action(act);
        assert!(*one_side_s.bid.borrow()==0u8);
        assert!(*one_side_s.ask.borrow()==100u8);
        
    }

}