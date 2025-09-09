use std::cell::RefCell;

use rand::Rng;

#[derive(Debug)]
pub struct Trader {
    id: u64,
    shares: RefCell<i32>,
    cents: RefCell<i32>,
    me: bool,
}

impl Trader {
    pub fn is_me(&self) -> bool {
        self.me
    }

    pub fn is_other(&self) -> bool {
        !self.me
    }

    pub fn is_same(&self, rhs: &Trader) -> bool {
        self.id==rhs.id
    }

    pub fn new_me() -> Self {
        // there can only be 1 me so id will always be 0
        Self { id: 0, shares: RefCell::new(0), cents: RefCell::new(0), me: true }
    }
    
    pub fn new_other() -> Self {
        // there can be many others so id will always be 0
        let mut rng = rand::rng();
        Self { id: rng.random_range(1..u64::MAX), shares: RefCell::new(0), cents: RefCell::new(0), me: false }
    }

    pub fn delta_shares(&self, shares: i32) {
        *self.shares.borrow_mut() += shares;
    }
    pub fn delta_cents(&self, cents: i32) {
        *self.cents.borrow_mut() += cents;
    }

    pub fn shares(&self) -> i32 {
        *self.shares.borrow()
    }

    pub fn cents(&self) -> i32 {
        *self.cents.borrow()
    }

    
}

#[derive(Debug, Clone)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn is_buy(&self) -> bool {
        match self {
            Self::Buy => true,
            Self::Sell => false,
        }
    }

    pub fn is_sell(&self) -> bool {
        match self {
            Self::Buy => false,
            Self::Sell => true,
        }
    }

    pub fn is_same(&self, rhs: &Side) -> bool {
        if self.is_buy() == rhs.is_buy() {
            return true;
        } else {
            return false;
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

pub enum BookUpdate {
    OrderbookDelta(OrderbookDelta),
    OrderTake(OrderTake),
}

pub struct OrderbookDelta {
    pub trader: Trader, // trader is self or other
    pub side: Side,     // side of the orderbook to modify quantity
    pub price: u8,      // price to modify quantity at
    pub quantity: i32,  // change in order quantity
}

pub struct OrderTake {
    pub taker: Trader,  // trade is self or other
    pub side: Side,     // takers side
    pub best_price: u8, // side=Buy -> highest price taker will buy at, side=Sell -> lowest price taker will sell at
    pub quantitiy: i32, // shares at best price
}
