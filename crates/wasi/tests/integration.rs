use std::{collections::HashMap, env, path::PathBuf};

use feta_core::config::Config as FetaConfig;
use wasmtime::{
    component::{Component, HasSelf, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::host_bindings::{feta::wasi::tracking::Event, Decision, FetaWasi, Reason, Value};

pub mod host_bindings {
    wasmtime::component::bindgen!({
        path: "wit/feta-wasi.wit",
        additional_derives: [PartialEq],
    });
}

struct State {
    ctx: WasiCtx,
    table: ResourceTable,
    pub captured_events: Vec<Event>,
}

impl host_bindings::feta::wasi::tracking::Host for State {
    fn track_event(&mut self, event: Event) -> () {
        self.captured_events.push(event);
    }
}

impl WasiView for State {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

#[test]
fn test_instance() {
    let config_json = feta_integration::CONFIG.to_string();
    let config: FetaConfig =
        serde_json::from_str(&config_json).expect("failed to deserialize config");

    let (instance, mut store) = new_instance(&config_json);

    // decide cases
    let decide_cases = feta_integration::decide_cases();
    let num_decide_cases = decide_cases.len();

    for test in decide_cases {
        let context_json =
            serde_json::to_string(&test.context).expect("failed to serialize context");

        let actual = instance
            .call_decide(&mut store, &test.feature_key, &context_json)
            .expect("failed to call decide");
        assert_eq!(convert_decision(actual), test.expected)
    }

    // decide_all cases
    let decide_all_cases = feta_integration::decide_all_cases();
    let num_decide_all_cases = decide_all_cases.len();

    for test in decide_all_cases {
        let context_json =
            serde_json::to_string(&test.context).expect("failed to serialize context");

        let decisions = instance
            .call_decide_all(&mut store, &context_json)
            .expect("failed to call decide_all")
            .expect("decide_all failed");

        let actual: HashMap<String, feta_integration::Decision> = decisions
            .into_iter()
            .map(|(k, v)| (k, convert_decision(v)))
            .collect();

        assert_eq!(actual, test.expected)
    }

    let num_expected_events = num_decide_cases + num_decide_all_cases * config.features.len();

    let final_state = store.data();
    assert_eq!(final_state.captured_events.len(), num_expected_events);
}

fn new_instance(config_json: &str) -> (FetaWasi, Store<State>) {
    let mut config = Config::new();
    config.wasm_component_model(true);

    let engine = Engine::new(&config).unwrap();

    let mut linker: Linker<State> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    host_bindings::feta::wasi::tracking::add_to_linker::<State, HasSelf<State>>(
        &mut linker,
        |state| state,
    )
    .unwrap();

    let wasi = WasiCtxBuilder::new().inherit_stdio().build();

    let state = State {
        ctx: wasi,
        table: ResourceTable::new(),
        captured_events: Vec::new(),
    };

    let mut store = Store::new(&engine, state);
    let component = Component::from_file(&engine, get_wasm_path()).unwrap();

    let instance = FetaWasi::instantiate(&mut store, &component, &linker).unwrap();

    instance
        .call_init(&mut store, config_json)
        .unwrap()
        .unwrap();

    (instance, store)
}

fn get_wasm_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../target/wasm32-wasip2/release/feta_wasi.wasm");
    if !path.exists() {
        panic!(
            "WASM module not found at {:?}. Please build it with `cargo component build --release --target wasm32-wasip2`.",
            path
        );
    }
    path
}

fn convert_decision(decision: Decision) -> feta_integration::Decision {
    feta_integration::Decision {
        variant: decision.variant,
        reason: convert_reason(decision.reason),
        value: convert_value(decision.value),
        audience: decision.audience,
        has_error: decision.error.is_some(),
    }
}

fn convert_reason(reason: Reason) -> feta_core::Reason {
    match reason {
        Reason::Unknown => feta_core::Reason::Unknown,
        Reason::Static => feta_core::Reason::Static,
        Reason::Split => feta_core::Reason::Split,
        Reason::Match => feta_core::Reason::Match,
        Reason::MatchSplit => feta_core::Reason::MatchSplit,
        Reason::Disabled => feta_core::Reason::Disabled,
        Reason::Error => feta_core::Reason::Error,
    }
}

fn convert_value(value: Value) -> feta_core::Value {
    match value {
        Value::Null => feta_core::Value::Null,
        Value::Integer(i) => feta_core::Value::Integer(i),
        Value::Float(f) => feta_core::Value::Float(f),
        Value::Boolean(b) => feta_core::Value::Boolean(b),
        Value::String(s) => feta_core::Value::String(s),
    }
}
