pub mod client;
pub mod grpc_client;

pub use client::{RpcClient, PUBLIC_TESTNET10_RPC, PUBLIC_TESTNET12_RPC};
pub use grpc_client::{GrpcRpcClient, GrpcUtxoInfo, GrpcBalanceResponse};
