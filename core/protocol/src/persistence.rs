//! State persistence implementation for QuDAG node
//!
//! This module provides persistence backends for saving and restoring node state,
//! including peer lists, DAG state, and other critical data that should survive
//! node restarts.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::state::{ProtocolState, SessionInfo, StateMachineMetrics};
use qudag_dag::vertex::{Vertex, VertexId};

/// Errors that can occur during persistence operations
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// IO error during read/write operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Database error (for RocksDB/SQLite backends)
    #[error("Database error: {0}")]
    Database(String),

    /// State validation error
    #[error("State validation error: {0}")]
    Validation(String),

    /// State version mismatch
    #[error("State version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },

    /// Corruption detected in persisted state
    #[error("Corrupted state detected: {0}")]
    Corruption(String),

    /// Backup/restore operation failed
    #[error("Backup/restore failed: {0}")]
    BackupRestore(String),
}

/// State version for migration support
pub const CURRENT_STATE_VERSION: u32 = 1;

/// Persisted peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedPeer {
    /// Peer identifier (stored as bytes for persistence)
    pub id: Vec<u8>,
    /// Peer address
    pub address: String,
    /// Reputation score (0-100)
    pub reputation: u32,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Connection statistics
    pub stats: PeerStats,
    /// Whether peer is blacklisted
    pub blacklisted: bool,
    /// Whether peer is whitelisted
    pub whitelisted: bool,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Peer connection statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeerStats {
    /// Total connections
    pub total_connections: u64,
    /// Successful connections
    pub successful_connections: u64,
    /// Failed connections
    pub failed_connections: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Average response time in milliseconds
    pub avg_response_time: u64,
}

/// Persisted DAG state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDagState {
    /// DAG vertices
    pub vertices: HashMap<VertexId, Vertex>,
    /// Current tips
    pub tips: HashSet<VertexId>,
    /// Consensus voting records
    pub voting_records: HashMap<VertexId, VotingRecord>,
    /// Last checkpoint
    pub last_checkpoint: Option<Checkpoint>,
}

/// Voting record for consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingRecord {
    /// Vertex ID
    pub vertex_id: VertexId,
    /// Yes votes
    pub yes_votes: u32,
    /// No votes
    pub no_votes: u32,
    /// Voting round
    pub round: u32,
    /// Finalized status
    pub finalized: bool,
}

/// DAG checkpoint for fast recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint ID
    pub id: Uuid,
    /// Checkpoint height
    pub height: u64,
    /// Checkpoint timestamp
    pub timestamp: u64,
    /// Checkpoint hash
    pub hash: Vec<u8>,
    /// Included vertices
    pub vertices: HashSet<VertexId>,
}

/// Complete persisted state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    /// State version
    pub version: u32,
    /// Node ID
    pub node_id: Vec<u8>,
    /// Protocol state
    pub protocol_state: ProtocolState,
    /// Active sessions
    pub sessions: HashMap<Uuid, SessionInfo>,
    /// Peer list
    pub peers: Vec<PersistedPeer>,
    /// DAG state
    pub dag_state: PersistedDagState,
    /// Node metrics
    pub metrics: StateMachineMetrics,
    /// Last save timestamp
    pub last_saved: u64,
}

/// Trait for state persistence backends
#[async_trait]
pub trait StatePersistence: Send + Sync {
    /// Save complete state
    async fn save_state(&self, state: &PersistedState) -> Result<(), PersistenceError>;

    /// Load complete state
    async fn load_state(&self) -> Result<Option<PersistedState>, PersistenceError>;

    /// Save peer list
    async fn save_peers(&self, peers: &[PersistedPeer]) -> Result<(), PersistenceError>;

    /// Load peer list
    async fn load_peers(&self) -> Result<Vec<PersistedPeer>, PersistenceError>;

    /// Save DAG state
    async fn save_dag_state(&self, dag_state: &PersistedDagState) -> Result<(), PersistenceError>;

    /// Load DAG state
    async fn load_dag_state(&self) -> Result<Option<PersistedDagState>, PersistenceError>;

    /// Create backup
    async fn create_backup(&self, backup_path: &Path) -> Result<(), PersistenceError>;

    /// Restore from backup
    async fn restore_backup(&self, backup_path: &Path) -> Result<(), PersistenceError>;

    /// Prune old data
    async fn prune_old_data(&self, before_timestamp: u64) -> Result<u64, PersistenceError>;

