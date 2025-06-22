# QuDAG Exchange CLI Command Reference

## Overview

The QuDAG Exchange CLI (`qudag-exchange-cli`) provides command-line access to all exchange functionality. Commands are organized into logical groups for ease of use.

## Global Options

These options can be used with any command:

```bash
--config <path>     # Path to config file (default: ~/.qudag/config.toml)
--verbose, -v       # Enable verbose output
--quiet, -q         # Suppress non-error output
--json              # Output in JSON format
--network <name>    # Network to connect to (mainnet, testnet, local)
--no-color         # Disable colored output
```

## Account Commands

### create-account

Create a new account with quantum-resistant keys.

```bash
qudag-exchange-cli create-account --name <name> [options]

Options:
  --name <name>              # Account name (required)
  --password <password>      # Vault password (prompted if not provided)
  --key-type <type>          # Key algorithm: ml-dsa, ml-kem (default: ml-dsa)
  --derivation-path <path>   # HD derivation path (default: m/44'/0'/0'/0/0)
  --backup <path>            # Backup file path

Examples:
  # Create account with prompt for password
  qudag-exchange-cli create-account --name alice
  
  # Create with specific key type
  qudag-exchange-cli create-account --name bob --key-type ml-kem
  
  # Create with backup
  qudag-exchange-cli create-account --name carol --backup carol-backup.json
```

### list-accounts

List all local accounts.

```bash
qudag-exchange-cli list-accounts [options]

Options:
  --show-keys        # Display public keys
  --show-balances    # Fetch and display balances

Example:
  qudag-exchange-cli list-accounts --show-balances
```

### delete-account

Delete an account (requires confirmation).

```bash
qudag-exchange-cli delete-account --name <name> [options]

Options:
  --name <name>      # Account name (required)
  --force            # Skip confirmation prompt
  --backup <path>    # Create backup before deletion

Example:
  qudag-exchange-cli delete-account --name alice --backup alice-final.json
```

## Balance Commands

### balance

Check rUv token balance for an account.

```bash
qudag-exchange-cli balance [options]

Options:
  --account <name>   # Account name (default: active account)
  --address <addr>   # Check balance for address
  --all              # Show all account balances

Examples:
  # Check current account balance
  qudag-exchange-cli balance
  
  # Check specific account
  qudag-exchange-cli balance --account alice
  
  # Check address balance
  qudag-exchange-cli balance --address qd1a2b3c4d5e6f...
```

### history

View transaction history.

```bash
qudag-exchange-cli history [options]

Options:
  --account <name>   # Account name
  --limit <n>        # Number of transactions (default: 20)
  --offset <n>       # Skip first n transactions
  --type <type>      # Filter by type: sent, received, all
  --after <date>     # Transactions after date
  --before <date>    # Transactions before date

Example:
  qudag-exchange-cli history --account alice --limit 50 --type sent
```

## Transaction Commands

### transfer

Transfer rUv tokens between accounts.

```bash
qudag-exchange-cli transfer [options]

Options:
  --from <account>   # Sender account (default: active)
  --to <address>     # Recipient address or name
  --amount <n>       # Amount to transfer
  --memo <text>      # Transaction memo
  --fee <n>          # Transaction fee (default: auto)
  --password <pass>  # Vault password

Examples:
  # Simple transfer
  qudag-exchange-cli transfer --to bob --amount 100
  
  # Transfer with memo
  qudag-exchange-cli transfer --to alice --amount 50 --memo "Payment for services"
  
  # Transfer with custom fee
  qudag-exchange-cli transfer --to qd1abc... --amount 1000 --fee 0.1
```

### sign

Sign a message or transaction.

```bash
qudag-exchange-cli sign [options]

Options:
  --account <name>   # Signing account
  --message <text>   # Message to sign
  --file <path>      # File to sign
  --output <path>    # Output signature file

Example:
  qudag-exchange-cli sign --account alice --message "Hello, QuDAG!" --output sig.json
```

### verify

Verify a signature.

```bash
qudag-exchange-cli verify [options]

Options:
  --signature <sig>  # Signature to verify
  --message <text>   # Original message
  --file <path>      # File containing message
  --signer <addr>    # Expected signer address

Example:
  qudag-exchange-cli verify --signature sig.json --message "Hello, QuDAG!"
```

