#![allow(unused)]

use std::cmp::Ord;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::*;
use std::ops::Deref;

#[derive(Clone, Eq, PartialEq)]
enum Side {
    Sell,
    Buy,
}

#[derive(Clone, Eq, PartialEq)]
struct Order {
    pub order_no: usize,
    pub price: usize,
    pub qty: usize,
    pub side: Side,
    pub order_type: OrderType,
    pub condition: Condition
}

#[derive(Clone, Eq, PartialEq)]
enum Condition {
    GFD, // Good for Day
    GTD, // Good till Date
    FAK, // Fill and Kill (IOC)
    FOK, // Fill or Kill
}

#[derive(Clone, Eq, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

impl Ord for Order {
    fn cmp(&self, other: &Order) -> std::cmp::Ordering {
        self.price.cmp(&other.price)
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Order) -> Option<std::cmp::Ordering> {
        Some(self.price.cmp(&other.price))
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct OrderNo(usize);

#[derive(Clone)]
struct Quote(usize, usize); // (price, quantity)

struct Book<INDEX: Ord, T> {
    pub price: BTreeMap<INDEX, BTreeSet<OrderNo>>,
    pub orders: HashMap<OrderNo, T>, // so we can remove and amend
}

impl<INDEX: Ord, T> Book<INDEX, T> {
    pub fn new() -> Book<INDEX, T> {
        Book {
            price: BTreeMap::<INDEX, BTreeSet<OrderNo>>::new(),
            orders: HashMap::<OrderNo, T>::new(),
        }
    }
}

impl<INDEX: Ord> Book<INDEX, Order> {
    // O(log n)
    pub fn best_price(&self) -> Option<&INDEX> {
        self.price.keys().next()
    }
}

impl<INDEX: Ord, T> Deref for BuyBook<INDEX, T> {
    type Target = Book<std::cmp::Reverse<INDEX>, T>;
    fn deref(&self) -> &Self::Target {
        &self.book
    }
}

impl<INDEX: Ord, T> Deref for SellBook<INDEX, T> {
    type Target = Book<INDEX, T>;
    fn deref(&self) -> &Self::Target {
        &self.book
    }
}

struct BuyBook<INDEX: Ord, T> {
    pub book: Book<std::cmp::Reverse<INDEX>, T>,
}

struct SellBook<INDEX: Ord, T> {
    pub book: Book<INDEX, T>,
}

struct Asset {
    pub name: String,
    pub buy: BuyBook<usize, Order>,   // max-heap
    pub sell: SellBook<usize, Order>, // min-heap
}

// IOC
// FOK
// Market
// Limit

impl Asset {
    pub fn matches(&mut self, mut order: Order) -> Result<(), anyhow::Error> {
        


        


        


        Ok(())
    }
    //pub fn matches(&mut self, mut order:Order) -> Result<(), anyhow::Error>{
    //match order {
    //Order{side: Side::Buy, order_type:OrderType::Market, ..}  => {}
    //Order{side: Side::Buy, ..} if order.price >= sell. => {}
    //_ => {}
    //};

    //Ok(())
    //}

    fn buy(&mut self, order: Order) {}

    fn sell(&mut self, order: Order) {}

    /*
    pub fn matches(&mut self, mut order: Order) -> Result<(), anyhow::Error>{
        // highest Buy, sell
        // lowest Sell, buy
        while order.qty != 0 && matches!(self.price_heap.peek(), Some(Order{price, ..}) if price <= &order.price) {
            let booked_qty = self.price_heap.peek().unwrap().qty;
            if booked_qty > order.qty{
                self.price_heap.peek_mut().map(|mut booked|{
                    booked.qty-= std::mem::replace(&mut order.qty, 0);
                    booked
                });
            }else {
                let booked = self.price_heap.pop().unwrap();
                order.qty-=booked.qty;
            }
        }

        if order.qty != 0 {

        }

        Ok(())
    }
    */
}

fn main() {
    let sell_book = SellBook {
        book: Book::<usize, Order>::new(),
    };

    let best_price = sell_book.best_price();
    //let r = sell_book.get(Order);

    /*
    let buy_order_book = OrderBook {
        price_heap: BinaryHeap::<Order>::new(), // max-heap
        cancel_map: HashSet::<OrderNo>::new(),
        best_price: None,
        last_executed_quote: None,
    };
    */

    println!("Hello, world!");
}
