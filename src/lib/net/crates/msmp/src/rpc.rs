mod openrpc;

pub struct RpcMethod {
    name: &'static str,
    handler: fn(jsonrpc_core::Params) -> jsonrpc_core::Result<jsonrpc_core::Value>,
}

impl RpcMethod {
    pub const fn new(
        name: &'static str,
        handler: fn(jsonrpc_core::Params) -> jsonrpc_core::Result<jsonrpc_core::Value>,
    ) -> Self {
        RpcMethod { name, handler }
    }

    pub fn add(&self, io: &mut jsonrpc_core::IoHandler) {
        io.add_sync_method(self.name, self.handler);
    }
}

inventory::collect!(RpcMethod);
