use std::iter::Peekable;
use std::error::Error;
use std::rc::Rc;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;

use super::actions::Action;
use crate::time_keeping::timer::Timer;
use crate::order_book::updates::{Side, Trader};

#[derive(Deserialize, Debug)]
pub struct Event {
    pub r#type: String,
    pub seq: u32,
    pub quantity: i32,
    pub ts: f64,
    pub side: String,
    pub price: u8,
}

#[derive(Deserialize, Debug)]
pub struct Init {
    pub r#type: String,
    pub buy: Vec<(u8, i32)>,
    pub sell: Vec<(u8, i32)>,

}

#[derive(Debug)]
pub enum HistoricalAction{
    Init(Init),
    Event(Event),
}


impl HistoricalAction {
    pub fn is_init(&self) -> bool {
        match self {
            HistoricalAction::Init(_) => true,
            HistoricalAction::Event(_) => false,
        }
    }

    pub fn is_event(&self) -> bool {
        match self {
            HistoricalAction::Init(_) => false,
            HistoricalAction::Event(_) => true,
        }
    }

    fn into_action(self, trader: &Rc<Trader>) -> Result<Action, String> {
        match self {
            HistoricalAction::Event(event) => {
                let side = match event.side.as_str(){
                    "buy" => Side::Buy,
                    "sell" => Side::Sell,
                    _ => return Err(String::from("side not recognized while performing HistricalAction into Action"))
                };
                if event.r#type == "trade" {
                    // is trade take
                    return Ok(Action::TradeTake(event.ts, event.price, event.quantity, side, trader.clone()))
                } else {
                    // is orderbook delta
                    return Ok(Action::OrderPlace(event.ts, event.price, event.quantity, side, trader.clone()))
                }
            },
            HistoricalAction::Init(_) => {
                Err(String::from("cannot turn an orderbook snapshot into an action"))
            }
        }
    }
}

struct BufferedActionRecordReader {
    read_buffer: Peekable<std::io::Lines<BufReader<File>>>
}

impl BufferedActionRecordReader {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let file_path = Path::new(&path);
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines = reader.lines().into_iter().peekable();
        Ok(Self{
            read_buffer: lines
        })
    }

    fn de(line: &str) -> Option<HistoricalAction> {
        let hist_action = match serde_json::from_str::<Event>(&line) {
            Ok(event) => HistoricalAction::Event(event),
            Err(_) => HistoricalAction::Init(serde_json::from_str::<Init>(&line).ok()?),
        };
        Some(hist_action)

    }

    pub fn peek_next_time(&mut self) -> Option<f64>{
        let a = self
            .read_buffer
            .peek()?
            .as_ref()
            .ok()?;
        let next = Self::de(&a)?;
        match next {
            HistoricalAction::Event(event) => Some(event.ts),
            _ => None
        }

    }
}

impl Iterator for BufferedActionRecordReader{
    type Item = HistoricalAction;
    fn next(&mut self) -> Option<Self::Item> {
        let next_line = self.read_buffer.next()?.ok()?;
        Self::de(&next_line)
    }
}

// have this open a file and read it into a buffer
pub struct HistoricalActionProducer {
    timer: Rc<Timer>,
    trader: Rc<Trader>,
    action_buffer: BufferedActionRecordReader,
}



impl HistoricalActionProducer{

    pub fn new(timer: &Rc<Timer>, trader: &Rc<Trader>, path: &str) -> Result<Self, Box<dyn Error>> {
        let action_reader = BufferedActionRecordReader::new(path)?;
        Ok(Self{
            timer: timer.clone(),
            trader: trader.clone(),
            action_buffer: action_reader,
        })
    }

    // pop all actions that will occur before the next time step
    pub fn grab_cycle(&mut self) -> Vec<Action> {
        let mut current_actions = Vec::new();

        // pop actions that will occur before next time step and return them
        while let Some(action) = self.pop_next_action() {
            current_actions.push(action);
        }
        current_actions
    }

    // only pops actions if it will occur before next time step
    fn pop_next_action(&mut self) -> Option<Action> {
        let next_ts = self.timer.peek_next_time();
        
        let next_item_time = self.action_buffer.peek_next_time()?;
        if next_item_time < next_ts {
            return Some(self.action_buffer.next()?.into_action(&self.trader).ok()?)
        } else {
            return None
        };
    }

    // force grabs next item in the read_buffer even if it's time stamp is too far
    // in the future
    pub fn force_grab_next(&mut self) -> Option<Action> {
        Some(
            self
                .action_buffer
                .next()?
                .into_action(&self.trader)
                .ok()?
        )
    }
}


