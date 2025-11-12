mod allowlist;
mod bans;
mod gamerules;
mod ip_bans;
mod openrpc;
mod operators;
mod players;
mod server;
mod serversettings;

use jsonrpc_core::{ErrorCode, Params, Value};
pub use openrpc::discover;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("JSON-RPC error: {0}")]
    Rpc(#[from] jsonrpc_core::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub struct RpcMethod {
    name: &'static str,
    handler: fn(Params) -> Result<jsonrpc_core::Value, RpcError>,
}

impl RpcMethod {
    /// Create a new RPC method.
    const fn new(name: &'static str, handler: fn(Params) -> Result<Value, RpcError>) -> Self {
        RpcMethod { name, handler }
    }

    /// A default handler that returns "method not found".
    fn unimplemented(_: Params) -> Result<Value, RpcError> {
        Err(RpcError::Rpc(jsonrpc_core::Error {
            code: ErrorCode::MethodNotFound,
            message: "beacon does not yet support this method.".into(),
            data: None,
        }))
    }

    /// Add this RPC method to the given handler.
    pub fn add(&self, io: &mut jsonrpc_core::IoHandler) {
        let name = self.name;
        let handler = self.handler;
        io.add_sync_method(&format!("minecraft:{}", name), move |params: Params| {
            (handler)(params).map_err(|e| match e {
                RpcError::Rpc(err) => err,
                RpcError::Serde(err) => {
                    jsonrpc_core::Error::invalid_params(format!("Serde error: {}", err))
                }
            })
        });
    }
}

inventory::collect!(RpcMethod);

#[macro_export]
macro_rules! method {
    ($name:expr, $handler:expr) => {
        inventory::submit! { $crate::method::RpcMethod::new($name, $handler) }
    };
    ($name:expr) => {
        $crate::method!($name, $crate::method::RpcMethod::unimplemented);
    };
}
