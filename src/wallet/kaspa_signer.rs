use std::collections::BTreeMap;

use borsh::BorshSerialize;
use kaspa_addresses::Address;
use kaspa_consensus_core::sign::sign;
use kaspa_consensus_core::tx::{
    MutableTransaction, ScriptPublicKey, Transaction, TransactionId, TransactionInput,
    TransactionOutpoint, TransactionOutput, UtxoEntry,
};
use kaspa_txscript::pay_to_address_script;
use secp256k1::{Keypair, Secp256k1};
use serde::Serialize;

const SIG_HASH_ALL_U8: u8 = 0x01;

const MASS_PER_TX_BYTE: u64 = 1;
const MASS_PER_SCRIPT_PUB_KEY_BYTE: u64 = 10;
const MASS_PER_SIG_OP: u64 = 1000;

fn compute_transaction_mass(tx: &Transaction) -> u64 {
    let mut size: u64 = 0;
    size += 2;
    size += 8;
    for input in &tx.inputs {
        size += 32;
        size += 4;
        size += 8;
        size += input.signature_script.len() as u64;
        size += 8;
    }
    size += 8;
    for output in &tx.outputs {
        size += 8;
        size += 2;
        size += 8;
        size += output.script_public_key.script().len() as u64;
    }
    size += 8;
    size += 20;
    size += 8;
    size += tx.payload.len() as u64;

    let compute_mass_for_size = size * MASS_PER_TX_BYTE;
    let total_script_pub_key_size: u64 = tx
        .outputs
        .iter()
        .map(|output| 2 + output.script_public_key.script().len() as u64)
        .sum();
    let total_script_pub_key_mass = total_script_pub_key_size * MASS_PER_SCRIPT_PUB_KEY_BYTE;
    let total_sigops: u64 = tx
        .inputs
        .iter()
        .map(|input| input.sig_op_count as u64)
        .sum();
    let total_sigops_mass = total_sigops * MASS_PER_SIG_OP;

    compute_mass_for_size + total_script_pub_key_mass + total_sigops_mass
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTransactionInput {
    #[serde(rename = "previousOutpoint")]
    pub previous_outpoint: JsonOutPoint,
    #[serde(rename = "signatureScript")]
    pub signature_script: String,
    pub sequence: u64,
    #[serde(rename = "sigOpCount")]
    pub sig_op_count: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonOutPoint {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonScriptPublicKey {
    pub version: u16,
    #[serde(rename = "scriptPublicKey")]
    pub script: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTransactionOutput {
    #[serde(rename = "amount")]
    pub amount: u64,
    #[serde(rename = "scriptPublicKey")]
    pub script_public_key: JsonScriptPublicKey,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonTransaction {
    pub version: u32,
    pub inputs: Vec<JsonTransactionInput>,
    pub outputs: Vec<JsonTransactionOutput>,
    #[serde(rename = "lockTime")]
    pub lock_time: u64,
    #[serde(rename = "subnetworkId")]
    pub subnetwork_id: String,
    pub gas: u64,
    #[serde(rename = "payload")]
    pub payload: String,
    pub mass: u64,
}

#[derive(Debug, Clone)]
pub struct KaspaSignedTransaction {
    pub tx_hex: String,
    pub tx_id: String,
    pub json_tx: JsonTransaction,
}

impl KaspaSignedTransaction {
    pub fn hex(&self) -> &str {
        &self.tx_hex
    }

    pub fn id(&self) -> &str {
        &self.tx_id
    }

    pub fn json(&self) -> &JsonTransaction {
        &self.json_tx
    }
}

pub struct KaspaTransactionSigner {
    transaction: Transaction,
    utxos: Vec<UtxoEntry>,
}

impl KaspaTransactionSigner {
    pub fn new() -> Self {
        Self {
            transaction: Transaction::new(
                0,
                Vec::new(),
                Vec::new(),
                0,
                Default::default(),
                0,
                Vec::new(),
            ),
            utxos: Vec::new(),
        }
    }

    pub fn add_input(
        &mut self,
        txid: &str,
        vout: u32,
        amount: u64,
        script_pubkey: &[u8],
    ) -> Result<(), String> {
        let txid_bytes = hex::decode(txid).map_err(|e| format!("Invalid txid: {}", e))?;
        let txid_obj = TransactionId::from_bytes(
            txid_bytes
                .try_into()
                .map_err(|_| "Invalid txid length, expected 32 bytes")?,
        );

        let outpoint = TransactionOutpoint {
            transaction_id: txid_obj,
            index: vout,
        };

        let script_public_key = ScriptPublicKey::new(0, script_pubkey.to_vec().into());
        let utxo = UtxoEntry::new(amount, script_public_key.clone(), 0, false);

        self.utxos.push(utxo.clone());

        let input = TransactionInput {
            previous_outpoint: outpoint,
            signature_script: Vec::new(),
            sequence: 0,
            sig_op_count: 1,
        };

        self.transaction.inputs.push(input);

        Ok(())
    }

    pub fn add_output(&mut self, address: &str, amount: u64) -> Result<(), String> {
        let address = Address::try_from(address).map_err(|e| format!("Invalid address: {}", e))?;
        let script_pubkey = pay_to_address_script(&address);

        let output = TransactionOutput {
            value: amount,
            script_public_key: script_pubkey,
        };

        self.transaction.outputs.push(output);

        Ok(())
    }

    pub fn set_payload(&mut self, payload: &[u8]) {
        self.transaction.payload = payload.to_vec();
    }

    pub fn sign(&mut self, private_key: &[u8]) -> Result<KaspaSignedTransaction, String> {
        let secp = Secp256k1::new();
        let keypair = Keypair::from_seckey_slice(&secp, private_key)
            .map_err(|e| format!("Invalid private key: {}", e))?;

        let (xonly_pubkey, _) = keypair.x_only_public_key();
        let pubkey_bytes: [u8; 32] = xonly_pubkey.serialize();

        eprintln!("DEBUG: X-only public key: {}", hex::encode(&pubkey_bytes));

        // Create signable transaction with UTXO entries
        let mut signable_tx =
            MutableTransaction::with_entries(self.transaction.clone(), self.utxos.clone());

        // Sign using Kaspa's official sign function
        let signed_tx = sign(signable_tx.clone(), keypair);
        let tx = signed_tx.tx;

        // Serialize transaction using borsh
        let mut serialized = Vec::new();
        borsh::BorshSerialize::serialize(&tx, &mut serialized)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let tx_hex = hex::encode(&serialized);

        // Calculate transaction ID by finalizing the transaction
        let mut tx_final = tx.clone();
        tx_final.finalize();
        let tx_id = tx_final.id();
        let tx_id_hex = hex::encode(tx_id.as_bytes());

        eprintln!("DEBUG: Signed tx ID: {}", tx_id_hex);
        eprintln!("DEBUG: Signed tx hex length: {}", tx_hex.len());

        // Build JSON transaction for API submission
        let mut json_inputs = Vec::new();
        for input in &tx.inputs {
            json_inputs.push(JsonTransactionInput {
                previous_outpoint: JsonOutPoint {
                    transaction_id: hex::encode(input.previous_outpoint.transaction_id.as_bytes()),
                    index: input.previous_outpoint.index,
                },
                signature_script: hex::encode(&input.signature_script),
                sequence: input.sequence,
                sig_op_count: input.sig_op_count,
            });
        }

        let mut json_outputs = Vec::new();
        for output in &tx.outputs {
            json_outputs.push(JsonTransactionOutput {
                amount: output.value,
                script_public_key: JsonScriptPublicKey {
                    version: output.script_public_key.version(),
                    script: hex::encode(output.script_public_key.script()),
                },
            });
        }

        let json_tx = JsonTransaction {
            version: tx.version as u32,
            inputs: json_inputs,
            outputs: json_outputs,
            lock_time: tx.lock_time,
            subnetwork_id: format!("{}", tx.subnetwork_id),
            gas: 0,
            payload: hex::encode(&tx.payload),
            mass: compute_transaction_mass(&tx),
        };

        Ok(KaspaSignedTransaction {
            tx_hex,
            tx_id: tx_id_hex,
            json_tx,
        })
    }

    pub fn sign_no_payload(
        &mut self,
        private_key: &[u8],
    ) -> Result<KaspaSignedTransaction, String> {
        let secp = Secp256k1::new();
        let keypair = Keypair::from_seckey_slice(&secp, private_key)
            .map_err(|e| format!("Invalid private key: {}", e))?;

        let (xonly_pubkey, _) = keypair.x_only_public_key();
        let pubkey_bytes: [u8; 32] = xonly_pubkey.serialize();

        eprintln!(
            "DEBUG: X-only public key (transfer): {}",
            hex::encode(&pubkey_bytes)
        );

        // Create signable transaction with UTXO entries
        let mut signable_tx =
            MutableTransaction::with_entries(self.transaction.clone(), self.utxos.clone());

        // Sign using Kaspa's official sign function
        let signed_tx = sign(signable_tx.clone(), keypair);
        let tx = signed_tx.tx;

        // Serialize transaction using borsh
        let mut serialized = Vec::new();
        borsh::BorshSerialize::serialize(&tx, &mut serialized)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let tx_hex = hex::encode(&serialized);

        // Calculate transaction ID
        let mut tx_final = tx.clone();
        tx_final.finalize();
        let tx_id = tx_final.id();
        let tx_id_hex = hex::encode(tx_id.as_bytes());

        eprintln!("DEBUG: Signed tx ID (transfer): {}", tx_id_hex);
        eprintln!("DEBUG: Signed tx hex length (transfer): {}", tx_hex.len());

        let mut json_inputs = Vec::new();
        for input in &tx.inputs {
            json_inputs.push(JsonTransactionInput {
                previous_outpoint: JsonOutPoint {
                    transaction_id: hex::encode(input.previous_outpoint.transaction_id.as_bytes()),
                    index: input.previous_outpoint.index,
                },
                signature_script: hex::encode(&input.signature_script),
                sequence: input.sequence,
                sig_op_count: input.sig_op_count,
            });
        }

        let mut json_outputs = Vec::new();
        for output in &tx.outputs {
            json_outputs.push(JsonTransactionOutput {
                amount: output.value,
                script_public_key: JsonScriptPublicKey {
                    version: output.script_public_key.version(),
                    script: hex::encode(output.script_public_key.script()),
                },
            });
        }

        let json_tx = JsonTransaction {
            version: tx.version as u32,
            inputs: json_inputs,
            outputs: json_outputs,
            lock_time: tx.lock_time,
            subnetwork_id: format!("{}", tx.subnetwork_id),
            gas: 0,
            payload: String::new(),
            mass: compute_transaction_mass(&tx),
        };

        let json_tx_str = serde_json::to_string_pretty(&json_tx)
            .map_err(|e| format!("Failed to serialize JSON tx: {}", e))?;
        eprintln!("DEBUG: JSON transaction:\n{}", json_tx_str);

        Ok(KaspaSignedTransaction {
            tx_hex,
            tx_id: tx_id_hex,
            json_tx,
        })
    }
}
