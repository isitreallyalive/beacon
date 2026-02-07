//! # beacon-macros
//!
//! Macros to help speed up the development of beacon.

use std::{collections::HashMap, sync::LazyLock};

use darling::FromMeta;
use proc_macro::{Span, TokenStream};
use proc_macro_error2::{ResultExt, proc_macro_error};
use quote::quote;
use serde::Deserialize;
use syn::{Field, Ident, VisRestricted, Visibility, token::Pub};

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
    protocol_id: i32,
}

static PACKETS_JSON: LazyLock<Packets> = LazyLock::new(|| {
    let json = include_str!("../../assets/packets.json");
    serde_json::from_str(json).expect("invalid packets.json")
});

#[derive(FromMeta)]
#[darling(derive_syn_parse)]
struct PacketArgs {
    resource: String,
    state: Ident,
}

/// Find the packet ID for a given path and state.
fn find_packet(resource: &str, state: &Ident) -> (i32, bool) {
    let direction = match state.to_string().as_str() {
        "Handshake" => &PACKETS_JSON.handshake,
        "Status" => &PACKETS_JSON.status,
        "Login" => &PACKETS_JSON.login,
        "Configuration" => &PACKETS_JSON.configuration,
        "Play" => &PACKETS_JSON.play,
        _ => panic!("invalid state: {}", state),
    };
    let locator = format!("minecraft:{resource}");
    if let Some(pkt) = direction.clientbound.get(&locator) {
        (pkt.protocol_id, true)
    } else if let Some(pkt) = direction.serverbound.get(&locator) {
        (pkt.protocol_id, false)
    } else {
        panic!("packet not found: {}", locator)
    }
}

/// Mark a struct as a packet.
/// - Makes all fields public.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn packet(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(input as syn::ItemStruct);
    let PacketArgs { resource, state } = syn::parse(args).unwrap_or_abort();
    let (packet_id, clientbound) = find_packet(&resource, &state);

    // make every field public
    item.fields
        .iter_mut()
        .for_each(|f| f.vis = Visibility::Public(Pub::default()));

    // aliases
    let decode = quote! { beacon_codec::decode };
    let encode = quote! { beacon_codec::encode };
    let raw = quote! { crate::packet::RawPacket };
    let io = quote! { tokio::io };
    let data = quote! { crate::packet::PacketData };
    let varint = quote! { beacon_codec::types::VarInt };
    let protostate = quote! { beacon_codec::ProtocolState };
    let entity = quote! { bevy_ecs::entity::Entity };

    // encode/decode
    let name = &item.ident;

    let net_impl = if clientbound {
        // need Encode
        let encode_fields = item.fields.iter().map(|Field { ident: name, .. }| {
            quote! {
                &self.#name.encode(write).await?;
            }
        });

        quote! {
            impl #encode::Encode for #name {
                async fn encode<W: #io::AsyncWrite + Unpin>(&self, write: &mut W) -> Result<(), #encode::EncodeError> {
                    #(#encode_fields)*
                    Ok(())
                }
            }

            impl #name {
                /// Turn this packet into a raw packet synchronously.
                pub fn blocking_raw(self) -> Result<#raw, #encode::EncodeError> {
                    use #encode::Encode;

                    futures::executor::block_on(async {
                        let mut buf = Vec::new();
                        self.encode(&mut buf).await?;

                        Ok(#raw {
                            id: <#name as #data>::ID,
                            data: buf.into(),
                        })
                    })
                }
            }
        }
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
                async fn decode<R: #io::AsyncRead + Unpin>(read: &mut R) -> Result<Self, #decode::DecodeError> {
                    #(#decode_fields)*

                    Ok(Self {
                        #(#field_assignments),*
                    })
                }
            }
        }
    };
    let event = Ident::new(&format!("{name}Event"), Span::call_site().into());

    quote! {
        #[allow(missing_docs)]
        #item
        #net_impl

        impl #data for #name {
            const ID: #varint = #varint(#packet_id);
            const STATE: #protostate = #protostate::#state;
        }

        impl #name {
            #[doc = concat!("Convert this packet into a [[", stringify!(#event), "]] for the given entity.") ]
            pub fn event(self, entity: #entity) -> #event {
                #event { entity, packet: self }
            }
        }

        #[doc = concat!("Event fired when a [[", stringify!(#name), "]] packet is received.") ]
        #[derive(bevy_ecs::event::EntityEvent)]
        pub(crate) struct #event {
            pub entity: #entity,
            pub packet: #name,
        }
    }
    .into()
}

/// Create a handler for a packet.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(input as syn::ItemFn);
    let packet: Ident = syn::parse(args).unwrap_or_abort();
    let call_site = Span::call_site().into();
    let event = Ident::new(&format!("{}Event", packet), call_site);

    // rename to handle
    item.sig.ident = Ident::new("handle", call_site);

    // add event parameter to function signature
    item.sig.inputs.insert(
        0,
        syn::parse_quote! { event: bevy_ecs::observer::On<#event> },
    );

    // make it public to the crate
    item.vis = Visibility::Restricted(VisRestricted {
        pub_token: Default::default(),
        paren_token: Default::default(),
        in_token: None,
        path: Box::new(syn::parse_quote! { crate }),
    });

    quote! {
        impl #packet {
            #item
        }
    }
    .into()
}
