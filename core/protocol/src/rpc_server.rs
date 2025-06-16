use crate::{Node, ProtocolError, ProtocolState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info};
use uuid::Uuid;

/// RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: Uuid,
    pub method: String,
    pub params: serde_json::Value,
}

/// RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: Uuid,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
}

/// RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// RPC command types
#[derive(Debug, Clone)]
pub enum RpcCommand {
    Stop,
    GetStatus,
}

/// RPC server for handling remote commands
pub struct RpcServer {
    port: u16,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    command_tx: mpsc::Sender<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
}

impl RpcServer {
    /// Create new RPC server
    pub fn new(port: u16) -> (Self, mpsc::Receiver<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>) {
        let (command_tx, command_rx) = mpsc::channel(100);
        
        let server = Self {
            port,
            shutdown_tx: None,
            command_tx,
        };
        
        (server, command_rx)
    }

    /// Start RPC server
    pub async fn start(&mut self) -> Result<(), ProtocolError> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .await
            .map_err(|e| ProtocolError::NetworkError(e.to_string()))?;
        
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);
        
        let command_tx = self.command_tx.clone();
        
        tokio::spawn(async move {
            info!("RPC server listening on port {}", listener.local_addr().unwrap());
            
            loop {
                tokio::select! {
                    Ok((stream, addr)) = listener.accept() => {
                        debug!("New RPC connection from {}", addr);
                        let command_tx = command_tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, command_tx).await {
                                error!("Error handling RPC connection: {}", e);
                            }
                        });
                    }
                    _ = &mut shutdown_rx => {
                        info!("RPC server shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Stop RPC server
    pub async fn stop(&mut self) -> Result<(), ProtocolError> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }
}

/// Handle RPC connection
async fn handle_connection(
    mut stream: TcpStream,
    command_tx: mpsc::Sender<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read request length
    let request_len = stream.read_u32().await?;
    
    // Read request data
    let mut request_data = vec![0u8; request_len as usize];
    stream.read_exact(&mut request_data).await?;
    
    // Parse request
    let request: RpcRequest = serde_json::from_slice(&request_data)?;
    
    // Handle request
    let response = handle_request(request, command_tx).await;
    
    // Send response
    let response_data = serde_json::to_vec(&response)?;
    stream.write_u32(response_data.len() as u32).await?;
    stream.write_all(&response_data).await?;
    
    Ok(())
}

/// Handle RPC request
async fn handle_request(
    request: RpcRequest,
    command_tx: mpsc::Sender<(RpcCommand, tokio::sync::oneshot::Sender<serde_json::Value>)>,
) -> RpcResponse {
    match request.method.as_str() {
        "stop" => {
            info!("Received stop request via RPC");
            let (tx, rx) = tokio::sync::oneshot::channel();
            
            if let Err(_) = command_tx.send((RpcCommand::Stop, tx)).await {
                return RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -1,
                        message: "Failed to send stop command".to_string(),
                        data: None,
                    }),
                };
            }
            
            match rx.await {
                Ok(result) => RpcResponse {
                    id: request.id,
                    result: Some(result),
                    error: None,
                },
                Err(_) => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -1,
                        message: "Command execution failed".to_string(),
                        data: None,
                    }),
                },
            }
        }
        "get_status" => {
            let (tx, rx) = tokio::sync::oneshot::channel();
            
            if let Err(_) = command_tx.send((RpcCommand::GetStatus, tx)).await {
                return RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -1,
                        message: "Failed to send status command".to_string(),
                        data: None,
                    }),
                };
            }
            
            match rx.await {
                Ok(result) => RpcResponse {
                    id: request.id,
                    result: Some(result),
                    error: None,
                },
                Err(_) => RpcResponse {
                    id: request.id,
                    result: None,
                    error: Some(RpcError {
                        code: -1,
                        message: "Command execution failed".to_string(),
                        data: None,
                    }),
                },
            }
        }
        _ => RpcResponse {
            id: request.id,
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: format!("Method '{}' not found", request.method),
                data: None,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_serialization() {
        let request = RpcRequest {
            id: Uuid::new_v4(),
            method: "stop".to_string(),
            params: serde_json::Value::Null,
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: RpcRequest = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(request.method, deserialized.method);
    }
}