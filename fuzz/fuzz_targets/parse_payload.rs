#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str;

use axionvera_network_node::config::{NetworkConfig, DatabaseConfig};

fuzz_target!(|data: &[u8]| {
    // Try to deserialize as JSON into important config/payload structs
    let _ = serde_json::from_slice::<NetworkConfig>(data);
    let _ = serde_json::from_slice::<DatabaseConfig>(data);

    // Also try to interpret input as UTF-8 and parse as socket addresses
    if let Ok(s) = str::from_utf8(data) {
        let _ = s.parse::<std::net::SocketAddr>();
    }
});
