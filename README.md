# KG3-12 - KaspaGraffiti CLI

A Rust-based Kaspa wallet for testnet-10 and testnet-12 with CLI and web UI interfaces.

## Project Summary

KG3-12 is a command-line wallet and web interface for Kaspa testnet networks. It supports:
- **Wallet generation** - Create new private keys and addresses
- **HD Wallets** - BIP32/BIP44 hierarchical deterministic wallets  
- **Balance checking** - Query UTXOs from local or public nodes
- **Transfers** - Send KAS with Schnorr signatures
- **Multi-network** - Testnet-10, Testnet-12, and local nodes

### Key Features

| Feature          | Description                      |
|------------------|----------------------------------|
| gRPC Support     | Auto-detect local nodes via gRPC |
| HTTP Support     | Public API endpoints             |
| Network Switcher | TN10 / TN12 / Local Node         |
| Sync Status      | Real-time sync progress          |
| Web UI           | Browser-based wallet interface   |

## Technical Specifications

### Network Configuration

| Network | RPC URL | Protocol | Address Prefix |
|---------|---------|----------|---------------|
| Testnet-10 | https://api-tn10.kaspa.org | HTTP | kaspatest: |
| Testnet-12 | https://api-tn12.kaspa.org | HTTP | kaspatest: |
| Local Node | localhost:16210 | gRPC | kaspatest: |

### Local Node Setup

```bash
# Start local testnet-12 node
/Users/4dsto/kaspa-node/rusty-kaspa/target/release/kaspad \
  --testnet --netsuffix=12 --utxoindex

# Check sync status
/Users/4dsto/kaspa-node/rusty-kaspa/target/release/rothschild \
  --private-key <key> -t 0
```

### Ports

| Service     | Port  |
|-------------|-------|
| gRPC (TN12) | 16210 |
| wRPC JSON   | 18210 |
| P2P         | 16311 |

### Consensus Parameters (TN12)

- **Covenants**: Enabled
- **Crescendo**: Enabled
- **Max Signature Script**: 300,000 bytes
- **Block Mass**: 500K compute, 500K storage, 1M transient

## Quick Start

### CLI

```bash
# Build
cargo build --release

# Generate wallet
./target/release/kaspa-graffiti-cli generate

# Check balance
./target/release/kaspa-graffiti-cli balance <address>

# Transfer
./target/release/kaspa-graffiti-cli transfer <key> <recipient> <amount>
```

### Web UI

```bash
cd src-ui
node server.cjs
# Open http://localhost:8081
```

## Commands

| Command | Description |
|---------|-------------|
| `generate` | Generate new wallet |
| `load <key>` | Load wallet from private key |
| `hd-generate` | Generate HD wallet (seed) |
| `hd-load <seed>` | Load HD wallet |
| `balance <address>` | Check balance |
| `utxos <address>` | Get UTXOs |
| `transfer <key> <addr> <amt>` | Send KAS |

## Web UI Features

- **Network Switcher**: TN10 / TN12 / Local Node buttons
- **Node Sync**: Real-time sync progress display
- **Test Connection**: Verify network connectivity
- **Multi-wallet**: Load and switch between wallets

## Architecture

```
┌─────────────────────────────────────────────┐
│              KG3-12 (CLI + Web UI)          │
├─────────────────────────────────────────────┤
│                                             │
│  ┌──────────┐    ┌──────────┐    ┌───────┐  │
│  │HTTP Client│   │gRPC Client│   │Wallet │  │
│  └────┬─────┘    └────┬─────┘    └───┬───┘  │
│       │                │               │    │
│       ▼                ▼               ▼    │
│  ┌─────────────────────────────────────────┐│
│  │          Kaspa Network                  ││
│  │  (Public API / Local Node)              ││
│  └─────────────────────────────────────────┘│
└─────────────────────────────────────────────┘
```

## Dependencies

- **kaspa-addresses** - Address encoding
- **kaspa-consensus-core** - Transaction signing
- **kaspa-txscript** - Script utilities
- **kaspa-grpc-client** - gRPC connectivity
- **secp256k1** - Schnorr signatures

## Build

```bash
# CLI only
cargo build --release

# Binary location
./target/release/kaspa-graffiti-cli
```

## Current Status

| Feature | Status |
|---------|--------|
| Wallet generation | ✅ Working |
| HD wallets | ✅ Working |
| Balance (TN10/TN12) | ✅ Working |
| Balance (Local) | ✅ Working |
| KAS transfers | ✅ Working |
| Web UI | ✅ Working |
| gRPC auto-detect | ✅ Working |

## Documentation

- [specs.md](./specs.md) - Full TN12 technical specs
- [CHANGES.md](./CHANGES.md) - Change history
- [debug_session.md](./debug_session.md) - Session notes

## License

MIT