## Key Management Commands

### key generate

Generate new quantum-resistant keys.

```bash
qudag-exchange-cli key generate [options]

Options:
  --account <name>   # Account name
  --type <type>      # Key type: signing, encryption, both
  --algorithm <alg>  # Algorithm: ml-dsa, ml-kem, hqc
  --password <pass>  # Vault password

Examples:
  # Generate signing keys
  qudag-exchange-cli key generate --account alice --type signing
  
  # Generate ML-KEM encryption keys
  qudag-exchange-cli key generate --account bob --type encryption --algorithm ml-kem
```

### key list

List keys for an account.

```bash
qudag-exchange-cli key list [options]

Options:
  --account <name>   # Account name
  --show-private     # Show private keys (dangerous!)
  --type <type>      # Filter by type

Example:
  qudag-exchange-cli key list --account alice
```

### key export

Export public keys.

```bash
qudag-exchange-cli key export [options]

Options:
  --account <name>   # Account name
  --key-id <id>      # Specific key ID
  --format <fmt>     # Format: pem, jwk, raw
  --output <path>    # Output file

Example:
  qudag-exchange-cli key export --account alice --format pem --output alice-pub.pem
```

### key import

Import keys from file.

```bash
qudag-exchange-cli key import [options]

Options:
  --account <name>   # Account name
  --file <path>      # Key file to import
  --format <fmt>     # Format: pem, jwk, raw
  --type <type>      # Key type if not specified

Example:
  qudag-exchange-cli key import --account alice --file key.pem
```

## Node Commands

### node start

Start a QuDAG Exchange node.

```bash
qudag-exchange-cli node start [options]

Options:
  --account <name>   # Node operator account
  --port <port>      # P2P port (default: 8080)
  --rpc-port <port>  # RPC port (default: 9090)
  --bootstrap <addr> # Bootstrap peer addresses
  --data-dir <path>  # Data directory
  --resources <spec> # Resources to offer

Examples:
  # Start basic node
  qudag-exchange-cli node start --account alice
  
  # Start with resource offering
  qudag-exchange-cli node start \
    --account alice \
    --port 8080 \
    --resources "cpu=4,memory=16GB,storage=1TB"
  
  # Start with bootstrap peers
  qudag-exchange-cli node start \
    --bootstrap "/ip4/1.2.3.4/tcp/8080/p2p/Qm..."
```

### node stop

Stop a running node.

```bash
qudag-exchange-cli node stop [options]

Options:
  --graceful         # Wait for pending operations
  --timeout <secs>   # Shutdown timeout

Example:
  qudag-exchange-cli node stop --graceful --timeout 30
```

### node status

Check node status.

```bash
qudag-exchange-cli node status [options]

Options:
  --detailed         # Show detailed metrics
  --resources        # Show resource usage

Example:
  qudag-exchange-cli node status --detailed
```

## Network Commands

### peer list

List connected peers.

```bash
qudag-exchange-cli peer list [options]

Options:
  --verbose          # Show peer details
  --sort <by>        # Sort by: latency, uptime, reputation

Example:
  qudag-exchange-cli peer list --verbose --sort latency
```

### peer connect

Connect to a specific peer.

```bash
qudag-exchange-cli peer connect <address> [options]

Options:
  --persist          # Add to persistent peer list

Example:
  qudag-exchange-cli peer connect /ip4/1.2.3.4/tcp/8080/p2p/QmPeer...
```

### peer disconnect

Disconnect from a peer.

```bash
qudag-exchange-cli peer disconnect <peer-id> [options]

Options:
  --ban              # Ban peer from reconnecting
  --reason <text>    # Disconnection reason

Example:
  qudag-exchange-cli peer disconnect QmPeer... --reason "Misbehavior"
```

### network stats

Display network statistics.

```bash
qudag-exchange-cli network stats [options]

Options:
  --interval <secs>  # Update interval for live stats
  --export <path>    # Export stats to file

Example:
  qudag-exchange-cli network stats --interval 5
```

## Resource Commands

### offer create

Create a resource offer.

