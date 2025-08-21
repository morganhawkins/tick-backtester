**/src/bin**
- add binary or .py to format JSON data from kalshi as a deserializable format strait into action types
 - book delta -> order place
 - trade -> trade take
 - timestamp string -> f64 unix 
- add binary or .py to clean paired (TradeUpdate, OrderBookDelta) into just trade updates
    - how easy is it to id pairs?
    - how many cases do we have multiple candidates? which to select?
- binary in other project to grab meta data associated with tick data
- binary to grab cb tick data

**src/actions/historical_action_producer.rs**
- control memory usage, only load a portion of dataset at a time?
    - implement buffered reader or something like that
    - make this concurrent so buffer is written to when data is not being accessed?
- benchamrk how big this buffer needs to be

**/src/order_book/order_book.rs**
- implement methods to digest updates
    - Trader::Other updates should add to back and subtract from front (or maybe randomize where in the book it subtracts from)
    - Trader::Me updates should add to back and subtract from back
- implement method to generate update from an action. Needs to look at self
- add error corrector


