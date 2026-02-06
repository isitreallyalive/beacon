use beacon_codec::{ProtocolState, decode::*, types::VarInt};
use bevy_ecs::prelude::*;
use bytes::{Bytes, BytesMut};
use crossbeam_channel::{Receiver, Sender};
use futures::executor::block_on;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{
    conn::{PacketQueue, RawReceiver},
    server::*,
};

/// A raw packet, before any processing is done.
#[derive(Debug)]
pub struct RawPacket {
    id: VarInt,
    data: Bytes,
}

impl Decode for RawPacket {
    async fn decode<R: AsyncRead + Unpin>(read: &mut R) -> Result<Self, DecodeError> {
        // read header
        let length = VarInt::decode(read).await?;
        let id = VarInt::decode(read).await?;

        // read data
        let remaining = *length as usize - id.size();
        let mut reader = read.take(remaining as u64);
        let mut data = BytesMut::with_capacity(remaining);
        reader.read_buf(&mut data).await?;

        Ok(Self {
            id,
            data: data.freeze(),
        })
    }
}

/// Trait containing packet metadata.
pub trait PacketData {
    /// The packet ID.
    const ID: VarInt;
}

macro_rules! packets {
    (
        $state_val:expr, $packet_val:expr;
        $(
            $state:ident = {
                $($packet:ident),* $(,)?
            }
        )*
    ) => {
        match $state_val {
            $(
                ProtocolState::$state => match $packet_val.id {
                    $(
                        $packet::ID => block_on($packet::decode(&mut $packet_val.data.as_ref()))
                            .map(ServerboundPacket::from)
                    )*,
                    _ => continue,
                }
            )+,
            _ => unimplemented!()
        }
    };
}

/// Drain the raw packet receiver and insert packets into the ECS.
pub(crate) fn drain(
    mut conns: Query<(&ProtocolState, &mut PacketQueue, &RawReceiver)>,
) -> Result<()> {
    for (state, mut queue, rx) in conns.iter_mut() {
        while let Ok(packet) = rx.try_recv() {
            println!("{:?}", state);
            if let Ok(packet) = packets! {
                state, packet;

                Handshake = {
                    Handshake
                }

                Status = {
                    StatusRequest
                }
            } {
                let handshake = matches!(packet, ServerboundPacket::Handshake(_));
                queue.push(packet);

                // packets that are handshaking need to have a chance to be processed before
                // the next packet is read
                if handshake {
                    break;
                }
            }
        }
    }

    Ok(())
}
