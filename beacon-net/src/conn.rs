use beacon_codec::ProtocolState;
use bevy_ecs::prelude::*;
use crossbeam_channel::{Receiver, Sender};

use crate::{packet::RawPacket, server::*};
