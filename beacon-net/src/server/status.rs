use crate::prelude::*;

/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Status_Request>
#[packet(resource = "status_request", state = Status)]
#[derive(Debug)]
pub struct StatusRequest;