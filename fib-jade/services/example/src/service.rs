//! Fibonacci JADE service

use jade::{
    prelude::Vec,
    service::{OpaqueHash, vm::AccumulateItem},
};

#[jade::refine]
fn refine(_core: u16, _index: u16, _service_id: u32, payload: Vec<u8>, _package_hash: OpaqueHash) -> Vec<u8> {
    payload
}

#[jade::accumulate]
fn accumulate(_now: u32, _id: u32, _items: Vec<AccumulateItem>) -> Option<OpaqueHash> {
    // Calculate 10th Fibonacci number: 55
    let n: u64 = 10;
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    for _ in 0..n {
        let temp = a.wrapping_add(b);
        a = b;
        b = temp;
    }
    // Result is in `a` (fib(10) = 55)
    None
}