```bash
qudag-exchange-cli offer create [options]

Options:
  --type <type>      # Resource type: compute, storage, bandwidth
  --specs <spec>     # Resource specifications
  --price <price>    # Price in rUv per unit
  --duration <time>  # Offer duration
  --auto-renew       # Auto-renew offer

Examples:
  # Offer compute resources
  qudag-exchange-cli offer create \
    --type compute \
    --specs "cpu=8,memory=32GB,gpu=1xRTX4090" \
    --price "50 rUv/hour" \
    --duration 24h
  
  # Offer storage
  qudag-exchange-cli offer create \
    --type storage \
    --specs "capacity=10TB,redundancy=3,bandwidth=1Gbps" \
    --price "10 rUv/TB/day"
```

### offer list

List available resource offers.

```bash
qudag-exchange-cli offer list [options]

Options:
  --type <type>      # Filter by resource type
  --max-price <n>    # Maximum price filter
  --min-specs <spec> # Minimum specifications
  --provider <addr>  # Filter by provider

Example:
  qudag-exchange-cli offer list --type compute --max-price 100
```

### offer accept

Accept a resource offer.

```bash
qudag-exchange-cli offer accept <offer-id> [options]

Options:
  --duration <time>  # Reservation duration
  --auto-renew       # Enable auto-renewal
  --start <time>     # Start time (default: now)

Example:
  qudag-exchange-cli offer accept offer_123abc --duration 4h
```

### resources monitor

Monitor resource usage.

```bash
qudag-exchange-cli resources monitor [options]

Options:
  --account <name>   # Monitor specific account
  --interval <secs>  # Update interval
  --alerts           # Enable usage alerts

Example:
  qudag-exchange-cli resources monitor --interval 10 --alerts
```

## Provider Commands

### provider register

Register as a resource provider.

```bash
qudag-exchange-cli provider register [options]

Options:
  --type <types>     # Resource types to provide
  --specs <spec>     # Hardware specifications
  --location <loc>   # Geographic location
  --sla <level>      # SLA commitment level

Example:
  qudag-exchange-cli provider register \
    --type "compute,storage" \
    --specs "cpu=64,memory=256GB,storage=100TB" \
    --location "US-EAST" \
    --sla "99.9"
```

### provider set-price

Set resource pricing.

```bash
qudag-exchange-cli provider set-price [options]

Options:
  --resource <type>  # Resource type
  --price <price>    # Price per unit
  --discount <spec>  # Volume discounts

Example:
  qudag-exchange-cli provider set-price \
    --resource compute \
    --price "100 rUv/hour" \
    --discount "10%@100hours,20%@1000hours"
```

### provider stats

View provider statistics.

```bash
qudag-exchange-cli provider stats [options]

Options:
  --period <time>    # Time period
  --export <path>    # Export stats

Example:
  qudag-exchange-cli provider stats --period 30d
```

## Market Commands

### market search

Search for resources.

```bash
qudag-exchange-cli market search [options]

Options:
  --resource <type>  # Resource type
  --specs <spec>     # Required specifications
  --max-price <n>    # Maximum price
  --location <loc>   # Preferred location

Example:
  qudag-exchange-cli market search \
    --resource compute \
    --specs "gpu>=2,memory>=40GB" \
    --max-price 200
```

### market reserve

Reserve resources.

```bash
qudag-exchange-cli market reserve [options]

Options:
  --provider <addr>  # Provider address
  --resource <type>  # Resource type
  --amount <n>       # Amount to reserve
  --duration <time>  # Reservation period

Example:
  qudag-exchange-cli market reserve \
    --provider qd1provider... \
    --resource compute \
    --amount 4 \
    --duration 8h
```

## Vault Commands

### vault create

Create a new vault.

```bash
qudag-exchange-cli vault create [options]

Options:
  --name <name>      # Vault name
  --password <pass>  # Master password
  --kdf-rounds <n>   # Key derivation rounds

Example:
  qudag-exchange-cli vault create --name main-vault
```

### vault unlock

Unlock a vault.

```bash
qudag-exchange-cli vault unlock [options]

Options:
  --name <name>      # Vault name
  --password <pass>  # Master password
  --duration <time>  # Auto-lock duration

Example:
  qudag-exchange-cli vault unlock --name main-vault --duration 30m
```

### vault backup

Backup vault data.

