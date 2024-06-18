use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Side {
    Buy,
    Sell,
}
impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell"),
        }
    }
}
#[derive(Debug, Clone)]
struct Order {
    id: i64,
    price: Decimal,
    quantity: i64,
    side: Side,
}

#[derive(Debug, Clone)]
struct Fill {
    matched_id: i64,
    volume: i64,
    price: Decimal,
}

#[derive(Debug, Clone)]
struct OrderBook {
    bids: BTreeMap<Decimal, Vec<Order>>,
    asks: BTreeMap<Decimal, Vec<Order>>,
    orders: HashMap<i64, Order>,
    match_id: i64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            match_id: 0,
        }
    }

    fn print_book(&self) {
        println!("## Orderbook");
        println!("{:<8} {:<8} {:<8} {:<8}", "ID", "Side", "Volume", "Price");

        for (price, orders) in self.asks.iter().rev() {
            for order in orders {
                println!(
                    "{:<8} {:<8} {:<8} {:<8}",
                    order.id, order.side, order.quantity, price
                );
            }
        }

        println!("{:-<32}", "");

        for (price, orders) in self.bids.iter().rev() {
            for order in orders {
                println!(
                    "{:<8} {:<8} {:<8} {:<8}",
                    order.id, order.side, order.quantity, price
                );
            }
        }
    }

    fn add_order(&mut self, mut order: Order) -> Vec<Fill> {
        let mut fills = Vec::new();

        if order.side == Side::Buy {
            while order.quantity > 0 {
                if let Some((&ask_price, ask_orders)) = self.asks.iter_mut().next() {
                    if order.price >= ask_price {
                        let mut remaining_quantity = order.quantity;
                        let mut i = 0;

                        while i < ask_orders.len() && remaining_quantity > 0 {
                            let ask_order = &mut ask_orders[i];
                            let trade_quantity = remaining_quantity.min(ask_order.quantity);
                            self.match_id += 1;
                            fills.push(Fill {
                                matched_id: self.match_id,
                                volume: trade_quantity,
                                price: ask_price,
                            });

                            ask_order.quantity -= trade_quantity;
                            remaining_quantity -= trade_quantity;

                            if ask_order.quantity == 0 {
                                self.orders.remove(&ask_order.id);
                                ask_orders.remove(i);
                            } else {
                                i += 1;
                            }
                        }

                        order.quantity = remaining_quantity;

                        if ask_orders.is_empty() {
                            self.asks.remove(&ask_price);
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if order.quantity > 0 {
                self.orders.insert(order.id, order.clone());
                self.bids
                    .entry(order.price)
                    .or_insert_with(Vec::new)
                    .push(order);
            }
        } else {
            while order.quantity > 0 {
                if let Some((&bid_price, bid_orders)) = self.bids.iter_mut().rev().next() {
                    if order.price <= bid_price {
                        let mut remaining_quantity = order.quantity;
                        let mut i = 0;

                        while i < bid_orders.len() && remaining_quantity > 0 {
                            let bid_order = &mut bid_orders[i];
                            let trade_quantity = remaining_quantity.min(bid_order.quantity);
                            self.match_id += 1;
                            fills.push(Fill {
                                matched_id: self.match_id,
                                volume: trade_quantity,
                                price: bid_price,
                            });

                            bid_order.quantity -= trade_quantity;
                            remaining_quantity -= trade_quantity;

                            if bid_order.quantity == 0 {
                                self.orders.remove(&bid_order.id);
                                bid_orders.remove(i);
                            } else {
                                i += 1;
                            }
                        }

                        order.quantity = remaining_quantity;

                        if bid_orders.is_empty() {
                            self.bids.remove(&bid_price);
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if order.quantity > 0 {
                self.orders.insert(order.id, order.clone());
                self.asks
                    .entry(order.price)
                    .or_insert_with(Vec::new)
                    .push(order);
            }
        }
        fills
    }

    fn remove_order(&mut self, id: i64) -> Option<Order> {
        if let Some(order) = self.orders.remove(&id) {
            let book_side = match order.side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };

            if let Some(orders) = book_side.get_mut(&order.price) {
                if let Some(pos) = orders.iter().position(|o| o.id == order.id) {
                    orders.remove(pos);
                    if orders.is_empty() {
                        book_side.remove(&order.price);
                    }
                    return Some(order);
                }
            }
        }
        return Option::None;
    }
}

fn print_fills(fills: &[Fill]) {
    println!("## Fills");
    println!("{:<10} {:<8} {:<8}", "MatchedId", "Volume", "Price");
    for fill in fills {
        println!(
            "{:<10} {:<8} {:<8}",
            fill.matched_id, fill.volume, fill.price
        );
    }
    println!()
}

fn main() {
    let mut order_book = OrderBook::new();

    let order1 = Order {
        id: 1,
        price: dec!(100.0),
        quantity: 10,
        side: Side::Buy,
    };
    let order2 = Order {
        id: 2,
        price: dec!(100.0),
        quantity: 5,
        side: Side::Buy,
    };
    let order3 = Order {
        id: 3,
        price: dec!(101.0),
        quantity: 7,
        side: Side::Buy,
    };

    order_book.add_order(order1);
    order_book.add_order(order2);
    order_book.add_order(order3);

    let order4 = Order {
        id: 4,
        price: dec!(99.0),
        quantity: 18,
        side: Side::Sell,
    };

    let fills = order_book.add_order(order4);

    print_fills(&fills);
    order_book.print_book();

    order_book.remove_order(2);
    order_book.print_book();
}
