# KG3-12 - Session Documentation

## Date
Mon Feb 16 2026

## Summary
KG3-12 (KaspaGraffiti CLI) - Successfully updated with gRPC support and web UI enhancements.

---

## Key Changes Made

### 1. gRPC Support (Auto-detect)
- Added `kaspa-grpc-client` dependency
- Created `src/rpc/grpc_client.rs` with gRPC client implementation
- Auto-detects local vs public URLs:
  - `localhost` / `127.0.0.1` → gRPC
  - `https://api-tn*.kaspa.org` → HTTP/REST
- Fixed URL handling: strips `http://` before adding `grpc://`

### 2. Web UI Enhancements
- **Network Switcher**: TN10 / TN12 / Local Node buttons
- **Node Sync Button**: Shows sync progress via rothschild
- **Test Connection Button**: Tests selected network
- **Get Network Info**: Shows detailed network stats per network
- **Auto-update RPC URL**: When switching networks, updates input field
- **Sync Percentage**: Shows progress (X% / blocks)

### 3. Testnet 12 Specs Added
- Created `specs.md` with full TN12 documentation:
  - Network specs (ports, DNS seeders)
  - Genesis block details
  - Consensus parameters
  - Architecture diagram
  - RPC protocol comparison
  - Hardware requirements

---

## File Changes

### Rust Code
| File | Change |
|------|--------|
| `Cargo.toml` | Added `kaspa-grpc-client`, `kaspa-rpc-core`, `kaspa-core` |
| `src/rpc/grpc_client.rs` | NEW - gRPC client implementation |
| `src/rpc/mod.rs` | Added module exports |
| `src/lib.rs` | Added GrpcRpcClient exports |
| `src/commands.rs` | Added gRPC support for balance, utxos, transfer, graffiti |
| `src/wallet/kaspa_signer.rs` | Updated signing (kaspa_consensus_core::sign) |

### Web UI (index.html)
- Added network switcher buttons
- Added "Node Sync" button
- Fixed "Test Connection" to use current network
- Fixed "Get Network Info" to use current network
- Added sync progress display
- Added warning/emoji styles for log messages

### Server (server.cjs)
- Added `/run-raw` endpoint for executing arbitrary commands (rothschild)

---

## Current Status

### Local Node
- **gRPC Port**: 16210
- **Sync Progress**: ~28-30% (syncing)
- **Status**: IBD (Initial Block Download)

### Balances
| Network | Balance | Status |
|---------|---------|--------|
| Public TN12 | 8,790 KAS | ✅ Synced |
| Public TN10 | 636 KAS | ✅ Synced |
| Local TN12 | 0 | ⚠️ Syncing |

### URLs
- **Web UI**: http://localhost:8081
- **CLI**: `./target/release/kaspa-graffiti-cli`

---

## Previous Issues Resolved

### Transaction Failure (Mempool Conflict)
- **Error**: `output already spent by transaction ... in mempool`
- **Cause**: Previous pending transaction locked the UTXO
- **Solution**: Waited for mempool to clear

### gRPC Connection
- **Issue**: Local node uses gRPC, CLI used HTTP
- **Fix**: Added gRPC client with auto-detection

### TN10 vs TN12
- **Issue**: Both pointing to api-tn12 (bug)
- **Fix**: TN10 → api-tn10, TN12 → api-tn12

---

## Commands Reference

### CLI
```bash
# Balance on public TN12
./kaspa-graffiti-cli balance <addr> --rpc https://api-tn12.kaspa.org

# Balance on local node (gRPC)
./kaspa-graffiti-cli balance <addr> --rpc localhost:16210

# Transfer
./kaspa-graffiti-cli transfer <key> <addr> <amount>
```

### Node Sync
```bash
/Users/4dsto/kaspa-node/rusty-kaspa/target/release/rothschild \
  --private-key <key> -t 0
```

---

## Next Steps
1. Wait for local node to finish syncing (100%)
2. Test transfer on local node
3. Test deadman switch in ktn12 silverscript
