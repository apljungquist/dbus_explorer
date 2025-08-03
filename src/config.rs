use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_addr: SocketAddr,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:2001".parse().expect("Valid socket address"),
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("DBUS_EXPLORER_ADDR") {
            if let Ok(parsed_addr) = addr.parse() {
                config.server_addr = parsed_addr;
            }
        }

        if let Ok(level) = std::env::var("DBUS_EXPLORER_LOG_LEVEL") {
            config.log_level = level;
        }

        config
    }
}
