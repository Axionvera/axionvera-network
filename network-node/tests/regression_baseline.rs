use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Mutex;

use axionvera_network_node::config::{DatabaseConfig, NetworkConfig, SorobanConfig};
use axionvera_network_node::consensus::{ConsensusEngine, Proposal, ProposalStatus, VoteType};
use axionvera_network_node::error::NetworkError;
use axionvera_network_node::p2p::P2PManager;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn soroban_config_rpc_endpoints_deduplicate_and_fallback() {
    let mut config = SorobanConfig::default();
    config.rpc_url = String::new();
    config.rpc_urls = vec![
        "https://rpc-a.example".to_string(),
        "https://rpc-a.example".to_string(),
        "https://rpc-b.example".to_string(),
    ];

    let endpoints = config.rpc_endpoints();

    assert_eq!(
        endpoints,
        vec!["https://rpc-a.example", "https://rpc-b.example"]
    );
}

#[test]
fn database_config_from_url_reports_default_settings() {
    let config = DatabaseConfig::from_url("postgres://localhost/test").unwrap();

    assert_eq!(config.min_connections, 2);
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.connection_timeout, std::time::Duration::from_secs(30));
    assert_eq!(config.idle_timeout, std::time::Duration::from_secs(300));
}

#[test]
fn network_config_from_env_reports_invalid_shutdown_period() {
    let _guard = ENV_LOCK.lock().unwrap();
    let original = env::var("SHUTDOWN_GRACE_PERIOD").ok();
    env::set_var("SHUTDOWN_GRACE_PERIOD", "not-a-number");

    let error = NetworkConfig::from_env().unwrap_err();

    assert!(matches!(
        error,
        NetworkError::Config(message) if message.contains("SHUTDOWN_GRACE_PERIOD")
    ));

    match original {
        Some(value) => env::set_var("SHUTDOWN_GRACE_PERIOD", value),
        None => env::remove_var("SHUTDOWN_GRACE_PERIOD"),
    }
}

#[tokio::test]
async fn peer_connect_disconnect_and_broadcast_flow_is_consistent() {
    let manager = P2PManager::new([7u8; 32]);
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000);

    let session_id = manager.connect_to_peer(address, "peer-a".to_string()).await.unwrap();
    assert!(session_id.starts_with("session_"));

    let peer_list = manager.get_peer_list().await;
    assert_eq!(peer_list.len(), 1);
    assert_eq!(peer_list[0].id, "peer-a");
    assert!(peer_list[0].is_connected);

    let (recipients, failures) = manager
        .broadcast_message(42, b"hello", &["peer-a".to_string()], 1)
        .await
        .unwrap();

    assert_eq!(recipients, 1);
    assert!(failures.is_empty());

    manager.disconnect_from_peer("peer-a").await.unwrap();
    assert_eq!(manager.get_connected_peers_count().await, 0);
}

#[tokio::test]
async fn broadcast_reports_unconnected_targets() {
    let manager = P2PManager::new([8u8; 32]);
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9001);

    manager.connect_to_peer(address, "peer-a".to_string()).await.unwrap();

    let (recipients, failures) = manager
        .broadcast_message(
            11,
            b"payload",
            &["peer-a".to_string(), "peer-b".to_string()],
            2,
        )
        .await
        .unwrap();

    assert_eq!(recipients, 1);
    assert_eq!(failures, vec!["peer-b"]);
}

#[tokio::test]
async fn consensus_engine_finalizes_a_proposal_when_quorum_is_reached() {
    let (engine, _vote_rx, _proposal_rx) = ConsensusEngine::new("node-a".to_string(), 2, 5);

    let proposal = engine.create_proposal(b"upgrade".to_vec()).await.unwrap();
    let proposal_id = proposal.id.clone();

    engine
        .vote(&proposal_id, VoteType::Approve, vec![1, 2, 3])
        .await
        .unwrap();

    let vote = axionvera_network_node::consensus::Vote::new(
        proposal_id.clone(),
        "node-b".to_string(),
        VoteType::Approve,
        vec![4, 5, 6],
    );
    engine.process_vote(vote).await.unwrap();

    let finalized = engine.get_proposal(&proposal_id).await.unwrap();
    assert!(matches!(finalized.status, ProposalStatus::Approved));
    assert_eq!(finalized.current_votes, 2);
}

#[tokio::test]
async fn duplicate_votes_are_rejected_and_expired_proposals_are_cleaned_up() {
    let (engine, _vote_rx, _proposal_rx) = ConsensusEngine::new("node-a".to_string(), 2, 1);

    let proposal = engine.create_proposal(b"cleanup".to_vec()).await.unwrap();
    engine
        .vote(&proposal.id, VoteType::Approve, vec![1, 2, 3])
        .await
        .unwrap();

    let duplicate_error = engine
        .vote(&proposal.id, VoteType::Approve, vec![4, 5, 6])
        .await
        .unwrap_err();
    assert!(matches!(
        duplicate_error,
        NetworkError::Validation(message) if message.contains("Already voted")
    ));

    let mut expired_proposal = Proposal::new("node-a".to_string(), b"expired".to_vec(), 1, 0);
    expired_proposal.expires_at = chrono::Utc::now() - chrono::Duration::minutes(1);
    expired_proposal.status = ProposalStatus::Active;
    engine.process_proposal(expired_proposal.clone()).await.unwrap();

    let cleaned = engine.cleanup_expired_proposals().await.unwrap();
    assert_eq!(cleaned, 1);

    let stored = engine.get_proposal(&expired_proposal.id).await.unwrap();
    assert!(matches!(stored.status, ProposalStatus::Expired));
}