    /// Validate persisted state
    async fn validate_state(&self) -> Result<bool, PersistenceError>;

    /// Get backend type name
    fn backend_type(&self) -> &'static str;
}

/// RocksDB persistence backend for production use
#[cfg(feature = "rocksdb")]
pub struct RocksDbBackend {
    db: Arc<rocksdb::DB>,
    path: PathBuf,
}

#[cfg(feature = "rocksdb")]
impl RocksDbBackend {
    /// Create new RocksDB backend
    pub fn new(path: PathBuf) -> Result<Self, PersistenceError> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        let db = rocksdb::DB::open(&opts, &path)
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
            path,
        })
    }

    fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, PersistenceError> {
        bincode::serialize(value)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, PersistenceError> {
        bincode::deserialize(bytes)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))
    }
}

#[cfg(feature = "rocksdb")]
#[async_trait]
impl StatePersistence for RocksDbBackend {
    async fn save_state(&self, state: &PersistedState) -> Result<(), PersistenceError> {
        let key = b"state";
        let value = Self::serialize(state)?;
        
        self.db.put(key, value)
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        // Also save individual components for faster access
        self.save_peers(&state.peers).await?;
        self.save_dag_state(&state.dag_state).await?;
        
        info!("State saved to RocksDB");
        Ok(())
    }

