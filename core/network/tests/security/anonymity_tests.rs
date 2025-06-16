use qudag_network::{routing::AnonymousRouter, connection::Connection, message::Message};
use test_utils::network::*;

#[cfg(test)]
mod anonymity_tests {
    use super::*;

    #[test]
    fn test_route_anonymity() {
        let network = TestNetwork::new(10); // Create test network with 10 nodes
        let router = AnonymousRouter::new();
        
        // Send message through network
        let message = Message::new(b"test message");
        let route = router.calculate_route(&network);
        
        // Verify route properties
        assert!(route.length() >= 3, "Route too short for anonymity");
        assert!(!route.contains_sequential_nodes(), "Route contains sequential nodes");
        
        // Verify message content is encrypted between hops
        for hop in route.hops() {
            assert!(hop.is_encrypted(), "Message not encrypted between hops");
        }
    }

    #[test]
    fn test_connection_security() {
        let conn = Connection::new();
        
        // Test TLS configuration
        assert!(conn.is_tls_1_3(), "Connection not using TLS 1.3");
        assert!(conn.perfect_forward_secrecy(), "Perfect forward secrecy not enabled");
        
        // Test cipher suite selection
        let ciphers = conn.cipher_suites();
        assert!(ciphers.contains(&"TLS_CHACHA20_POLY1305_SHA256"), 
            "ChaCha20-Poly1305 not available");
    }

    #[test]
    fn test_message_confidentiality() {
        let message = Message::new(b"sensitive data");
        
        // Test encryption at rest
        assert!(message.is_encrypted(), "Message not encrypted at rest");
        
        // Test proper key derivation
        assert!(message.key_derived_with_hkdf(), "Improper key derivation");
        
        // Test forward secrecy
        assert!(message.has_forward_secrecy(), "Forward secrecy not implemented");
    }
}