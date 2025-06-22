//! QuDAG Exchange CLI - Command line interface for resource exchange

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use qudag_exchange_core::{Ledger, RuvAmount};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

mod commands;
mod config;
mod display;

use commands::{WalletCommand, TransactionCommand, ResourceCommand};
use config::Config;

/// QuDAG Exchange - Decentralized resource utilization voucher system
#[derive(Parser)]
#[command(name = "qudag-exchange")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wallet operations
    Wallet {
        #[command(subcommand)]
        cmd: WalletCommand,
    },
    
    /// Transaction operations
    Transaction {
        #[command(subcommand)]
        cmd: TransactionCommand,
    },
    
    /// Resource contribution operations
    Resource {
        #[command(subcommand)]
        cmd: ResourceCommand,
    },
    
    /// Show network statistics
    Stats,
    
    /// Initialize configuration
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(level)
        .init();

    // Load configuration
    let config = if let Some(path) = cli.config {
        Config::load(&path)?
    } else {
        Config::default()
    };

    // Create ledger instance
    let ledger = Arc::new(RwLock::new(Ledger::new()));

    // Execute command
    match cli.command {
        Commands::Wallet { cmd } => {
            commands::handle_wallet_command(cmd, ledger, config).await?;
        }
        Commands::Transaction { cmd } => {
            commands::handle_transaction_command(cmd, ledger, config).await?;
        }
        Commands::Resource { cmd } => {
            commands::handle_resource_command(cmd, ledger, config).await?;
        }
        Commands::Stats => {
            display::show_stats(ledger).await?;
        }
        Commands::Init => {
            config::initialize_config()?;
            println!("{}", "Configuration initialized successfully!".green());
        }
    }

    Ok(())
}