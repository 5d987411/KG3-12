# Kaspa Testnet 12 - Technical Specifications

## Overview

Testnet 12 (TN12) is the Kaspa test network running with **covenants** enabled. It was created to test the new covenant functionality that enables smart contracts on Kaspa.

## Network Specifications

| Property | Value |
|----------|-------|
| **Network ID** | testnet-12 |
| **P2P Port** | 16311 |
| **gRPC RPC Port** | 16210 |
| **wRPC JSON Port** | 18210 |
| **HTTP-REST Port** | Requires external proxy |
| **Address Prefix** | `kaspatest:` |
| **DNS Seeders** | tn12-dnsseed.kas.pa, tn12-dnsseed.kasia.fyi |

## Genesis Block

```
Hash:        873c9ca33a848e165fefefe39a805f411af8bab6cdcb77ed927a438480cfebbe
Bits:        504155340
Coinbase:   kaspa-testnet (Launch 2)
```

## Consensus Parameters

| Parameter | Value |
|-----------|-------|
| **Crescendo Activation** | Always (activated) |
| **Covenants Activation** | Always (activated) |
| **Max Signature Script** | 300,000 bytes |
| **Block Mass - Compute** | 500,000 |
| **Block Mass - Storage** | 500,000 |
| **Block Mass - Transient** | 1,000,000 |
| **Max Script Public Key** | 10,000 bytes |

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Testnet 12 Network                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐     ┌──────────────┐     ┌─────────────┐ │
│  │   kaspad     │────▶│  gRPC (16210)│◀───│   KG3-12   │ │
│  │   (node)     │     │  wRPC (18210)│     │   (CLI)    │ │
│  └──────────────┘     └──────────────┘     └─────────────┘ │
│         │                                                         │
│         │ P2P (16311)                                           │
│         ▼                                                         │
│  ┌──────────────┐                                               │
│  │  Other Nodes │                                               │
│  └──────────────┘                                               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### RPC Protocol Comparison

| Protocol | Port | Use Case |
|----------|------|----------|
| gRPC | 16210 | Local nodes, programmatic access |
| wRPC JSON | 18210 | WebSocket JSON-RPC |
| HTTP-REST | N/A | Public APIs (api-tn12.kaspa.org) |

### Starting the Node

```bash
# Basic start
./target/release/kaspad --testnet --netsuffix=12 --utxoindex

# With JSON-RPC
./target/release/kaspad --testnet --netsuffix=12 --utxoindex --rpclisten-json=127.0.0.1:18210
```

## KG3-12 Integration

KG3-12 (KaspaGraffiti CLI) supports both gRPC and HTTP-REST:

### Auto-Detection
- **Local URLs** (localhost, 127.0.0.1) → gRPC
- **Public URLs** (https://api-tn*.kaspa.org) → HTTP-REST

### Usage

```bash
# Using local node (gRPC)
./kaspa-graffiti-cli balance <addr> --rpc localhost:16210

# Using public API (HTTP)
./kaspa-graffiti-cli balance <addr> --rpc https://api-tn12.kaspa.org
```

### Web UI
- URL: http://localhost:8081
- Networks: TN10, TN12, Local Node

## Smart Contracts (Silverscript)

Testnet 12 supports covenant-based smart contracts:

### Available Contracts
| Contract | Size | Description |
|----------|------|-------------|
| p2pkh | 45 bytes | Basic payment to public key hash |
| transfer_with_timeout | 108 bytes | Time-locked transfer |
| escrow | ~100 bytes | Arbiter escrow |
| mecenas | ~200 bytes | Recurring payments |
| hodl_vault | 162 bytes | Oracle vault |
| deadman_switch | Variable | Estate planning / inheritance |

### Deadman Switch Contract

```
pragma silverscript ^0.1.0;

contract DeadmanSwitchSimple(
    pubkey owner,
    pubkey beneficiary,
    int timeout
) {
    entrypoint function cancel(sig s) {
        require(checkSig(s, owner));
        bytes34 lock = new LockingBytecodeP2PK(owner);
        require(tx.outputs[0].lockingBytecode == lock);
    }
    
    entrypoint function claim(sig s) {
        require(checkSig(s, beneficiary));
        require(this.age >= timeout);
        bytes34 lock = new LockingBytecodeP2PK(beneficiary);
        require(tx.outputs[0].lockingBytecode == lock);
    }
}
```

## Protocol Differences: TN10 vs TN12

| Feature | TN10 | TN12 |
|---------|------|------|
| Covenants | ❌ | ✅ |
| Max Signature Script | Default | 300,000 |
| Block Mass (Transient) | Default | 1,000,000 |
| P2P Port | 16310 | 16311 |
| RPC Port | 16110 | 16210 |

## Resources

- **Documentation**: `/kaspa-node/rusty-kaspa/docs/testnet12.md`
- **Genesis Code**: `rusty-kaspa/consensus/core/src/config/genesis.rs`
- **Params Code**: `rusty-kaspa/consensus/core/src/config/params.rs`
- **Faucet**: https://faucet-tn12.kaspanet.io
- **Discord**: #testnet channel

## Hardware Requirements

- **RAM**: 16GB (8GB possible with --ram-scale=0.6)
- **CPU**: 8+ cores
- **Storage**: 250GB+ SSD

## Current Status

- **Local Node**: Running on gRPC port 16210
- **Block Height**: ~1.1M blocks
- **DAA Score**: ~4.5M

---

*Last Updated: Mon Feb 16 2026*
