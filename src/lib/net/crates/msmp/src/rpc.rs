use jsonrpc_core::{ErrorCode, Params};
pub use openrpc::discover;

mod allowlist;
mod bans;
mod gamerules;
mod ip_bans;
mod openrpc;
mod operators;
mod players;
mod server;
mod serversettings;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("JSON-RPC error: {0}")]
    Rpc(#[from] jsonrpc_core::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Serialize)]
pub enum RpcNotification {
    Server(server::ServerNotification),
    Players(players::PlayerNotification),
    Operators(operators::OperatorNotification),
    Allowlist(allowlist::AllowlistNotification),
    IPBans(ip_bans::IPBanNotification),
    Bans(bans::BanNotification),
    Gamerules(gamerules::GameruleNotification),
}

type RpcHandler = fn(Params) -> Result<jsonrpc_core::Value, RpcError>;

pub enum RpcMethod {
    Supported {
        name: &'static str,
        handler: RpcHandler,
    },
    Unsupported {
        name: &'static str,
    },
}

impl RpcMethod {
    /// Add this RPC method to the given handler.
    pub fn add(&self, io: &mut jsonrpc_core::IoHandler) {
        match self {
            RpcMethod::Unsupported { name } => {
                let name = name.to_string();
                io.add_sync_method(&format!("minecraft:{}", name), move |_: Params| {
                    Err(jsonrpc_core::Error {
                        code: ErrorCode::MethodNotFound,
                        message: "Method unsupported".into(),
                        data: Some(
                            format!("Method unsupported by beacon: minecraft:{name}").into(),
                        ),
                    })
                });
            }
            RpcMethod::Supported { name, handler } => {
                let handler = *handler;
                io.add_sync_method(&format!("minecraft:{}", name), move |params: Params| {
                    (handler)(params).map_err(|e| match e {
                        RpcError::Rpc(err) => err,
                        RpcError::Serde(err) => {
                            jsonrpc_core::Error::invalid_params(format!("Serde error: {}", err))
                        }
                    })
                })
            }
        }
    }
}

inventory::collect!(RpcMethod);

#[macro_export]
macro_rules! method {
    ($name:expr, $handler:expr) => {
        inventory::submit! { $crate::rpc::RpcMethod::Supported { name: $name, handler: $handler } }
    };
    ($name:expr) => {
        inventory::submit! { $crate::rpc::RpcMethod::Unsupported { name: $name } }
    };
}
