use clap::{Parser, Subcommand};
use std::path::PathBuf;
use thiserror::Error;

mod commands;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Node error: {0}")]
    Node(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Visualization error: {0}")]
    Visualization(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Parser)]
#[command(name = "qudag")]
#[command(about = "QuDAG node operation and management CLI")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the QuDAG node
    Start {
        #[arg(long)]
        data_dir: Option<PathBuf>,
        
        #[arg(long)]
        port: Option<u16>,
        
        #[arg(long)]
        peers: Vec<String>,
    },

    /// Stop the QuDAG node
    Stop,

    /// Show node status
    Status,

    /// Peer management commands
    Peer {
        #[command(subcommand)]
        command: PeerCommands,
    },

    /// Network management commands
    Network {
        #[command(subcommand)]
        command: NetworkCommands,
    },

    /// DAG visualization
    Dag {
        #[arg(long)]
        output: Option<PathBuf>,
        
        #[arg(long)]
        format: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PeerCommands {
    /// List all peers
    List,
    
    /// Add a new peer
    Add {
        address: String,
    },
    
    /// Remove a peer
    Remove {
        address: String,
    },
}

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// Display network statistics
    Stats,
    
    /// Test network connectivity
    Test,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { data_dir, port, peers } => {
            commands::start_node(data_dir, port, peers).await?;
        }
        Commands::Stop => {
            commands::stop_node().await?;
        }
        Commands::Status => {
            commands::show_status().await?;
        }
        Commands::Peer { command } => {
            match command {
                PeerCommands::List => {
                    commands::list_peers().await?;
                }
                PeerCommands::Add { address } => {
                    commands::add_peer(address).await?;
                }
                PeerCommands::Remove { address } => {
                    commands::remove_peer(address).await?;
                }
            }
        }
        Commands::Network { command } => {
            match command {
                NetworkCommands::Stats => {
                    commands::show_network_stats().await?;
                }
                NetworkCommands::Test => {
                    commands::test_network().await?;
                }
            }
        }
        Commands::Dag { output, format } => {
            commands::visualize_dag(output, format).await?;
        }
    }

    Ok(())
}