#![cfg_attr(any(target_arch = "riscv32", target_arch = "riscv64"), no_std)]

extern crate alloc;

use jam_pvm_common::*;
use jam_types::*;

pub struct Service;
jam_pvm_common::declare_service!(Service);

impl jam_pvm_common::Service for Service {
    fn refine(
        _core_index: CoreIndex,
        _item_index: usize,
        service_id: ServiceId,
        _payload: WorkPayload,
        _package_hash: WorkPackageHash,
    ) -> WorkOutput {
        info!("Fibonacci Service Refine, {service_id:x}h");
        alloc::vec![].into()
    }

    fn accumulate(now: Slot, id: ServiceId, _item_count: usize) -> Option<Hash> {
        info!("Fibonacci Service Accumulate, {id:x}h @{now}");

        // Calculate fibonacci using accumulator pattern
        let n: u64 = 10; // Calculate fib(10)
        let result = fibonacci(n);
        info!("fibonacci({n}) = {result}");

        None
    }
}

/// Calculate fibonacci number using accumulator pattern (iterative approach)
fn fibonacci(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }

    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let temp = a.saturating_add(b);
        a = b;
        b = temp;
    }
    a
}

pub const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
