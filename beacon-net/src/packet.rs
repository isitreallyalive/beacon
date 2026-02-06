use beacon_codec::{ProtocolState, decode::*, types::VarInt};
use bevy_ecs::prelude::*;
use bytes::{Bytes, BytesMut};
use crossbeam_channel::{Receiver, Sender};
use tokio::io::{AsyncRead, AsyncReadExt};

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

/// Receive raw packets from the networking task.
#[derive(Resource, Deref)]
pub struct RawPacketReceiver(Receiver<(Entity, RawPacket)>);

/// Send raw packets to the ECS.
pub type RawPacketSender = Sender<(Entity, RawPacket)>;

impl RawPacketReceiver {
    /// Register raw packet receiver with the ECS.
    pub fn ecs(world: &mut World, schedule: &mut Schedule) -> RawPacketSender {
        let (tx, rx) = crossbeam_channel::unbounded();
        world.insert_resource(Self(rx));
        schedule.add_systems(Self::drain);
        tx
    }

    /// Drain the raw packet receiver and insert packets into the ECS.
    fn drain(recv: Res<Self>, mut states: Query<&ProtocolState>) -> Result<()> {
        while let Ok((id, packet)) = recv.try_recv() {
            let state = states.get(id)?;
            println!("{:?} {:?}", state, packet);
        }

        Ok(())
    }
}
