# Anonymous Routing Implementation

The QuDAG network layer implements a sophisticated anonymous routing system built on onion routing principles to ensure communication privacy.

## Core Components

### Router Implementation

The anonymous routing system is implemented in the `Router` struct, which provides:

- Minimum circuit length of 3 hops for base anonymity
- Maximum circuit length of 7 hops for performance balance
- Dynamic route creation with forward secrecy
- Circuit-based message routing

### Key Features

1. **Circuit Building**
   - Random peer selection for route diversity
   - Layered encryption using ML-KEM
   - Per-hop forward secrecy
   - TTL-based circuit expiration

2. **Route Validation**
   - Minimum hop count enforcement
   - Peer diversity checks
   - Circuit integrity verification
   - Route expiration management

### Routing Strategies

The system supports multiple routing strategies:

1. **Direct**
   - Point-to-point communication
   - Uses routing table lookups
   - Fallback path handling

2. **Flood**
   - Broadcast to all peers
   - Used for network-wide messages
   - Automatic peer discovery

3. **Random Subset**
   - Select random subset of peers
   - Load balancing
   - Network probing

4. **Anonymous**
   - Onion routing with multiple hops
   - Layered encryption
   - Forward secrecy
   - Circuit reuse

## Security Considerations

- All routing layers use quantum-resistant encryption
- No single peer knows the complete route
- Circuit building uses forward secrecy
- Route randomization prevents correlation
- Minimum hop count ensures base anonymity level