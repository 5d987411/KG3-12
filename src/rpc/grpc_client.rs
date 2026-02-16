use std::sync::Arc;
use thiserror::Error;

use kaspa_addresses::Address;
use kaspa_consensus_core::tx::Transaction;
use kaspa_grpc_client::{ClientPool, GrpcClient};
use kaspa_rpc_core::{RpcUtxoEntry, api::rpc::RpcApi, notify::mode::NotificationMode};

#[derive(Error, Debug)]
pub enum GrpcError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("RPC error: {0}")]
    Rpc(String),
}

pub struct GrpcRpcClient {
    client: GrpcClient,
}

impl GrpcRpcClient {
    pub async fn new(url: &str) -> Result<Self, GrpcError> {
        // Strip http:// or https:// prefix if present
        let url = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        
        // Add grpc:// prefix
        let url = format!("grpc://{}", url);

        let client = GrpcClient::connect_with_args(
            NotificationMode::Direct,
            url,
            None,
            true,
            None,
            false,
            Some(500_000),
            Default::default(),
        )
        .await
        .map_err(|e| GrpcError::Connection(e.to_string()))?;

        Ok(Self { client })
    }

    pub async fn get_balance(&self, address: &str) -> Result<u64, GrpcError> {
        let addr = Address::try_from(address.to_string())
            .map_err(|e| GrpcError::Connection(e.to_string()))?;
        let entries = self.client
            .get_utxos_by_addresses(vec![addr])
            .await
            .map_err(|e| GrpcError::Rpc(e.to_string()))?;

        let total: u64 = entries.iter().map(|entry| entry.utxo_entry.amount).sum();
        Ok(total)
    }

    pub async fn get_utxos(&self, address: &str) -> Result<Vec<GrpcUtxoInfo>, GrpcError> {
        let addr = Address::try_from(address.to_string())
            .map_err(|e| GrpcError::Connection(e.to_string()))?;
        let entries = self.client
            .get_utxos_by_addresses(vec![addr])
            .await
            .map_err(|e| GrpcError::Rpc(e.to_string()))?;

        let utxos: Vec<GrpcUtxoInfo> = entries.into_iter().map(|entry| {
            GrpcUtxoInfo {
                outpoint: format!("{}:{}", entry.outpoint.transaction_id, entry.outpoint.index),
                amount: entry.utxo_entry.amount,
                script_public_key: hex::encode(entry.utxo_entry.script_public_key.script()),
                is_coinbase: entry.utxo_entry.is_coinbase,
                blockDAAScore: entry.utxo_entry.block_daa_score,
            }
        }).collect();

        Ok(utxos)
    }

    pub async fn submit_transaction(&self, tx_hex: &str) -> Result<String, GrpcError> {
        let tx_bytes = hex::decode(tx_hex)
            .map_err(|e| GrpcError::Connection(e.to_string()))?;

        let tx: Transaction = borsh::BorshDeserialize::deserialize(&mut tx_bytes.as_slice())
            .map_err(|e| GrpcError::Rpc(e.to_string()))?;

        let txid = self.client
            .submit_transaction((&tx).into(), false)
            .await
            .map_err(|e| GrpcError::Rpc(e.to_string()))?;

        Ok(txid.to_string())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrpcUtxoInfo {
    pub outpoint: String,
    pub amount: u64,
    pub script_public_key: String,
    pub is_coinbase: bool,
    pub blockDAAScore: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrpcBalanceResponse {
    pub balance: u64,
}
