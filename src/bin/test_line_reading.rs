use std::rc::Rc;

use tick_backtest::order_book::updates::Trader;
use tick_backtest::time_keeping::timer::Timer;
use tick_backtest::actions::historical_action_producer::HistoricalActionProducer;


fn main() {
    let path = "/Users/morganhawkins/Projects/current/tick-backtest/mock_data/kalshi_websocket_clean.txt";
    let timer = Rc::new(Timer::new(1756428105.0, 0.5));
    let other = Rc::new(Trader::new_other());
    let mut producer = HistoricalActionProducer::new(&timer, &other, path).unwrap();

    println!("{:?}", producer.force_grab_next());

    while timer.get_time() < 1756429205.0 {

        let cycle = producer.grab_cycle();
        println!("shares: {}", other.shares());
        println!("cents: {}", other.cents());
        println!("{:?}", cycle);
        println!("\n\n");

        timer.increment();
    }

}


