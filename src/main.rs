#![allow(unused)]

use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::collections::*;
use std::cmp::Ord;

#[derive(Clone, Eq, PartialEq)]
enum Side {
    Sell,
    Buy
}

#[derive(Clone, Eq, PartialEq)]
struct Order {
    pub order_no: usize,
    pub price: usize,
    pub qty: usize,
    pub side: Side,
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

struct OrderBook<K: Ord>{
    pub price_heap: BinaryHeap<K>,
    pub cancel_map: HashSet<OrderNo>,
    pub best_price: Option<usize>,
    pub last_executed_quote: Option<Quote>
}

impl OrderBook<Order> {
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

    pub fn cancel(order_no: OrderNo) -> Result<(), anyhow::Error> {
        Ok(())
    }

    pub fn best_price() -> Option<usize>{
        None
    }

    pub fn quote() -> Option<Quote>{
        None
    }
}


fn main() {
    let buy_order_book = OrderBook {
        price_heap: BinaryHeap::<Order>::new(), // max-heap
        cancel_map: HashSet::<OrderNo>::new(),
        best_price: None,
        last_executed_quote: None,
    };

    let sell_order_book = OrderBook {
        price_heap: BinaryHeap::<std::cmp::Reverse<Order>>::new(), // min-heap
        cancel_map: HashSet::<OrderNo>::new(),
        best_price: None,
        last_executed_quote: None,
    };

    println!("Hello, world!");
}

