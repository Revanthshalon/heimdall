#![allow(unused)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}
