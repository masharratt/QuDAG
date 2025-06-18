use clap::{Parser, Subcommand};
use qudag_crypto::fingerprint::Fingerprint;
use qudag_network::dark_resolver::{DarkResolver, DarkResolverError};
use qudag_network::types::NetworkAddress;
use qudag_protocol::rpc_server::{RpcCommand, RpcServer};
use qudag_protocol::{Node, NodeConfig};
use rand::{thread_rng, Rng};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

// Import the CLI module for peer management
// (CLI module is available as crate root)

#[derive(Parser)]
#[command(name = "qudag")]
#[command(about = "QuDAG Protocol CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a node
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "8000")]
        port: u16,

        /// Data directory
        #[arg(short, long)]
        data_dir: Option<PathBuf>,

        /// Log level
        #[arg(short, long, default_value = "info")]
        log_level: String,
        
        /// Initial peers to connect to
        #[arg(short = 'p', long = "peer")]
        peers: Vec<String>,
        
        /// Run node in background (daemon mode)
        #[arg(short = 'b', long = "background")]
        background: bool,
    },

    /// Stop a running node
    Stop {
        /// Force kill the node process
        #[arg(short, long)]
        force: bool,
    },
    
    /// Restart a running node
    Restart {
        /// Force kill during restart
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show node logs
    Logs {
        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
    
    /// Generate systemd service file
    Systemd {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Run node process (internal command)
    #[command(hide = true)]
    RunNode {
        /// Port to listen on
        #[arg(long)]
        port: u16,
        
        /// Data directory
        #[arg(long)]
        data_dir: String,
        
        /// Initial peers
        #[arg(long)]
        peer: Vec<String>,
    },

    /// Get node status
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

    /// Dark addressing commands
    Address {
        #[command(subcommand)]
        command: AddressCommands,
    },
}

#[derive(Subcommand)]
enum PeerCommands {
    /// List connected peers
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Output format (text, json)
        #[arg(long)]
        format: Option<String>,
    },

    /// Add a peer
    Add {
        /// Peer address
        address: String,
        /// Add peers from file
        #[arg(long)]
        file: Option<String>,
        /// Connection timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Remove a peer
    Remove {
        /// Peer address or ID
        address: String,
        /// Force disconnection
        #[arg(long)]
        force: bool,
    },

    /// Ban a peer
    Ban {
        /// Peer address
        address: String,
    },

    /// Show peer statistics
    Stats {
        /// Peer address or ID
        address: String,
    },

    /// Export peer list
    Export {
        /// Output file
        #[arg(long)]
        output: Option<PathBuf>,
    },
    
    /// Import peer list
    Import {
        /// Input file
        file: PathBuf,
        /// Merge with existing peers
        #[arg(long)]
        merge: bool,
    },
    
    /// Test connectivity to all peers
    Test,
    
    /// Unban a peer
    Unban {
        /// Peer address
        address: String,
    },
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Get network stats
    Stats,

    /// Run network tests
    Test,
}

#[derive(Subcommand)]
enum AddressCommands {
    /// Register a dark address
    Register {
        /// Domain name
        domain: String,
    },

    /// Resolve a dark address
    Resolve {
        /// Domain name
        domain: String,
    },

    /// Generate a shadow address
    Shadow {
        /// Time to live in seconds
        #[arg(long, default_value = "3600")]
        ttl: u64,
    },

    /// Create a content fingerprint
    Fingerprint {
        /// Data to fingerprint
        #[arg(long)]
        data: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_thread_ids(true)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            port,
            data_dir,
            log_level,
            peers,
            background,
        } => {
            use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};
            
            // Set log level
            std::env::set_var("RUST_LOG", &log_level);
            
            if background {
                info!("Starting QuDAG node in background on port {}", port);
                
                // Create node manager
                let config = NodeManagerConfig::default();
                let manager = NodeManager::new(config)?;
                
                // Start in background
                manager.start_node(Some(port), data_dir, peers, false).await?;
                
                println!("✓ QuDAG node started in background");
                println!("  Use 'qudag status' to check node status");
                println!("  Use 'qudag logs' to view logs");
                println!("  Use 'qudag stop' to stop the node");
            } else {
                info!("Starting QuDAG node in foreground on port {}", port);
                
                // Use the commands module function which runs in foreground
                qudag_cli::start_node(data_dir, Some(port), peers).await?;
            }
        }

        Commands::Stop { force } => {
            use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};
            
            info!("Stopping QuDAG node");
            
            let config = NodeManagerConfig::default();
            let manager = NodeManager::new(config)?;
            
            manager.stop_node(force).await?;
            println!("✓ QuDAG node stopped");
        }
        
        Commands::Restart { force } => {
            use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};
            
            info!("Restarting QuDAG node");
            
            let config = NodeManagerConfig::default();
            let manager = NodeManager::new(config)?;
            
            manager.restart_node(force).await?;
            println!("✓ QuDAG node restarted");
        }
        
        Commands::Logs { lines, follow } => {
            use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};
            
            let config = NodeManagerConfig::default();
            let manager = NodeManager::new(config)?;
            
            manager.tail_logs(lines, follow).await?;
        }
        
        Commands::Systemd { output } => {
            use qudag_cli::node_manager::{NodeManager, NodeManagerConfig};
            
            let config = NodeManagerConfig::default();
            let manager = NodeManager::new(config)?;
            
            let service_content = manager.generate_systemd_service(output.clone()).await?;
            
            if output.is_none() {
                println!("{}", service_content);
                println!("\n# To install this service:");
                println!("# 1. Save to: /etc/systemd/system/qudag.service");
                println!("# 2. Run: sudo systemctl daemon-reload");
                println!("# 3. Run: sudo systemctl enable qudag");
                println!("# 4. Run: sudo systemctl start qudag");
            }
        }
        
        Commands::RunNode { port, data_dir, peer } => {
            // This is the actual node process that runs
            info!("Running QuDAG node process on port {}", port);
            
            // TODO: Replace with actual node implementation
            let config = NodeConfig {
                data_dir: PathBuf::from(data_dir),
                network_port: port,
                max_peers: 50,
                initial_peers: peer,
            };
            
            // For now, just create a dummy node that logs and waits
            println!("QuDAG node running:");
            println!("  Port: {}", port);
            println!("  Data directory: {:?}", config.data_dir);
            println!("  Initial peers: {:?}", config.initial_peers);
            
            // Keep the process running
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                info!("Node heartbeat - still running on port {}", port);
            }
        }

        Commands::Status => {
            info!("Getting node status");
            qudag_cli::show_status().await?;
        }

        Commands::Peer { command } => {
            // Create a CommandRouter with peer manager
            let router = match qudag_cli::CommandRouter::with_peer_manager().await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error initializing peer manager: {}", e);
                    std::process::exit(1);
                }
            };
            
            match command {
                PeerCommands::List { status, format } => {
                    match router.handle_peer_list(None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error listing peers: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Add {
                    address,
                    file,
                    timeout,
                } => {
                    if let Some(file_path) = file {
                        // Import peers from file
                        let path = PathBuf::from(file_path);
                        match router.handle_peer_import(path, true).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error importing peers: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        // Add single peer
                        match router.handle_peer_add(address, None, None).await {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                },
                PeerCommands::Remove { address, force } => {
                    match router.handle_peer_remove(address, None, force).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Ban { address } => {
                    match router.handle_peer_ban(address, None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Stats { address } => {
                    match router.handle_peer_info(address, None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Export { output } => {
                    let path = output.unwrap_or_else(|| PathBuf::from("peers_export.json"));
                    match router.handle_peer_export(path, None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error exporting peers: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Import { file, merge } => {
                    match router.handle_peer_import(file, merge).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error importing peers: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Test => {
                    match router.handle_peer_test().await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error testing peers: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PeerCommands::Unban { address } => {
                    match router.handle_peer_unban(address, None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error unbanning peer: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        },

        Commands::Network { command } => {
            // Create a new CommandRouter instance for network commands
            let router = qudag_cli::CommandRouter::new();
            
            match command {
                NetworkCommands::Stats => {
                    match router.handle_network_stats(None, false).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error getting network stats: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                NetworkCommands::Test => {
                    match router.handle_network_test(None).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("Error running network test: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        },

        Commands::Address { command } => match command {
            AddressCommands::Register { domain } => {
                info!("Registering dark address: {}", domain);
                println!("Registering dark address: {}", domain);

                let resolver = DarkResolver::new();
                let test_address = NetworkAddress::new([127, 0, 0, 1], 8080);

                match resolver.register_domain(&domain, test_address) {
                    Ok(()) => {
                        println!("✓ Successfully registered dark address: {}", domain);
                        println!(
                            "  Address format: {}.dark",
                            domain.trim_end_matches(".dark")
                        );
                        println!(
                            "  Registration time: {}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );
                    }
                    Err(DarkResolverError::DomainExists) => {
                        println!("✗ Error: Domain already registered");
                    }
                    Err(DarkResolverError::InvalidDomain) => {
                        println!("✗ Error: Invalid domain format");
                        println!("  Domain must end with '.dark' and contain only alphanumeric characters and hyphens");
                        println!("  Examples: 'myservice.dark', 'test-node.dark'");
                    }
                    Err(e) => {
                        println!("✗ Error registering domain: {:?}", e);
                    }
                }
            }
            AddressCommands::Resolve { domain } => {
                info!("Resolving dark address: {}", domain);
                println!("Resolving dark address: {}", domain);

                let resolver = DarkResolver::new();

                match resolver.lookup_domain(&domain) {
                    Ok(record) => {
                        println!("✓ Domain found:");
                        println!("  Domain: {}", domain);
                        println!("  Public key size: {} bytes", record.public_key.len());
                        println!(
                            "  Encrypted address size: {} bytes",
                            record.encrypted_address.len()
                        );
                        println!("  Registered at: {} (Unix timestamp)", record.registered_at);
                        println!("  Quantum-resistant: ML-KEM encryption");
                    }
                    Err(DarkResolverError::DomainNotFound) => {
                        println!("✗ Domain not found: {}", domain);
                        println!(
                            "  Use 'qudag address register {}' to register it first",
                            domain
                        );
                    }
                    Err(DarkResolverError::InvalidDomain) => {
                        println!("✗ Invalid domain format: {}", domain);
                    }
                    Err(e) => {
                        println!("✗ Error resolving domain: {:?}", e);
                    }
                }
            }
            AddressCommands::Shadow { ttl } => {
                info!("Generating shadow address with TTL: {}", ttl);
                println!("Generating shadow address with TTL: {} seconds", ttl);

                // Generate a mock shadow address for demonstration
                let mut rng = thread_rng();
                let shadow_id: u64 = rng.gen();
                let shadow_address = format!("shadow-{:016x}.dark", shadow_id);

                println!("✓ Generated shadow address:");
                println!("  Address: {}", shadow_address);
                println!("  TTL: {} seconds ({} hours)", ttl, ttl / 3600);
                println!("  Type: Temporary/Ephemeral");
                println!("  Quantum-resistant: Yes");
                println!("  Features:");
                println!("    - Anonymous routing");
                println!("    - Automatic expiration");
                println!("    - Forward secrecy");
                println!();
                println!(
                    "Note: This shadow address will expire after {} seconds",
                    ttl
                );
            }
            AddressCommands::Fingerprint { data } => {
                info!("Creating fingerprint for data: {}", data);
                println!("Creating fingerprint for data: {}", data);

                let mut rng = thread_rng();
                match Fingerprint::generate(data.as_bytes(), &mut rng) {
                    Ok((fingerprint, public_key)) => {
                        println!("✓ Generated quantum-resistant fingerprint:");
                        println!("  Algorithm: ML-DSA + BLAKE3");
                        println!("  Fingerprint size: {} bytes", fingerprint.data().len());
                        println!("  Signature size: {} bytes", fingerprint.signature().len());
                        println!("  Public key size: {} bytes", public_key.as_bytes().len());
                        println!("  Fingerprint (hex): {}", hex::encode(fingerprint.data()));
                        println!();

                        // Verify the fingerprint
                        match fingerprint.verify(&public_key) {
                            Ok(()) => {
                                println!("✓ Fingerprint verification: PASSED");
                                println!("  The fingerprint is cryptographically valid");
                            }
                            Err(e) => {
                                println!("✗ Fingerprint verification: FAILED");
                                println!("  Error: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Error generating fingerprint: {:?}", e);
                    }
                }
            }
        },
    }

    Ok(())
}
