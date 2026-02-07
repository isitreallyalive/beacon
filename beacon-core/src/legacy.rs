use beacon_data::{LATEST_SUPPORTED_VERSION, PROTOCOL_VERSION};
use bytes::{BufMut, BytesMut};
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};

pub async fn handle(
    v2: bool,
    mut writer: OwnedWriteHalf,
    motd: &str,
    online: u32,
    max_players: u32,
) {
    let payload = if v2 {
        // 1.4-1.6
        // todo: validate input
        format!(
            "§1\0{}\0{}\0{}\0{}\0{}",
            PROTOCOL_VERSION, LATEST_SUPPORTED_VERSION, motd, online, max_players
        )
    } else {
        // pre-1.4
        format!("{}§{}§{}", motd, online, max_players)
    }
    .encode_utf16()
    .flat_map(|c| c.to_be_bytes())
    .collect::<Vec<_>>();

    // respond
    let len = payload.len() as u16 / 2;
    let mut buf = BytesMut::with_capacity(3 + payload.len());
    buf.put_u8(0xFF); // kick packet
    buf.put_u16(len); // length
    buf.extend_from_slice(&payload);

    let _ = writer.write_all(&buf).await;
}
