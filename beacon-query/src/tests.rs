use std::{net::SocketAddr, rc::Rc};

use beacon_config::Config;
use serial_test::serial;
use tokio::sync::Mutex;

use crate::{
    GAME_TYPE, QueryHandler,
    kv::KeyValue,
    req::{QueryRequest, StatRequest},
    res::*,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Test state
struct State {
    config: Rc<Mutex<Config>>,
    handler: QueryHandler,
    addr: SocketAddr,
    session_id: i32,
}

impl State {
    /// Set up a new test state
    async fn setup() -> Result<Self> {
        let config = Rc::new(Mutex::new(Config::default()));
        let handler = QueryHandler::new(config.clone()).await?;
        let addr = "0.0.0.0:0".parse()?;
        let session_id: i32 = rand::random();
        Ok(Self {
            config,
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
async fn handshake(state: &mut State) -> Result<HandshakeResponse> {
    let request = QueryRequest::Handshake {
        session_id: state.session_id,
    };
    let response = state.handler.handle(request, state.addr).await?;

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
#[serial]
async fn test_handshake() -> Result<()> {
    // handshake
    let mut state = State::setup().await?;
    let res = handshake(&mut state).await?;

    // verify that the session ID matches
    assert_eq!(state.session_id, res.session_id);

    // verify that the challenge token was stored correctly
    assert_eq!(
        state.handler.tokens.get(&state.addr),
        Some(&res.challenge_token)
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_basic_stat() -> Result<()> {
    // basic stat
    let mut state = State::setup().await?;
    let challenge_token = handshake(&mut state).await?.challenge_token;
    let request = QueryRequest::Stat(StatRequest {
        session_id: state.session_id,
        challenge_token,
        full: false,
    });
    let response = state.handler.handle(request, state.addr).await?;

    // verify response
    if let QueryResponse::BasicStat {
        session_id,
        motd,
        game_type,
        map,
        num_players,
        max_players,
        host_port,
        host_ip,
    } = response
    {
        // verify that the session ID matches
        assert_eq!(state.session_id, session_id);

        // verify that the values match the config
        let config = state.config.lock().await;
        assert_eq!(config.server.motd, motd.to_string_lossy());
        assert_eq!(GAME_TYPE.clone(), game_type);
        assert_eq!(config.world.name, map.to_string_lossy());
        assert_eq!("0", num_players.to_string_lossy()); // todo: get from server
        assert_eq!(
            config.server.max_players.to_string(),
            max_players.to_string_lossy()
        );
        assert_eq!(config.server.port, host_port);
        assert_eq!(config.server.ip.to_string(), host_ip.to_string_lossy());
    } else {
        return Err("expected basic stat response".into());
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_full_stat() -> Result<()> {
    // full stat
    let mut state = State::setup().await?;
    let challenge_token = handshake(&mut state).await?.challenge_token;
    let request = QueryRequest::Stat(StatRequest {
        session_id: state.session_id,
        challenge_token,
        full: true,
    });
    let response = state.handler.handle(request, state.addr).await?;

    // verify response
    if let QueryResponse::FullStat {
        session_id,
        kv,
        players,
        ..
    } = response
    {
        // verify that the session ID matches
        assert_eq!(state.session_id, session_id);

        // verify that the key-value pairs match expected values
        let config = state.config.lock().await;
        assert_eq!(
            Some(config.server.motd.clone()),
            kv.get(HOSTNAME_KEY.clone())
        );
        assert_eq!(
            Some(GAME_TYPE.to_string_lossy().into_owned()),
            kv.get(GAMETYPE_KEY.clone())
        );
        assert_eq!(
            Some(GAME_ID.to_string_lossy().into_owned()),
            kv.get(GAME_ID_KEY.clone())
        );
        assert_eq!(
            Some(VERSION.to_string_lossy().into_owned()),
            kv.get(VERSION_KEY.clone())
        );
        assert_eq!(
            Some(PLUGINS.to_string_lossy().into_owned()),
            kv.get(PLUGINS_KEY.clone())
        );
        assert_eq!(Some(config.world.name.clone()), kv.get(MAP_KEY.clone()));
        assert_eq!(Some("0".to_string()), kv.get(NUMPLAYERS_KEY.clone())); // todo: get from server
        assert_eq!(
            Some(config.server.max_players.to_string()),
            kv.get(MAXPLAYERS_KEY.clone())
        );
        assert_eq!(
            Some(config.server.port.to_string()),
            kv.get(HOSTPORT_KEY.clone())
        );
        assert_eq!(
            Some(config.server.ip.to_string()),
            kv.get(HOSTIP_KEY.clone())
        );

        // verify that the players list is correct
        assert!(players.is_empty()); // todo: get from server
    } else {
        return Err("expected full stat response".into());
    }

    Ok(())
}
