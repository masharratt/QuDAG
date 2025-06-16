use libfuzzer_sys::fuzz_target;
use qudag_network::{message::Message, routing::AnonymousRouter};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Fuzz message handling
    let message = Message::new(data);
    let _ = message.validate();

    // Fuzz routing
    if data.len() >= 32 {
        let router = AnonymousRouter::new();
        let _ = router.process_route_data(&data[..32]);
    }

    // Fuzz network packets
    if data.len() >= 64 {
        let _ = Message::from_network_bytes(&data[..64]);
    }
});