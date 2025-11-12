//! Endpoints are accessible at `minecraft:serversettings`
//!
//! https://minecraft.wiki/w/Minecraft_Server_Management_Protocol#Server_Settings

// todo: implement server settings
// - autosave
// - difficulty
// - enforce_allowlist
// - use_allowlist
// - max_players
// - pause_when_empty_seconds
// - player_idle_timeout
// - allow_flight
// - motd
// - spawn_protection_radius
// - force_game_mode
// - game_mode
// - view_distance
// - simulation_distance
// - accept_transfers
// - status_heartbeat_interval
// - operator_user_permission_level
// - hide_online_players
// - status_replies
// - entity_broadcast_range

macro_rules! setting {
    ($name:expr) => {
        $crate::method!(concat!("serversettings/", $name));
        $crate::method!(concat!("serversettings/", $name, "/set"));
    };
}

setting!("autosave");
setting!("difficulty");
setting!("enforce_allowlist");
setting!("use_allowlist");
setting!("max_players");
setting!("pause_when_empty_seconds");
setting!("player_idle_timeout");
setting!("allow_flight");
setting!("motd");
setting!("spawn_protection_radius");
setting!("force_game_mode");
setting!("game_mode");
setting!("view_distance");
setting!("simulation_distance");
setting!("accept_transfers");
setting!("status_heartbeat_interval");
setting!("operator_user_permission_level");
setting!("hide_online_players");
setting!("status_replies");
setting!("entity_broadcast_range");
