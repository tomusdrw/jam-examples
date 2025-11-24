//! JAM WASMI Service
//!
//! Provides a no_std-friendly wrapper around the `wasmi` interpreter so that a JAM
//! service can execute WASM blobs either embedded in the service binary or supplied
//! through work item payloads.

#![cfg_attr(any(target_arch = "riscv32", target_arch = "riscv64"), no_std)]
#![allow(clippy::unwrap_used)]

extern crate alloc;

use alloc::string::ToString;
use jam_pvm_common::{
    accumulate::{get_storage, set_storage},
    *,
};
use jam_types::*;

const REFINE_ENTRYPOINT: &str = "refine";
const ACCUMULATE_ENTRYPOINT: &str = "accumulate";

/// TODO [ToDr] make a proper enum. When invoking module 0 it actually deploys the wasm code.
const DEPLOY_CODE_MODULE: u32 = 0;

pub struct Service;
jam_pvm_common::declare_service!(Service);

impl jam_pvm_common::Service for Service {
    fn refine(
        _core_index: CoreIndex,
        _item_index: usize,
        service_id: ServiceId,
        payload: WorkPayload,
        _package_hash: WorkPackageHash,
    ) -> WorkOutput {
        info!(target = "wasmi", "WASMI Service Refine, {service_id:x}h");
        let invocation = InvokeData::decode(&mut &**payload).expect("Invalid invoke data");
        let InvokeData { module, input } = invocation;
        //
        if module == DEPLOY_CODE_MODULE {
            info!(target = "wasmi", "deploying code - passing to accumulate");
            return payload.0.into();
        }
        let code = module_code(module);
        if let Some(code) = code {
            execute_wasm(service_id, 0, REFINE_ENTRYPOINT, &code, &input);
            // TODO [ToDr] Construct output from refine output
            payload.0.into()
        } else {
            panic!("No code for {module}");
        }
    }

    fn accumulate(now: Slot, id: ServiceId, _item_count: usize) -> Option<Hash> {
        use accumulate::*;
        info!(
            target = "wasmi",
            "WASMI Service Accumulate, {id:x}h @{now} ${}",
            my_info().balance
        );
        for item in accumulate::accumulate_items() {
            match item {
                AccumulateItem::WorkItem(r) => on_work_item(r, now, id),
                AccumulateItem::Transfer(t) => on_transfer(t),
            }
        }
        None
    }
}

fn execute_wasm(service_id: ServiceId, now: Slot, entrypoint: &str, code: &[u8], args: &[u8]) {
    info!(
        target = "wasmi",
        "[{service_id}] Executing WASM blob ({len} bytes) with args: ({args} bytes)",
        len = code.len(),
        args = args.len(),
    );

    let host_state = host::HostArgs::new(service_id, now);
    match wasm_runtime::execute(code, entrypoint, host_state) {
        Ok(final_state) => {
            if let Some(value) = final_state.last_guest_log() {
                info!(target = "wasmi", "Guest emitted log value {value}");
            }
            info!(
                target = "wasmi",
                "Finished running `{entry}` from WASM for {service:x}h",
                entry = entrypoint,
                service = final_state.service_id()
            );
        }
        Err(err) => {
            error!(target = "wasmi", "WASM execution failed: {err}",);
        }
    }
}

#[derive(Encode, Decode)]
pub struct InvokeData {
    module: u32,
    input: Vec<u8>,
}

fn on_work_item(item: WorkItemRecord, now: Slot, service_id: ServiceId) {
    match item.result {
        Ok(data) => {
            let invocation = InvokeData::decode(&mut &**data).expect("Invoke data invalid");
            let InvokeData { module, input } = invocation;
            if module == DEPLOY_CODE_MODULE {
                module_code_set(module, input);
                return;
            }
            let code = module_code(module);
            if let Some(code) = code {
                execute_wasm(service_id, now, ACCUMULATE_ENTRYPOINT, &code, &input);
            } else {
                info!("No code for module {module}. Skipping.");
            }
        }
        Err(err) => {
            info!("Invalid work item: {err:?}")
        }
    }
}