    async fn load_state(&self) -> Result<Option<PersistedState>, PersistenceError> {
        let key = b"state";
        
        match self.db.get(key).map_err(|e| PersistenceError::Database(e.to_string()))? {
            Some(bytes) => {
                let state: PersistedState = Self::deserialize(&bytes)?;
                
                // Validate version
                if state.version != CURRENT_STATE_VERSION {
                    return Err(PersistenceError::VersionMismatch {
                        expected: CURRENT_STATE_VERSION,
                        actual: state.version,
                    });
                }
                
                info!("State loaded from RocksDB");
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    async fn save_peers(&self, peers: &[PersistedPeer]) -> Result<(), PersistenceError> {
        let key = b"peers";
        let value = Self::serialize(peers)?;
        
        self.db.put(key, value)
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        debug!("Saved {} peers to RocksDB", peers.len());
        Ok(())
    }

    async fn load_peers(&self) -> Result<Vec<PersistedPeer>, PersistenceError> {
        let key = b"peers";
        
        match self.db.get(key).map_err(|e| PersistenceError::Database(e.to_string()))? {
            Some(bytes) => {
                let peers: Vec<PersistedPeer> = Self::deserialize(&bytes)?;
                debug!("Loaded {} peers from RocksDB", peers.len());
                Ok(peers)
            }
            None => Ok(Vec::new()),
        }
    }

    async fn save_dag_state(&self, dag_state: &PersistedDagState) -> Result<(), PersistenceError> {
        let key = b"dag_state";
        let value = Self::serialize(dag_state)?;
        
        self.db.put(key, value)
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        debug!("Saved DAG state with {} vertices", dag_state.vertices.len());
        Ok(())
    }

    async fn load_dag_state(&self) -> Result<Option<PersistedDagState>, PersistenceError> {
        let key = b"dag_state";
        
        match self.db.get(key).map_err(|e| PersistenceError::Database(e.to_string()))? {
            Some(bytes) => {
                let dag_state: PersistedDagState = Self::deserialize(&bytes)?;
                debug!("Loaded DAG state with {} vertices", dag_state.vertices.len());
                Ok(Some(dag_state))
            }
            None => Ok(None),
        }
    }

    async fn create_backup(&self, backup_path: &Path) -> Result<(), PersistenceError> {
        let backup_engine = rocksdb::backup::BackupEngine::open(
            &rocksdb::backup::BackupEngineOptions::default(),
            backup_path,
        ).map_err(|e| PersistenceError::BackupRestore(e.to_string()))?;
        
        backup_engine.create_new_backup(&self.db)
            .map_err(|e| PersistenceError::BackupRestore(e.to_string()))?;
        
        info!("Backup created at {:?}", backup_path);
        Ok(())
    }

    async fn restore_backup(&self, backup_path: &Path) -> Result<(), PersistenceError> {
        let backup_engine = rocksdb::backup::BackupEngine::open(
            &rocksdb::backup::BackupEngineOptions::default(),
            backup_path,
        ).map_err(|e| PersistenceError::BackupRestore(e.to_string()))?;
        
        let restore_opts = rocksdb::backup::RestoreOptions::default();
        backup_engine.restore_from_latest_backup(&self.path, &self.path, &restore_opts)
            .map_err(|e| PersistenceError::BackupRestore(e.to_string()))?;
        
        info!("Backup restored from {:?}", backup_path);
        Ok(())
    }

    async fn prune_old_data(&self, before_timestamp: u64) -> Result<u64, PersistenceError> {
        // For RocksDB, we would iterate through entries and delete old ones
        // This is a simplified implementation
        warn!("Pruning not fully implemented for RocksDB backend");
        Ok(0)
    }

    async fn validate_state(&self) -> Result<bool, PersistenceError> {
        // Validate state integrity
        if let Some(state) = self.load_state().await? {
            // Check version
            if state.version != CURRENT_STATE_VERSION {
                return Ok(false);
            }
            
            // Basic validation checks
            if state.node_id.is_empty() {
                return Ok(false);
            }
            
            Ok(true)
        } else {
            Ok(true) // No state is valid
        }
    }

    fn backend_type(&self) -> &'static str {
        "RocksDB"
    }
}

/// SQLite persistence backend for lightweight deployments
pub struct SqliteBackend {
    pool: Arc<RwLock<sqlx::SqlitePool>>,
    path: PathBuf,
}

impl SqliteBackend {
    /// Create new SQLite backend
    pub async fn new(path: PathBuf) -> Result<Self, PersistenceError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Ensure path is absolute
        let abs_path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        };
        
        let db_url = format!("sqlite:{}", abs_path.display());
        
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS state (
                id INTEGER PRIMARY KEY,
                data BLOB NOT NULL,
                timestamp INTEGER NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS peers (
                id TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                timestamp INTEGER NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS dag_state (
                id INTEGER PRIMARY KEY,
                data BLOB NOT NULL,
                timestamp INTEGER NOT NULL
            );
            "#
        )
        .execute(&pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        Ok(Self {
            pool: Arc::new(RwLock::new(pool)),
            path: abs_path,
        })
    }

    fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, PersistenceError> {
        bincode::serialize(value)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, PersistenceError> {
        bincode::deserialize(bytes)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))
    }
}

#[async_trait]
impl StatePersistence for SqliteBackend {
    async fn save_state(&self, state: &PersistedState) -> Result<(), PersistenceError> {
        let pool = self.pool.read().await;
        let data = Self::serialize(state)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        sqlx::query(
            "INSERT OR REPLACE INTO state (id, data, timestamp) VALUES (1, ?, ?)"
        )
        .bind(&data[..])
        .bind(timestamp)
        .execute(&*pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        info!("State saved to SQLite");
        Ok(())
    }

    async fn load_state(&self) -> Result<Option<PersistedState>, PersistenceError> {
        let pool = self.pool.read().await;
        
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM state WHERE id = 1"
        )
        .fetch_optional(&*pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        match row {
            Some((data,)) => {
                let state: PersistedState = Self::deserialize(&data)?;
                
                // Validate version
                if state.version != CURRENT_STATE_VERSION {
                    return Err(PersistenceError::VersionMismatch {
                        expected: CURRENT_STATE_VERSION,
                        actual: state.version,
                    });
                }
                
                info!("State loaded from SQLite");
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    async fn save_peers(&self, peers: &[PersistedPeer]) -> Result<(), PersistenceError> {
        let pool = self.pool.read().await;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // Use transaction for batch insert
        let mut tx = pool.begin()
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        for peer in peers {
            let data = Self::serialize(peer)?;
            let id = hex::encode(&peer.id);
            
            sqlx::query(
                "INSERT OR REPLACE INTO peers (id, data, timestamp) VALUES (?, ?, ?)"
            )
            .bind(&id)
            .bind(&data[..])
            .bind(timestamp)
            .execute(&mut *tx)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        }
        
        tx.commit()
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        debug!("Saved {} peers to SQLite", peers.len());
        Ok(())
    }

    async fn load_peers(&self) -> Result<Vec<PersistedPeer>, PersistenceError> {
        let pool = self.pool.read().await;
        
        let rows: Vec<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM peers"
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        let mut peers = Vec::new();
        for (data,) in rows {
            let peer: PersistedPeer = Self::deserialize(&data)?;
            peers.push(peer);
        }
        
        debug!("Loaded {} peers from SQLite", peers.len());
        Ok(peers)
    }

    async fn save_dag_state(&self, dag_state: &PersistedDagState) -> Result<(), PersistenceError> {
        let pool = self.pool.read().await;
        let data = Self::serialize(dag_state)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        sqlx::query(
            "INSERT OR REPLACE INTO dag_state (id, data, timestamp) VALUES (1, ?, ?)"
        )
        .bind(&data[..])
        .bind(timestamp)
        .execute(&*pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        debug!("Saved DAG state with {} vertices", dag_state.vertices.len());
        Ok(())
    }

    async fn load_dag_state(&self) -> Result<Option<PersistedDagState>, PersistenceError> {
        let pool = self.pool.read().await;
        
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM dag_state WHERE id = 1"
        )
        .fetch_optional(&*pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        match row {
            Some((data,)) => {
                let dag_state: PersistedDagState = Self::deserialize(&data)?;
                debug!("Loaded DAG state with {} vertices", dag_state.vertices.len());
                Ok(Some(dag_state))
            }
            None => Ok(None),
        }
    }

    async fn create_backup(&self, backup_path: &Path) -> Result<(), PersistenceError> {
        let pool = self.pool.read().await;
        
        // SQLite backup using VACUUM INTO
        let backup_db = backup_path.join("backup.db");
        let query = format!("VACUUM INTO '{}'", backup_db.display());
        
        sqlx::query(&query)
            .execute(&*pool)
            .await
            .map_err(|e| PersistenceError::BackupRestore(e.to_string()))?;
        
        info!("Backup created at {:?}", backup_path);
        Ok(())
    }

    async fn restore_backup(&self, backup_path: &Path) -> Result<(), PersistenceError> {
        // For SQLite, we would copy the backup file
        let backup_db = backup_path.join("backup.db");
        
        if !backup_db.exists() {
            return Err(PersistenceError::BackupRestore(
                "Backup file not found".to_string()
            ));
        }
        
        // Close current connection and replace file
        // This is simplified - in production we'd handle this more carefully
        std::fs::copy(&backup_db, &self.path)
            .map_err(|e| PersistenceError::BackupRestore(e.to_string()))?;
        
        info!("Backup restored from {:?}", backup_path);
        Ok(())
    }

    async fn prune_old_data(&self, before_timestamp: u64) -> Result<u64, PersistenceError> {
        let pool = self.pool.read().await;
        
        // Delete old peer entries
        let result = sqlx::query(
            "DELETE FROM peers WHERE timestamp < ?"
        )
        .bind(before_timestamp as i64)
        .execute(&*pool)
        .await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        let pruned = result.rows_affected();
        debug!("Pruned {} old entries", pruned);
        Ok(pruned)
    }

    async fn validate_state(&self) -> Result<bool, PersistenceError> {
        let pool = self.pool.read().await;
        
        // Check database integrity
        let result: (String,) = sqlx::query_as("PRAGMA integrity_check")
            .fetch_one(&*pool)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        if result.0 != "ok" {
            return Err(PersistenceError::Corruption(result.0));
        }
        
        // Validate stored state
        if let Some(state) = self.load_state().await? {
            if state.version != CURRENT_STATE_VERSION {
                return Ok(false);
            }
            
            if state.node_id.is_empty() {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    fn backend_type(&self) -> &'static str {
        "SQLite"
    }
}

/// In-memory persistence backend for testing
pub struct MemoryBackend {
    state: Arc<RwLock<Option<PersistedState>>>,
    peers: Arc<RwLock<Vec<PersistedPeer>>>,
    dag_state: Arc<RwLock<Option<PersistedDagState>>>,
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self {
            state: Arc::new(RwLock::new(None)),
            peers: Arc::new(RwLock::new(Vec::new())),
            dag_state: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl StatePersistence for MemoryBackend {
    async fn save_state(&self, state: &PersistedState) -> Result<(), PersistenceError> {
        let mut stored_state = self.state.write().await;
        *stored_state = Some(state.clone());
        
        // Also save individual components
        let mut peers = self.peers.write().await;
        *peers = state.peers.clone();
        
        let mut dag_state = self.dag_state.write().await;
        *dag_state = Some(state.dag_state.clone());
        
        debug!("State saved to memory");
        Ok(())
    }

    async fn load_state(&self) -> Result<Option<PersistedState>, PersistenceError> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    async fn save_peers(&self, peers_list: &[PersistedPeer]) -> Result<(), PersistenceError> {
        let mut peers = self.peers.write().await;
        *peers = peers_list.to_vec();
        debug!("Saved {} peers to memory", peers_list.len());
        Ok(())
    }

    async fn load_peers(&self) -> Result<Vec<PersistedPeer>, PersistenceError> {
        let peers = self.peers.read().await;
        Ok(peers.clone())
    }

    async fn save_dag_state(&self, new_dag_state: &PersistedDagState) -> Result<(), PersistenceError> {
        let mut dag_state = self.dag_state.write().await;
        *dag_state = Some(new_dag_state.clone());
        debug!("Saved DAG state with {} vertices to memory", new_dag_state.vertices.len());
        Ok(())
    }

    async fn load_dag_state(&self) -> Result<Option<PersistedDagState>, PersistenceError> {
        let dag_state = self.dag_state.read().await;
        Ok(dag_state.clone())
    }

    async fn create_backup(&self, backup_path: &Path) -> Result<(), PersistenceError> {
        let state = self.state.read().await;
        if let Some(state) = &*state {
            let backup_file = backup_path.join("memory_backup.bin");
            let data = bincode::serialize(state)
                .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
            tokio::fs::write(&backup_file, data).await?;
            info!("Memory backup created at {:?}", backup_file);
        }
        Ok(())
    }

    async fn restore_backup(&self, backup_path: &Path) -> Result<(), PersistenceError> {
        let backup_file = backup_path.join("memory_backup.bin");
        let data = tokio::fs::read(&backup_file).await?;
        let state: PersistedState = bincode::deserialize(&data)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
        
        let mut stored_state = self.state.write().await;
        *stored_state = Some(state);
        
        info!("Memory backup restored from {:?}", backup_file);
        Ok(())
    }

    async fn prune_old_data(&self, _before_timestamp: u64) -> Result<u64, PersistenceError> {
        // No-op for memory backend
        Ok(0)
    }

    async fn validate_state(&self) -> Result<bool, PersistenceError> {
        let state = self.state.read().await;
        if let Some(state) = &*state {
            if state.version != CURRENT_STATE_VERSION {
                return Ok(false);
            }
            if state.node_id.is_empty() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn backend_type(&self) -> &'static str {
        "Memory"
    }
}

/// State persistence manager that handles state saving and recovery
pub struct PersistenceManager {
    pub backend: Arc<dyn StatePersistence>,
    auto_save_interval: Option<tokio::time::Duration>,
    compression_enabled: bool,
}

impl PersistenceManager {
    /// Create new persistence manager with specified backend
    pub fn new(backend: Arc<dyn StatePersistence>) -> Self {
        Self {
            backend,
            auto_save_interval: Some(tokio::time::Duration::from_secs(300)), // 5 minutes
            compression_enabled: true,
        }
    }

    /// Set auto-save interval
    pub fn set_auto_save_interval(&mut self, interval: Option<tokio::time::Duration>) {
        self.auto_save_interval = interval;
    }

    /// Enable/disable compression
    pub fn set_compression(&mut self, enabled: bool) {
        self.compression_enabled = enabled;
    }

    /// Start auto-save task
    pub fn start_auto_save(&self, state_provider: Arc<dyn StateProvider>) {
        if let Some(interval) = self.auto_save_interval {
            let backend = self.backend.clone();
            
            tokio::spawn(async move {
                let mut interval_timer = tokio::time::interval(interval);
                
                loop {
                    interval_timer.tick().await;
                    
                    match state_provider.get_current_state().await {
                        Ok(state) => {
                            if let Err(e) = backend.save_state(&state).await {
                                error!("Auto-save failed: {}", e);
                            } else {
                                debug!("Auto-save completed");
                            }
                        }
                        Err(e) => {
                            error!("Failed to get current state for auto-save: {}", e);
                        }
                    }
                }
            });
        }
    }

    /// Perform state recovery on startup
    pub async fn recover_state(&self) -> Result<Option<PersistedState>, PersistenceError> {
        info!("Starting state recovery from {} backend", self.backend.backend_type());
        
        // Validate existing state
        if !self.backend.validate_state().await? {
            warn!("State validation failed, attempting recovery");
            
            // Try to load anyway and fix what we can
            if let Some(mut state) = self.backend.load_state().await? {
                // Fix version if needed
                if state.version != CURRENT_STATE_VERSION {
                    warn!("Migrating state from version {} to {}", 
                          state.version, CURRENT_STATE_VERSION);
                    state = self.migrate_state(state)?;
                }
                
                // Re-save corrected state
                self.backend.save_state(&state).await?;
                return Ok(Some(state));
            }
        }
        
        // Load normal state
        self.backend.load_state().await
    }

    /// Migrate state from old version to current
    fn migrate_state(&self, mut state: PersistedState) -> Result<PersistedState, PersistenceError> {
        // Implement version-specific migrations
        match state.version {
            0 => {
                // Migration from version 0 to 1
                warn!("Migrating from version 0 to 1");
                state.version = 1;
                // Add any new fields with defaults
            }
            _ => {
                return Err(PersistenceError::VersionMismatch {
                    expected: CURRENT_STATE_VERSION,
                    actual: state.version,
                });
            }
        }
        
        Ok(state)
    }

    /// Export state to file
    pub async fn export_state(&self, export_path: &Path) -> Result<(), PersistenceError> {
        if let Some(state) = self.backend.load_state().await? {
            let json = serde_json::to_string_pretty(&state)
                .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
            
            tokio::fs::write(export_path, json).await?;
            info!("State exported to {:?}", export_path);
        }
        
        Ok(())
    }

    /// Import state from file
    pub async fn import_state(&self, import_path: &Path) -> Result<(), PersistenceError> {
        let json = tokio::fs::read_to_string(import_path).await?;
        let state: PersistedState = serde_json::from_str(&json)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
        
        // Validate imported state
        if state.version != CURRENT_STATE_VERSION {
            return Err(PersistenceError::VersionMismatch {
                expected: CURRENT_STATE_VERSION,
                actual: state.version,
            });
        }
        
        self.backend.save_state(&state).await?;
        info!("State imported from {:?}", import_path);
        
        Ok(())
    }
}

/// Trait for providing current state to persistence manager
#[async_trait]
pub trait StateProvider: Send + Sync {
    /// Get current state for persistence
    async fn get_current_state(&self) -> Result<PersistedState, PersistenceError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_backend() {
        let backend = MemoryBackend::default();
        
        // Test save and load
        let state = create_test_state();
        backend.save_state(&state).await.unwrap();
        
        let loaded = backend.load_state().await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().node_id, state.node_id);
    }

    #[tokio::test]
    async fn test_sqlite_backend() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let backend = SqliteBackend::new(db_path).await.unwrap();
        
        // Test save and load
        let state = create_test_state();
        backend.save_state(&state).await.unwrap();
        
        let loaded = backend.load_state().await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().node_id, state.node_id);
    }

    #[tokio::test]
    async fn test_peer_persistence() {
        let backend = MemoryBackend::default();
        
        let peers = vec![
            PersistedPeer {
                id: vec![1, 2, 3],
                address: "127.0.0.1:8000".to_string(),
                reputation: 75,
                last_seen: 12345,
                stats: PeerStats::default(),
                blacklisted: false,
                whitelisted: true,
                metadata: HashMap::new(),
            },
        ];
        
        backend.save_peers(&peers).await.unwrap();
        let loaded = backend.load_peers().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_state_validation() {
        let backend = MemoryBackend::default();
        
        // Empty state should be valid
        assert!(backend.validate_state().await.unwrap());
        
        // Save valid state
        let state = create_test_state();
        backend.save_state(&state).await.unwrap();
        assert!(backend.validate_state().await.unwrap());
        
        // Save invalid state (empty node_id)
        let mut invalid_state = state.clone();
        invalid_state.node_id = vec![];
        backend.save_state(&invalid_state).await.unwrap();
        assert!(!backend.validate_state().await.unwrap());
    }

    fn create_test_state() -> PersistedState {
        PersistedState {
            version: CURRENT_STATE_VERSION,
            node_id: vec![1, 2, 3, 4],
            protocol_state: ProtocolState::Initial,
            sessions: HashMap::new(),
            peers: vec![],
            dag_state: PersistedDagState {
                vertices: HashMap::new(),
                tips: HashSet::new(),
                voting_records: HashMap::new(),
                last_checkpoint: None,
            },
            metrics: StateMachineMetrics {
                current_state: ProtocolState::Initial,
                uptime: std::time::Duration::from_secs(0),
                active_sessions: 0,
                total_state_transitions: 0,
                total_messages_sent: 0,
                total_messages_received: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                total_errors: 0,
            },
            last_saved: 0,
        }
    }
}