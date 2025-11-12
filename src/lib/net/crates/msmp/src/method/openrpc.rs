use std::cell::LazyCell;

use jsonrpc_core::{Params, Result, Value};
use serde_json::Map;

const CONTENTS: LazyCell<Map<String, Value>> = LazyCell::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/json-rpc-api-schema.json"
    )))
    .unwrap()
});

pub fn discover(_: Params) -> Result<Value> {
    Ok(Value::Object(CONTENTS.clone()))
}
