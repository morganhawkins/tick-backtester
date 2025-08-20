use crate::order_book::updates::Side;

pub enum Action {
    OrderPlace(f64, u8, i32, Side),
    OrderCancel(f64, u8, i32, Side),
}

impl Action {
    pub fn get_ts(&self) -> f64 {
        match self {
            Action::OrderPlace(ts, _, _, _) => *ts,
            Action::OrderCancel(ts, _, _, _) => *ts,
        }
    }
}
