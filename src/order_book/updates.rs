pub enum Trader {
    Me,
    Other,
}

pub enum Side {
    Buy,
    Sell,
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
