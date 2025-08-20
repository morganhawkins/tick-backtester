use std::cell::RefCell;

pub struct Timer {
    time: RefCell<f64>,
    delta: f64, 
}

impl Timer {
    pub fn new(start_time: f64, delta: f64) -> Self {
        Self { time: RefCell::new(start_time), delta: delta  }
    }

    pub fn increment(&self) {
        *self.time.borrow_mut() += self.delta
    }
}