//! # beacon-macros
//!
//! Macros to help speed up the development of beacon.

use std::{collections::HashMap, sync::LazyLock};

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error2::{ResultExt, proc_macro_error};
use quote::quote;
use serde::Deserialize;
use syn::{Field, Ident, Visibility, token::Pub};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packets {
    configuration: PacketDirection,
    handshake: PacketDirection,
    login: PacketDirection,
    play: PacketDirection,
    status: PacketDirection,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PacketDirection {
    #[serde(default)]
    clientbound: HashMap<String, PacketId>,
    #[serde(default)]
    serverbound: HashMap<String, PacketId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PacketId {
    protocol_id: u8,
}

static PACKETS_JSON: LazyLock<Packets> = LazyLock::new(|| {
    let json = include_str!("../../assets/packets.json");
    serde_json::from_str(json).expect("invalid packets.json")
});

#[derive(FromMeta)]
#[darling(derive_syn_parse)]
struct PacketArgs {
    path: String,
    state: Ident,
}

/// Find the packet ID for a given path and state.
fn find_packet(path: &str, state: Ident) -> (u8, bool) {
    let direction = match state.to_string().as_str() {
        "Handshake" => &PACKETS_JSON.handshake,
        "Status" => &PACKETS_JSON.status,
        "Login" => &PACKETS_JSON.login,
        "Configuration" => &PACKETS_JSON.configuration,
        "Play" => &PACKETS_JSON.play,
        _ => panic!("invalid state: {}", state),
    };
    let resource = format!("minecraft:{path}");
    if let Some(pkt) = direction.clientbound.get(&resource) {
        (pkt.protocol_id, true)
    } else if let Some(pkt) = direction.serverbound.get(&resource) {
        (pkt.protocol_id, false)
    } else {
        panic!("packet not found: {}", resource)
    }
}

/// Mark a struct as a packet.
/// - Makes all fields public.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn packet(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(input as syn::ItemStruct);
    let PacketArgs { path, state } = syn::parse(args).unwrap_or_abort();
    let (packet_id, clientbound) = find_packet(&path, state);

    // make every field public
    item.fields
        .iter_mut()
        .for_each(|f| f.vis = Visibility::Public(Pub::default()));

    // aliases
    let decode = quote! { beacon_codec::decode };
    let varint = quote! { beacon_codec::types::VarInt };

    // encode/decode
    let name = &item.ident;

    let net_impl = if clientbound {
        // todo: need Encode
        quote! {}
    } else {
        // need Decode
        let decode_fields = item.fields.iter().map(
            |Field {
                 ident: name, ty, ..
             }| {
                quote! {
                    let #name = <#ty as #decode::Decode>::decode(read).await?;
                }
            },
        );

        let field_assignments = item.fields.iter().map(|Field { ident: name, .. }| {
            quote! { #name }
        });

        quote! {
            impl #decode::Decode for #name {
                async fn decode<R: tokio::io::AsyncRead + Unpin>(read: &mut R) -> Result<Self, #decode::DecodeError> {
                    #(#decode_fields)*

                    Ok(Self {
                        #(#field_assignments),*
                    })
                }
            }
        }
    };

    quote! {
        #[allow(missing_docs)]
        #item
        #net_impl
    }
    .into()
}
