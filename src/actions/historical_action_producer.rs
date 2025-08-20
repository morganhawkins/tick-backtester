use std::rc::Rc;

use crate::time_keeping::timer::Timer;

pub struct HistoricalActionProducer{
    timer: Rc<Timer>
}