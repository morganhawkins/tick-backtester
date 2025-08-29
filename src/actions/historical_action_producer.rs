use std::iter::Peekable;
use std::error::Error;
use std::rc::Rc;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

use serde::de::DeserializeOwned;

use super::actions::Action;
use crate::time_keeping::timer::Timer;

pub struct BufferedActionRecordReader {
    read_buffer: Peekable<std::io::Lines<BufReader<File>>>
}

impl BufferedActionRecordReader {
    pub fn new(path: String) -> Result<Self, Box<dyn Error>> {
        let file_path = Path::new(&path);
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines: Peekable<Lines<BufReader<File>>>  = reader.lines().into_iter().peekable();
        Ok(Self{
            read_buffer: lines
        })
    }
}

impl Iterator for BufferedActionRecordReader{
    type Item = Action;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

// have this open a file and read it into a buffer
pub struct HistoricalActionProducer {
    timer: Rc<Timer>,
    action_buffer: Peekable<Box<dyn Iterator<Item = Action>>>,
}



impl HistoricalActionProducer{

    pub fn new(timer: &Rc<Timer>, path: &str) -> ()
    {
        

    }

    // pop all actions that will occur before the next time step
    pub fn grab_cycle(&mut self) -> Vec<Action> {
        let mut current_actions = Vec::new();

        // pop actions that will occur before next time step and return them
        while let Some(action) = self.pop_action() {
            current_actions.push(action);
        }
        current_actions
    }

    // only pops actions if it will occur before next time step
    fn pop_action(&mut self) -> Option<Action> {
        let next_ts = self.timer.peek_next_time();
        
        // match  self.action_buffer.peek() {

        // }
        None
    }
}