```bash
qudag-exchange-cli vault backup [options]

Options:
  --vault <name>     # Vault name
  --output <path>    # Backup file path
  --encrypt          # Encrypt backup

Example:
  qudag-exchange-cli vault backup --vault main-vault --output backup.enc --encrypt
```

## Configuration Commands

### config get

Get configuration values.

```bash
qudag-exchange-cli config get <key> [options]

Example:
  qudag-exchange-cli config get network.bootstrap_peers
```

### config set

Set configuration values.

```bash
qudag-exchange-cli config set <key> <value> [options]

Example:
  qudag-exchange-cli config set api.port 3000
```

### config show

Display full configuration.

```bash
qudag-exchange-cli config show [options]

Options:
  --defaults         # Show default values
  --format <fmt>     # Output format: toml, json, yaml

Example:
  qudag-exchange-cli config show --format json
```

## Maintenance Commands

### maintenance compact

Compact database storage.

```bash
qudag-exchange-cli maintenance compact [options]

Options:
  --target <size>    # Target size reduction
  --analyze          # Analyze before compacting

Example:
  qudag-exchange-cli maintenance compact --analyze
```

### maintenance prune

Prune old data.

```bash
qudag-exchange-cli maintenance prune [options]

Options:
  --before <date>    # Prune data before date
  --keep-recent <n>  # Keep recent n blocks
  --dry-run          # Show what would be pruned

Example:
  qudag-exchange-cli maintenance prune --before 2024-01-01 --dry-run
```

## Debug Commands

### debug info

Display debug information.

```bash
qudag-exchange-cli debug info [options]

Options:
  --system           # Include system info
  --network          # Include network info
  --performance      # Include performance metrics

Example:
  qudag-exchange-cli debug info --system --network
```

### debug export

Export debug data.

```bash
qudag-exchange-cli debug export [options]

Options:
  --output <path>    # Export file path
  --include <types>  # Data types to include
  --compress         # Compress output

Example:
  qudag-exchange-cli debug export --output debug.tar.gz --compress
```

## Environment Variables

The CLI respects these environment variables:

```bash
QUDAG_HOME          # QuDAG home directory (default: ~/.qudag)
QUDAG_CONFIG        # Config file path
QUDAG_NETWORK       # Default network
QUDAG_LOG_LEVEL     # Log level: trace, debug, info, warn, error
QUDAG_NO_COLOR      # Disable colored output
QUDAG_VAULT_PASS    # Vault password (use with caution!)
```

## Configuration File

Default configuration file location: `~/.qudag/config.toml`

```toml
[account]
default = "alice"

[network]
name = "mainnet"
bootstrap_peers = [
  "/ip4/1.2.3.4/tcp/8080/p2p/Qm...",
  "/ip4/5.6.7.8/tcp/8080/p2p/Qm..."
]

[api]
host = "127.0.0.1"
port = 3000

[node]
data_dir = "~/.qudag/data"
max_peers = 50
listen_addresses = ["/ip4/0.0.0.0/tcp/8080"]

[resources]
cpu_limit = 4
memory_limit = "16GB"
storage_limit = "1TB"

[logging]
level = "info"
file = "~/.qudag/logs/node.log"
```

## Exit Codes

- `0`: Success
- `1`: General error
- `2`: Configuration error
- `3`: Network error
- `4`: Insufficient balance
- `5`: Authentication error
- `6`: Validation error
- `7`: Resource unavailable

## Examples

### Complete Workflow Example

```bash
# 1. Create account
qudag-exchange-cli create-account --name alice

# 2. Start node
qudag-exchange-cli node start --account alice

# 3. Check balance
qudag-exchange-cli balance --account alice

# 4. Offer resources
qudag-exchange-cli offer create \
  --type compute \
  --specs "cpu=8,memory=32GB" \
  --price "50 rUv/hour"

# 5. Earn rUv tokens
# (Automatically credited as resources are used)

# 6. Transfer tokens
qudag-exchange-cli transfer \
  --to bob \
  --amount 100 \
  --memo "Test transfer"

# 7. Monitor node
qudag-exchange-cli node status --detailed
```

## Getting Help

```bash
# General help
qudag-exchange-cli --help

# Command-specific help
qudag-exchange-cli transfer --help

# View manual page
man qudag-exchange-cli

# Online documentation
https://docs.qudag.io/cli
```