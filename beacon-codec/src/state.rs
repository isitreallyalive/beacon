/// The protocol state of a connection, which determines which packets can be sent and received.
#[derive(Clone, Copy, Debug, Display, Default, PartialEq, Eq)]
pub enum ProtocolState {
    /// Initial state after connection - doesn't really exist in the protocol.
    /// Used in place of [None](Option::None) to avoid using [Option].
    #[default]
    Handshake,
    /// Server List Ping.
    Status,
    /// Authentication, encryption, etc.
    Login,
    /// Transitions to the [Login](ProtocolState::Login) state, but indicates that the client connected
    /// due to a Transfer packet.
    Transfer,
    /// Share resource packs, registry data, etc.
    Configuration,
    /// Playing the game.
    Play,
}
