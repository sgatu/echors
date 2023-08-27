use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    pub bind: SocketAddr,
    pub max_connections: u16,
    pub log_config_path: String,
}