mod host {
    use super::*;
    use wasmi::{Caller, Linker};

    #[derive(Debug)]
    pub struct HostArgs {
        service_id: ServiceId,
        now: Slot,
        last_guest_log: Option<i64>,
    }

    impl HostArgs {
        pub fn new(service_id: ServiceId, now: Slot) -> Self {
            Self {
                service_id,
                now,
                last_guest_log: None,
            }
        }

        pub fn service_id(&self) -> &ServiceId {
            &self.service_id
        }

        pub fn slot(&self) -> Slot {
            self.now
        }

        pub fn last_guest_log(&self) -> Option<i64> {
            self.last_guest_log
        }

        fn record_guest_log(&mut self, value: i64) {
            self.last_guest_log = Some(value);
        }
    }

    /// Wires a minimal logging helper that records guest-provided values.
    pub fn wire_default_host_functions(linker: &mut Linker<HostArgs>) -> Result<(), wasmi::Error> {
        linker.func_wrap(
            "env",
            "log_u64",
            |mut caller: Caller<'_, HostArgs>, value: i64| {
                caller.data_mut().record_guest_log(value);
                info!(
                    target = "wasmi",
                    "Guest requested log value {value} @{slot}",
                    slot = caller.data().slot()
                );
                Ok(())
            },
        )?;
        Ok(())
    }

    /// Extend this to expose additional host functions to your WASM module.
    pub fn wire_custom_host_functions(_linker: &mut Linker<HostArgs>) -> Result<(), wasmi::Error> {
        // Example:
        // _linker.func_wrap("env", "host_call", |caller: Caller<'_, HostState>, arg: i32| {
        //     let state = caller.data();
        //     // TODO use JAM host capabilities here.
        //     Ok(())
        // })?;
        Ok(())
    }
}

mod wasm_runtime {
    use super::host;
    use wasmi::{Config, Engine, Instance, Linker, Module, Store, TypedFunc};

    pub fn execute(
        program: &[u8],
        entrypoint: &str,
        host_state: host::HostArgs,
    ) -> Result<host::HostArgs, wasmi::Error> {
        let config = Config::default();
        // Clone and tweak `config` if you need deterministic-only mode, fuel metering, etc.
        let engine = Engine::new(&config);
        let module = Module::new(&engine, program)?;
        let mut linker = Linker::new(&engine);
        host::wire_default_host_functions(&mut linker)?;
        host::wire_custom_host_functions(&mut linker)?;

        let mut store = Store::new(&engine, host_state);
        let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;
        call_entrypoint(entrypoint, &instance, &mut store)?;
        Ok(store.into_data())
    }

    fn call_entrypoint(
        entrypoint: &str,
        instance: &Instance,
        store: &mut Store<host::HostArgs>,
    ) -> Result<(), wasmi::Error> {
        if entrypoint.is_empty() {
            return Ok(());
        }

        let func: TypedFunc<(), ()> = instance.get_typed_func(&mut *store, entrypoint)?;
        func.call(store, ())?;
        Ok(())
    }
}

fn on_transfer(item: TransferRecord) {
    use accumulate::*;
    let TransferRecord {
        source,
        amount,
        memo,
        ..
    } = item;
    let count = get::<u32>(b"transfer-count").unwrap_or(0);
    set(b"transfer-count", count + 1).expect("balance?");
    info!(
        target = "boot",
        "Received transfer from {source} of {amount} with memo {}",
        alloc::string::String::from_utf8(memo.as_ref().to_vec()).unwrap_or("???".to_string())
    );
    set_storage(
        alloc::format!("transfer{count}").as_bytes(),
        &(source, amount, memo).encode()[..],
    )
    .expect("balance?");
}

fn module_code(module: u32) -> Option<Vec<u8>> {
    let storage = get_storage(alloc::format!("module{module}").as_bytes());
    storage
}

fn module_code_set(module: u32, code: Vec<u8>) {
    set_storage(alloc::format!("module{module}").as_bytes(), &code)
        .expect("Unable to deploy module code");
}

pub const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
