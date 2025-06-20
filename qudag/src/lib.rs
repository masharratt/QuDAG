//! QuDAG - Quantum Ultra-Fast Distributed Acyclic Graph
//! 
//! A high-performance DAG-based distributed ledger with quantum-resistant cryptography.

pub use qudag_crypto as crypto;
pub use qudag_dag as dag;
pub use qudag_network as network;
pub use qudag_protocol as protocol;

pub mod prelude {
    pub use crate::crypto::{
        Fingerprint, FingerprintError,
        MlDsaPublicKey, MlDsaKeyPair,
        MlKem768, 
        PublicKey, SecretKey, KeyPair,
        HashFunction,
    };
    
    pub use crate::dag::{
        Node, Dag, Vertex, VertexId,
        Consensus, QRAvalanche,
    };
    
    pub use crate::network::{
        peer::Peer,
        NetworkManager,
    };
    
    pub use crate::protocol::{
        NodeConfig,
        Message,
        ProtocolConfig,
    };
}