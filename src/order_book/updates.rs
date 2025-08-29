pub enum Trader {
    Me,
    Other,
}

impl Trader {
    pub fn is_me(&self) -> bool {
        match self {
            Self::Me => true,
            Self::Other => false,
        }
    }

    pub fn is_other(&self) -> bool {
        match self {
            Self::Me => false,
            Self::Other => true,
        }
    }

    pub fn is_same(&self, rhs: &Trader) -> bool {
        if self.is_me() == rhs.is_me() {
            return true;
        } else {
            return false;
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Me => Self::Other,
            Self::Other => Self::Me,
        }
    }
}

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
