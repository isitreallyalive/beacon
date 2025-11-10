use std::cell::LazyCell;

use jsonrpc_core::Value;
use serde_json::Map;

use crate::rpc::RpcMethod;

const CONTENTS: LazyCell<Map<String, Value>> = LazyCell::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/json-rpc-api-schema.json"
    )))
    .unwrap()
});

// return
inventory::submit! { RpcMethod::new("rpc.discover", |_| Ok(Value::Object(CONTENTS.clone()))) }
