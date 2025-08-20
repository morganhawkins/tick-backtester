use std::cell::RefCell;

pub struct Timer {
    time: RefCell<f64>,
    delta: f64, 
}

impl Timer {
    pub fn new(start_time: f64, delta: f64) -> Self {
        Self { time: RefCell::new(start_time), delta: delta  }
    }

    // increment time to the next time
    pub fn increment(&self) {
        *self.time.borrow_mut() += self.delta
    }

    // peek what the next time step will be
    pub fn peek_next_time(&self) -> f64 {
        *self.time.borrow_mut() + self.delta
    }

    // get the current time step
    pub fn get_time(&self) -> f64 {
        *self.time.borrow_mut()
    }
}