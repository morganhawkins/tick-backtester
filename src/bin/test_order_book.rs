use std::rc::Rc;

use tick_backtest::actions::action_producer::ActionProducer;
use tick_backtest::order_book::order_book::OrderBook;
use tick_backtest::order_book::updates::{Side, Trader};
use tick_backtest::time_keeping::timer::Timer;

fn main() {
    let timer = Rc::new(Timer::new(1756428105.0, 0.5));
    let me = Rc::new(Trader::new_me());
    let other = Rc::new(Trader::new_other());
    let book = OrderBook::from_snapshot(
        vec![(51, 10)],
        vec![(49, 10)],
        &other,
    );
    let me_producer = ActionProducer::new(&timer, &me, 0.0);
    let other_producer = ActionProducer::new(&timer, &other, 0.0);

    println!("\n   me: shares {} cents {}", me.shares(), me.cents());
    println!("other: shares {} cents {}", other.shares(), other.cents());
    println!("{:?}\n", book);
    
    let act = me_producer.trade_take(55, 10, Side::Buy);
    book.digest_action(act);

    println!("   me: shares {} cents {}", me.shares(), me.cents());
    println!("other: shares {} cents {}", other.shares(), other.cents());
    println!("{:?}\n", book);
    
    let act = me_producer.trade_take(55, 10, Side::Buy);
    book.digest_action(act);

    println!("   me: shares {} cents {}", me.shares(), me.cents());
    println!("other: shares {} cents {}", other.shares(), other.cents());
    println!("{:?}\n", book);
    

}
