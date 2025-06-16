#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str;

// Mock CLI command enum to match intended implementation
#[derive(Debug, PartialEq)]
enum CliCommand {
    Start { peer_id: Option<String>, port: Option<u16> },
    Stop,
    Status,
    Connect { address: String },
    SendMessage { target: String, message: String },
    ListPeers,
}

// Mock CLI parser that will be implemented in tools/cli
fn parse_command(input: &str) -> Result<CliCommand, String> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    match parts[0] {
        "start" => {
            let mut peer_id = None;
            let mut port = None;
            let mut i = 1;
            while i < parts.len() {
                match parts[i] {
                    "--peer-id" | "-p" => {
                        i += 1;
                        if i < parts.len() {
                            peer_id = Some(parts[i].to_string());
                        } else {
                            return Err("Missing peer ID value".to_string());
                        }
                    }
                    "--port" => {
                        i += 1;
                        if i < parts.len() {
                            port = match parts[i].parse::<u16>() {
                                Ok(p) => Some(p),
                                Err(_) => return Err("Invalid port number".to_string()),
                            };
                        } else {
                            return Err("Missing port value".to_string());
                        }
                    }
                    _ => return Err(format!("Unknown start argument: {}", parts[i])),
                }
                i += 1;
            }
            Ok(CliCommand::Start { peer_id, port })
        }
        "stop" => {
            if parts.len() > 1 {
                return Err("Stop command takes no arguments".to_string());
            }
            Ok(CliCommand::Stop)
        }
        "status" => {
            if parts.len() > 1 {
                return Err("Status command takes no arguments".to_string());
            }
            Ok(CliCommand::Status)
        }
        "connect" => {
            if parts.len() != 2 {
                return Err("Connect command requires address argument".to_string());
            }
            Ok(CliCommand::Connect {
                address: parts[1].to_string(),
            })
        }
        "send" => {
            if parts.len() < 3 {
                return Err("Send command requires target and message arguments".to_string());
            }
            let target = parts[1].to_string();
            let message = parts[2..].join(" ");
            Ok(CliCommand::SendMessage { target, message })
        }
        "peers" => {
            if parts.len() > 1 {
                return Err("Peers command takes no arguments".to_string());
            }
            Ok(CliCommand::ListPeers)
        }
        _ => Err(format!("Unknown command: {}", parts[0])),
    }
}

// Argument validation
fn validate_peer_id(peer_id: &str) -> bool {
    // Example validation - peer ID should be alphanumeric and 1-64 chars
    !peer_id.is_empty() && peer_id.len() <= 64 && peer_id.chars().all(|c| c.is_alphanumeric())
}

fn validate_port(port: u16) -> bool {
    // Example validation - port should be in valid range
    port > 0 && port < 65535
}

fn validate_address(address: &str) -> bool {
    // Example validation - simple format check for IP:PORT or hostname:PORT
    let parts: Vec<&str> = address.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    if let Ok(port) = parts[1].parse::<u16>() {
        validate_port(port)
    } else {
        false
    }
}

// Fuzz target
fuzz_target!(|data: &[u8]| {
    // Try to convert input bytes to string
    if let Ok(s) = str::from_utf8(data) {
        // Test command parsing
        if let Ok(cmd) = parse_command(s) {
            // Validate parsed command arguments
            match cmd {
                CliCommand::Start { peer_id, port } => {
                    if let Some(id) = peer_id {
                        let _ = validate_peer_id(&id);
                    }
                    if let Some(p) = port {
                        let _ = validate_port(p);
                    }
                }
                CliCommand::Connect { address } => {
                    let _ = validate_address(&address);
                }
                CliCommand::SendMessage { target, message: _ } => {
                    let _ = validate_peer_id(&target);
                }
                _ => {}
            }
        }
    }
});