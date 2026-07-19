pub mod network {
    tonic::include_proto!("axionvera.network");
}

pub mod gateway {
    tonic::include_proto!("axionvera.gateway");
}

pub mod vault {
    tonic::include_proto!("vault");
}

pub mod gateway_service;
pub mod health_service;
pub mod network_service;
pub mod p2p_service;
pub mod server;
pub mod service_registry_service;
pub mod vault_service;
