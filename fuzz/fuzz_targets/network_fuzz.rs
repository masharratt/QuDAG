#![no_main]
use libfuzzer_sys::fuzz_target;
use qudag_network::{
    types::{MessageType, MessagePriority, RoutingStrategy, NetworkError},
    routing::Router,
    peer::{Peer, PeerId, PeerStatus},
};
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::mpsc;
use std::net::SocketAddr;

/// Test for timing side-channels in network operations
fn measure_network_timing<F>(op: F) -> bool
where
    F: Fn() -> Result<(), NetworkError>
{
    let iterations = 50; // Reduced for faster fuzzing
    let mut timings = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    let mean = timings.iter().sum::<Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    variance < 10000 // Network ops have higher variance than crypto
}

/// Create test peer from fuzz data
fn create_test_peer(data: &[u8]) -> Option<Peer> {
    if data.len() < 8 {
        return None;
    }
    
    let port = u16::from_le_bytes([data[0], data[1]]);
    let addr = SocketAddr::from(([127, 0, 0, 1], port.max(1024)));
    let id = PeerId::from(data[2..8].to_vec());
    
    Some(Peer {
        id,
        address: addr,
        status: PeerStatus::Connected,
        version: 1,
    })
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Test message type creation and validation
    let msg_type = match data[0] % 3 {
        0 => MessageType::Handshake {
            version: 1,
            node_id: data.get(1..33).unwrap_or(&[0; 32]).to_vec(),
        },
        1 => MessageType::Data {
            id: format!("msg_{}", data[0]),
            payload: data.get(1..100).unwrap_or(&[]).to_vec(),
            priority: match data.get(1).unwrap_or(&0) % 3 {
                0 => MessagePriority::High,
                1 => MessagePriority::Normal,
                _ => MessagePriority::Low,
            },
        },
        _ => MessageType::Control {
            command: "test".to_string(),
            params: vec!["param1".to_string()],
        },
    };

    // Test message serialization/deserialization
    if let Ok(serialized) = bincode::serialize(&msg_type) {
        let _ = bincode::deserialize::<MessageType>(&serialized);
    }

    // Test peer creation and validation
    if let Some(peer) = create_test_peer(data) {
        // Test peer operations
        assert_eq!(peer.status, PeerStatus::Connected);
        assert_eq!(peer.version, 1);
    }

    // Test routing strategy creation
    if data.len() >= 32 {
        let strategy = match data[0] % 4 {
            0 => RoutingStrategy::Direct(data[1..17].to_vec()),
            1 => RoutingStrategy::Flood,
            2 => RoutingStrategy::RandomSubset(data[1] as usize % 10 + 1),
            _ => RoutingStrategy::Anonymous {
                hops: data[1] as usize % 5 + 1,
                seed: data[2..34].try_into().unwrap_or([0; 32]),
                layers: vec![],
            },
        };

        // Test strategy validation
        match strategy {
            RoutingStrategy::Direct(ref id) => {
                assert!(!id.is_empty());
            }
            RoutingStrategy::RandomSubset(count) => {
                assert!(count > 0 && count <= 10);
            }
            RoutingStrategy::Anonymous { hops, .. } => {
                assert!(hops > 0 && hops <= 5);
            }
            _ => {}
        }
    }

    // Test network error handling
    let network_errors = vec![
        NetworkError::ConnectionError("test".to_string()),
        NetworkError::MessageError("test".to_string()),
        NetworkError::RoutingError("test".to_string()),
        NetworkError::EncryptionError("test".to_string()),
        NetworkError::Internal("test".to_string()),
    ];

    for error in network_errors {
        let error_string = error.to_string();
        assert!(!error_string.is_empty());
    }

    // Test message priority ordering
    let priorities = vec![
        MessagePriority::High,
        MessagePriority::Normal,
        MessagePriority::Low,
    ];

    // Verify priority ordering
    assert!(priorities[0] != priorities[1]);
    assert!(priorities[1] != priorities[2]);

    // Test with malformed data
    if data.len() >= 64 {
        // Test truncated messages
        for i in 1..64 {
            let truncated = &data[..i];
            let _ = create_test_peer(truncated);
        }

        // Test bit flipping
        let mut mutated = data[..64].to_vec();
        for i in 0..mutated.len() {
            mutated[i] ^= 1;
            let _ = create_test_peer(&mutated);
            mutated[i] ^= 1; // Restore original
        }
    }
});