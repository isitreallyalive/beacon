use crate::method::RpcMethod;

inventory::submit! { RpcMethod::new("allowlist", None) }
inventory::submit! { RpcMethod::new("allowlist/set", None) }
inventory::submit! { RpcMethod::new("allowlist/add", None) }
inventory::submit! {RpcMethod::new("allowlist/remove", None)}
inventory::submit! { RpcMethod::new("allowlist/clear", None) }
