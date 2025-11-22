use std::net::SocketAddr;

use crate::{QueryHandler, req::QueryRequest, res::QueryResponse};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Test state
struct State {
    handler: QueryHandler,
    addr: SocketAddr,
    session_id: i32,
}

impl State {
    /// Set up a new test state
    async fn setup() -> Result<Self> {
        let handler = QueryHandler::new().await?;
        let addr = "0.0.0.0:0".parse()?;
        let session_id: i32 = rand::random();
        Ok(Self {
            handler,
            addr,
            session_id,
        })
    }
}

/// Handshake response container
struct HandshakeResponse {
    session_id: i32,
    challenge_token: i32,
}

/// Perform a handshake
fn handshake(state: &mut State) -> Result<HandshakeResponse> {
    let request = QueryRequest::Handshake {
        session_id: state.session_id,
    };
    let response = state.handler.handle(request, state.addr)?;

    if let QueryResponse::Handshake {
        session_id,
        challenge_token,
    } = response
    {
        let token: i32 = challenge_token.to_str()?.parse()?;
        Ok(HandshakeResponse {
            session_id,
            challenge_token: token,
        })
    } else {
        Err("expected handshake response".into())
    }
}

#[tokio::test]
async fn test_handshake() -> Result<()> {
    // handshake
    let mut state = State::setup().await?;
    let res = handshake(&mut state)?;

    // verify that the session ID matches
    assert_eq!(state.session_id, res.session_id);

    // verify that the challenge token was stored correctly
    assert_eq!(
        state.handler.tokens.get(&state.addr),
        Some(&res.challenge_token)
    );

    Ok(())
}
