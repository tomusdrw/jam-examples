//! JAM Bootstrap Service
//!
//! Use by concatenating one or more encoded `Instruction`s into a work item's payload.

#![cfg_attr(any(target_arch = "riscv32", target_arch = "riscv64"), no_std)]
#![cfg_attr(any(target_arch = "riscv32", target_arch = "riscv64"), no_main)]
#![allow(clippy::unwrap_used)]

extern crate alloc;

use game::Game;
use jam_pvm_common::{
	accumulate::*,
	*,
};
use jam_types::*;

mod game;

#[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
fn main() {}

#[allow(dead_code)]
struct Service;
jam_pvm_common::declare_service!(Service);

const SIZE_ENTRY: &[u8] = b"size";
const BOARD_ENTRY: &[u8] = b"board";

impl jam_pvm_common::Service for Service {
	fn refine(
		_id: ServiceId,
		_payload: WorkPayload,
		_package_info: PackageInfo,
		_extrinsics: Vec<Vec<u8>>,
	) -> WorkOutput {
		todo!()
	}

	fn accumulate(now: Slot, id: ServiceId, _results: Vec<AccumulateItem>) -> Option<Hash> {
        info!(target = "boot", "Executing acumulate at #{now} for service #{id}");
        let steps = now;
        let size = match get_storage(SIZE_ENTRY) {
            Some(v) => {
                let mut bytes = [0u8; 4];
                bytes[0..v.len()].copy_from_slice(&v);
                u32::from_le_bytes(bytes)
            },
            None => 8,
        };
        let mut game = match get_storage(BOARD_ENTRY) {
            Some(v) if v.len() as u32 == size * size => {
                Game::new(size, &v)
            },
            _ => Game::empty(size),
        };

        for _i in 0..steps {
            let mutations = game.next_step();
            game.mutate(&mutations);
        }

        set_storage(SIZE_ENTRY, &size.to_le_bytes()).expect("size");
        set_storage(BOARD_ENTRY, &game.export()).expect("board");

		None
	}

	fn on_transfer(_slot: Slot, _id: ServiceId, _items: Vec<TransferRecord>) {
        // TODO [ToDr] an idea for a demo:
        // 1. We use storage as display for GoL
        // 2. by sending `on_transfer` calls you can configure game parameters (board size?)
        // 3. The storage is updated every refine call.
        // 4. In refine we copy the storage to memory and perform as many steps as we can
        // 5. We might need to check the gas left to make sure we are able to sync the mem back to
        //    storage or use some storage item that tells us how many steps to perform.
        //
        // PVM debugger could have a special UI to display the storage as a board and
        // a special UI to send on_transfers for configuration.
	}
}
