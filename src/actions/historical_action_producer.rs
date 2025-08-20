use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;

use crate::time_keeping::timer::Timer;
use super::actions::Action;

// have this open a file and read it into a buffer
pub struct HistoricalActionProducer {
    timer: Rc<Timer>,
    action_buffer: RefCell<VecDeque<Action>>,
}

impl HistoricalActionProducer {
    pub fn grab_cycle(&self) -> Vec<Action> {
        let mut current_actions = Vec::new();

        // pop actions that will occur before next time step and return them
        while let Some(action) = self.pop_action() {
            current_actions.push(action);
        }
        current_actions
    }

    // only pops actions if it will occur before next time step
    fn pop_action (&self) -> Option<Action> {
        let next_ts = self.timer.peek_next_time();
        let next_action_ts = self.action_buffer.borrow().front()?.get_ts();
        if next_action_ts <= next_ts {
            Some(self.action_buffer.borrow_mut().pop_front()?)   
        } else {
            None
        }
        
    }
}