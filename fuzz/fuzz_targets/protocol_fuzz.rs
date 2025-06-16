use libfuzzer_sys::fuzz_target;
use qudag_protocol::{consensus::DAGConsensus, message::Message};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Fuzz consensus
    let consensus = DAGConsensus::new();
    let _ = consensus.process_message(data);

    // Fuzz DAG operations
    if data.len() >= 64 {
        let _ = consensus.process_dag_update(&data[..64]);
    }

    // Fuzz protocol messages
    if data.len() >= 128 {
        let message = Message::from_bytes(data);
        let _ = consensus.validate_message(&message);
    }
});