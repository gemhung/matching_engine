#![feature(map_first_last)]
#![allow(unused)]

use std::borrow::Borrow;
use std::cmp::Ord;
use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
enum Side {
    Bid = 0,
    Ask = 1,
}

impl Side {
    fn toggle(&self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct Order {
    pub order_no: usize,
    pub price: usize, // if market order, it's ignored
    pub qty: usize,
    pub side: Side,
    pub order_type: OrderType,
    pub condition: Condition,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Session {
    Morning,
    Afternoon,
}

#[derive(Clone, Eq, PartialEq)]
enum Condition {
    OnOpen,
    OnClose,
    Funari,
    IOC, // Fill and Kill
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
enum OrderType {
    Limit = 0,
    Market = 1,
}

#[derive(Clone, Eq, PartialEq)]
enum IN {
    Limit(usize, OrderNo),
    LimitRev(std::cmp::Reverse<usize>, OrderNo),
    Market(OrderNo), // this index has no price reference cause it's market order
}

impl<T: Borrow<Order>> From<T> for IN {
    fn from(order: T) -> IN {
        let order = order.borrow();
        match (order.side, order.order_type) {
            (_, OrderType::Market) => IN::Market(OrderNo(order.order_no)),
            (Side::Bid, OrderType::Limit) => {
                IN::LimitRev(Reverse(order.price), OrderNo(order.order_no))
            }
            (Side::Ask, OrderType::Limit) => IN::Limit(order.price, OrderNo(order.order_no)),
        }
    }
}

impl Ord for IN {
    fn cmp(&self, other: &IN) -> std::cmp::Ordering {
        match (self, other) {
            (IN::Market(order_no1), IN::Market(order_no2)) => order_no1.cmp(order_no2),
            (IN::Market(_), _) => std::cmp::Ordering::Less,
            (_, IN::Market(_)) => std::cmp::Ordering::Greater,
            (IN::Limit(price1, order_no1), IN::Limit(price2, order_no2)) => {
                price1.cmp(price2).then_with(|| order_no1.cmp(order_no2))
            }
            (IN::LimitRev(price1, order_no1), IN::LimitRev(price2, order_no2)) => {
                price1.cmp(price2).then_with(|| order_no1.cmp(order_no2))
            }
            _ => panic!(""),
        }
    }
}

impl PartialOrd for IN {
    fn partial_cmp(&self, other: &IN) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct OrderNo(usize);

#[derive(Clone)]
struct Quote(usize, usize); // (price, quantity)

#[derive(Clone, Default, Eq, PartialEq)]
struct Book(BTreeSet<IN>);

impl Book {
    pub fn new() -> Book {
        Book(BTreeSet::new())
    }

    fn peek(&self) -> Option<(&usize, &OrderNo)> {
        self.first().map(|index| match index {
            IN::Market(order_no) => (&0, order_no),
            IN::Limit(price, order_no) => (price, order_no),
            IN::LimitRev(Reverse(price), order_no) => (price, order_no),
        })
    }

    fn pop(&mut self) -> Option<(usize, OrderNo)> {
        self.pop_first().map(|index| match index {
            IN::Market(order_no) => (0, order_no),
            IN::Limit(price, order_no) => (price, order_no),
            IN::LimitRev(Reverse(price), order_no) => (price, order_no),
        })
    }
}

impl Deref for Book {
    type Target = BTreeSet<IN>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Book {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
struct OrderBook {
    pub name: String,
    pub books: [Book; 4], // [0: bid_limit, 1: ask_limit, 2: bid_market, 3: ask_market]
    pub orders: HashMap<OrderNo, Order>,
}

impl OrderBook {
    fn write(&mut self, order: Order) {
        self.orders.insert(OrderNo(order.order_no), order.clone());
        self.books[OrderBook::index(&order)].insert(order.into());
    }

    fn index(order: &Order) -> usize {
        order.side as usize + order.order_type as usize * 2
    }

    fn limit_book_matches(&mut self, incoming: &mut Order) {
        self.book_matches(incoming.side.toggle() as usize, incoming);
    }

    fn market_book_matches(&mut self, incoming: &mut Order) {
        self.book_matches(incoming.side.toggle() as usize + 2, incoming);
    }

    fn book_matches(&mut self, index: usize, incoming: &mut Order) -> Result<(), anyhow::Error> {
        let book = &mut self.books[index];
        while let Some((_, order_no)) = book.peek() {
            /*
            // might be already canceled
            if !self.orders.contains_key(&order_no) {
                let _ = book.pop();
                continueg;
            }
            */

            // Book            v.s incoming order
            // bid best_price  >=  ask price
            // ask best_price  <=  bid price
            let best_price = match try_trade(self.orders.get(&order_no).unwrap(), &incoming) {
                Some(inner) => inner,
                None => break, // there is a bid-ask spread
            };

            let (_, order_no) = book.pop().unwrap(); // never failed cause tradable
            let mut book_order = self.orders.remove(&order_no).unwrap(); // never failed cause tradable

            let min_qty = std::cmp::min(book_order.qty, incoming.qty);
            incoming.qty -= min_qty;
            book_order.qty -= min_qty;

            if book_order.qty != 0 {
                book.insert(book_order.into());
            }

            if incoming.qty == 0 {
                break;
            }
        }

        Ok(())
    }
}

fn try_trade(book_order: &Order, incoming: &Order) -> Option<usize> {
    match (book_order, incoming) {
        (
            Order {
                order_type: OrderType::Market,
                ..
            },
            Order {
                order_type: OrderType::Limit,
                ..
            },
        ) => Some(incoming.price),
        (
            Order {
                order_type: OrderType::Limit,
                ..
            },
            Order {
                order_type: OrderType::Market,
                ..
            },
        ) => Some(book_order.price),

        (
            Order {
                order_type: OrderType::Limit,
                price: p1,
                side: s1,
                ..
            },
            Order {
                order_type: OrderType::Limit,
                price: p2,
                side: s2,
                ..
            },
        ) => match (s1, s2) {
            // tradable if bid price is higher or equal to ask price
            (Side::Bid, Side::Ask) => (p1 >= p2).then(|| *p2),
            (Side::Ask, Side::Bid) => (p1 <= p2).then(|| *p1),
            _ => unreachable!(""),
        },

        _ => unreachable!(""),
    }
}

enum Status {
    Filled,
    PartialFilled,
}

impl OrderBook {
    pub fn new(name: String) -> OrderBook {
        OrderBook {
            name,
            ..Default::default()
        }
    }

    // O(log n)
    pub fn matches(&mut self, mut incoming: Order) {
        // if it's a limit order, first check if there is a market order in book to match
        if incoming.order_type == OrderType::Limit {
            self.market_book_matches(&mut incoming);
        }

        // then check if there is a limit order in book to match
        if incoming.qty != 0 {
            self.limit_book_matches(&mut incoming);
        }

        // write back unfilled part
        if incoming.qty != 0 && incoming.condition != Condition::IOC {
            self.write(incoming);
        }
    }

    // O(log n)
    // Note: if price changed, we need to match it rather than just update
    pub fn amend(&mut self, target: Order) {
        let cur = self.orders.get(&OrderNo(target.order_no)).unwrap();

        if cur.price != target.price {
            // remove old price in book
            let index = OrderBook::index(&cur);
            let _r: bool = self.books[index].remove(&cur.into());
            self.matches(target);
        } else {
            self.orders
                .get_mut(&OrderNo(target.order_no))
                .map(|old: &mut Order| {
                    let index = OrderBook::index(&target);
                    // remove old price in book
                    let _r: bool = self.books[index].remove(&old.into());
                    // insert new price in book
                    let _r: bool = self.books[index].insert(target.borrow().into());

                    // update old in hashmap
                    *old = target;
                });
        }
    }

    // O(log n)
    pub fn cancel(&mut self, target: Order) {
        self.orders.remove(&OrderNo(target.order_no));
        self.books[OrderBook::index(&target)].remove(&target.into());
    }
}

fn main() {
    let bid_book = Book(BTreeSet::new());
    let order_book = OrderBook::new("AAPL".to_string());

    println!("Hello, world!");
}
