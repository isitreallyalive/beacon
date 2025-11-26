use std::net::Ipv4Addr;

const ALL_INTERFACES: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

macro_rules! define {
    (
        $(
            $(#[$struct_meta:meta])*
            struct $struct:ident {
                $(
                    $(#[$field_meta:meta])*
                    $field:ident : $ty:ty $(= $default:expr)?,
                )*
            }
        )+
    ) => {
        $(
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
            #[serde(default, rename_all = "kebab-case")]
            pub struct $struct {
                $(
                    $(#[$field_meta])*
                    pub $field: $ty,
                )*
            }

            impl Default for $struct {
                fn default() -> Self {
                    Self {
                        $(
                            $field: define!(@default $ty $(, $default)?),
                        )*
                    }
                }
            }
        )+
    };

    // helper to handle optional default values
    (@default $ty:ty, $default:expr) => {
        $default
    };
    (@default $ty:ty) => {
        <$ty>::default()
    };
}

// todo: implement all of server.properties:
// accepts-transfers=false
// allow-flight=false
// broadcast-console-to-ops=true
// broadcast-rcon-to-ops=true
// bug-report-link=
// debug=false
// difficulty=easy
// enable-code-of-conduct=false
// enable-jmx-monitoring=false
// enable-query=false
// enable-rcon=false
// enable-status=true
// enforce-secure-profile=true
// enforce-whitelist=false
// entity-broadcast-range-percentage=100
// force-gamemode=false
// function-permission-level=2
// gamemode=survival
// generate-structures=true
// generator-settings={}
// hardcore=false
// hide-online-players=false
// initial-disabled-packs=
// initial-enabled-packs=vanilla
// level-seed=
// level-type=minecraft\:normal
// log-ips=true
// management-server-enabled=false
// management-server-host=localhost
// management-server-port=0
// management-server-secret=XFK641I8yqjhxiHljVQDqQEYC4gkIvixgR2omksd
// management-server-tls-enabled=true
// management-server-tls-keystore=
// management-server-tls-keystore-password=
// max-chained-neighbor-updates=1000000
// max-tick-time=60000
// max-world-size=29999984
// network-compression-threshold=256
// online-mode=true
// op-permission-level=4
// pause-when-empty-seconds=-1
// player-idle-timeout=0
// prevent-proxy-connections=false
// rate-limit=0
// rcon.password=
// rcon.port=25575
// region-file-compression=deflate
// require-resource-pack=false
// resource-pack=
// resource-pack-id=
// resource-pack-prompt=
// resource-pack-sha1=
// simulation-distance=10
// spawn-protection=16
// status-heartbeat-interval=0
// sync-chunk-writes=true
// text-filtering-config=
// text-filtering-version=0
// use-native-transport=true
// view-distance=10
// white-list=false

define! {
    struct ServerConfig {
        /// The IP address the server listens on.
        ///
        /// By default, it listens on all interfaces.
        /// This is the recommended behaviour for most users.
        #[serde(skip_serializing)]
        ip: Ipv4Addr = ALL_INTERFACES, // server-ip

        /// The TCP port the server listens on.
        port: u16 = 25565, // server-port

        // todo: enforce limits
        /// The message of the day displayed in the server list.
        motd: String = "A Beacon Server".to_string(), // motd

        /// The maximum number of players allowed on the server at one time.
        ///
        /// Ops with `bypassPlayerLimit` enabled can join the server even when it is full.
        max_players: u32 = 20, // max-players
    }

    struct WorldConfig {
        /// The world name and directory path.
        ///
        /// If a directory at this path exists and is a valid world, it will be loaded by the server.
        /// Otherwise, a world will be generated and saved at this path.
        name: String = "world".to_string(), // level-name
    }

    struct QueryConfig {
        // todo: enable
        /// The IP address the query listener listens on.
        #[serde(skip_serializing)]
        ip: Ipv4Addr = ALL_INTERFACES,
        /// The UDP port the query listener listens on.
        port: u16 = 25565, // query.port
    }
}
