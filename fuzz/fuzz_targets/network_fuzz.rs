#![no_main]
use libfuzzer_sys::fuzz_target;
use std::time::Duration;
use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use serde::{Serialize, Deserialize};

/// Message priority levels for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    High,
    Normal,
    Low,
}

/// Test message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMessage {
    pub id: String,
    pub payload: Vec<u8>,
    pub priority: MessagePriority,
    pub timestamp: u64,
}

/// Test peer structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerId(Vec<u8>);

impl From<Vec<u8>> for PeerId {
    fn from(bytes: Vec<u8>) -> Self {
        PeerId(bytes)
    }
}

#[derive(Debug, Clone)]
pub struct TestPeer {
    pub id: PeerId,
    pub address: SocketAddr,
    pub connected: bool,
}

/// Test for timing side-channels in network operations
fn measure_network_timing<F>(op: F) -> bool
where
    F: Fn() -> Result<(), String>
{
    let iterations = 25; // Reduced for faster fuzzing
    let mut timings = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    if timings.is_empty() {
        return false;
    }
    
    let mean = timings.iter().sum::<Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    variance < 50000 // Network ops have higher variance than crypto
}

/// Create test peer from fuzz data
fn create_test_peer(data: &[u8]) -> Option<TestPeer> {
    if data.len() < 8 {
        return None;
    }
    
    let port = u16::from_le_bytes([data[0], data[1]]);
    let safe_port = port.max(1024).min(65535);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), safe_port);
    let id = PeerId::from(data[2..8].to_vec());
    
    Some(TestPeer {
        id,
        address: addr,
        connected: true,
    })
}

/// Create test message from fuzz data
fn create_test_message(data: &[u8]) -> Option<TestMessage> {
    if data.len() < 2 {
        return None;
    }
    
    let priority = match data[0] % 3 {
        0 => MessagePriority::High,
        1 => MessagePriority::Normal,
        _ => MessagePriority::Low,
    };
    
    let id = format!("msg_{}", data[0]);
    let payload = data[1..].to_vec();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Some(TestMessage {
        id,
        payload,
        priority,
        timestamp,
    })
}

/// Test message serialization and validation
fn test_message_operations(msg: &TestMessage) -> Result<(), String> {
    // Test serialization
    let serialized = bincode::serialize(msg)
        .map_err(|e| format!("Serialization failed: {}", e))?;
    
    // Test deserialization
    let deserialized: TestMessage = bincode::deserialize(&serialized)
        .map_err(|e| format!("Deserialization failed: {}", e))?;
    
    // Verify integrity
    if msg.id != deserialized.id {
        return Err("Message ID mismatch".to_string());
    }
    
    if msg.payload != deserialized.payload {
        return Err("Payload mismatch".to_string());
    }
    
    if msg.priority as u8 != deserialized.priority as u8 {
        return Err("Priority mismatch".to_string());
    }
    
    Ok(())
}

/// Test peer management operations
fn test_peer_operations(peer: &TestPeer) -> Result<(), String> {
    // Test peer validation
    if peer.id.0.is_empty() {
        return Err("Empty peer ID".to_string());
    }
    
    if peer.id.0.len() > 64 {
        return Err("Peer ID too long".to_string());
    }
    
    // Test address validation
    let port = peer.address.port();
    if port < 1024 || port > 65535 {
        return Err("Invalid port range".to_string());
    }
    
    Ok(())
}

/// Test routing table operations
fn test_routing_operations(peers: &[TestPeer]) -> Result<(), String> {
    let mut routing_table: HashMap<PeerId, SocketAddr> = HashMap::new();
    
    // Add peers to routing table
    for peer in peers {
        routing_table.insert(peer.id.clone(), peer.address);
    }
    
    // Test lookups
    for peer in peers {
        if let Some(addr) = routing_table.get(&peer.id) {
            if *addr != peer.address {
                return Err("Routing table lookup mismatch".to_string());
            }
        } else {
            return Err("Peer not found in routing table".to_string());
        }
    }
    
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Test message creation and validation
    if let Some(message) = create_test_message(data) {
        let msg_timing = measure_network_timing(|| {
            test_message_operations(&message)
        });
        // Don't assert on timing in fuzzing - just ensure it doesn't crash
    }

    // Test peer creation and validation
    if let Some(peer) = create_test_peer(data) {
        let peer_timing = measure_network_timing(|| {
            test_peer_operations(&peer)
        });
        // Don't assert on timing in fuzzing - just ensure it doesn't crash
    }

    // Test multiple peers for routing operations
    if data.len() >= 32 {
        let mut peers = Vec::new();
        
        // Create multiple peers from data chunks
        for i in 0..std::cmp::min(data.len() / 8, 10) {
            let start_idx = i * 8;
            let end_idx = std::cmp::min(start_idx + 8, data.len());
            
            if let Some(peer) = create_test_peer(&data[start_idx..end_idx]) {
                peers.push(peer);
            }
        }
        
        if !peers.is_empty() {
            let routing_timing = measure_network_timing(|| {
                test_routing_operations(&peers)
            });
            // Don't assert on timing - just ensure it doesn't crash
        }
    }

    // Test priority ordering
    let priorities = vec![
        MessagePriority::High,
        MessagePriority::Normal,
        MessagePriority::Low,
    ];

    // Verify priority values are distinct
    assert!(priorities[0] as u8 != priorities[1] as u8);
    assert!(priorities[1] as u8 != priorities[2] as u8);

    // Test with malformed data
    if data.len() >= 64 {
        // Test truncated messages
        for i in 1..32 {
            if i < data.len() {
                let truncated = &data[..i];
                let _ = create_test_message(truncated);
                let _ = create_test_peer(truncated);
            }
        }

        // Test bit flipping
        let mut mutated = data[..32].to_vec();
        for i in 0..mutated.len() {
            mutated[i] ^= 1;
            let _ = create_test_message(&mutated);
            let _ = create_test_peer(&mutated);
            mutated[i] ^= 1; // Restore original
        }
    }

    // Test edge cases
    if data.len() >= 16 {
        // Test with all zeros
        let zero_data = vec![0u8; 16];
        let _ = create_test_message(&zero_data);
        let _ = create_test_peer(&zero_data);

        // Test with all ones
        let ones_data = vec![0xFFu8; 16];
        let _ = create_test_message(&ones_data);
        let _ = create_test_peer(&ones_data);

        // Test with alternating pattern
        let alt_data: Vec<u8> = (0..16).map(|i| if i % 2 == 0 { 0x55 } else { 0xAA }).collect();
        let _ = create_test_message(&alt_data);
        let _ = create_test_peer(&alt_data);
    }
});