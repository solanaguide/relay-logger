use log::{debug, info};
use solana_sdk::{
    transaction::VersionedTransaction,
    pubkey::Pubkey,
    message::{self, VersionedMessage, Message as LegacyMessage},
    instruction::CompiledInstruction,
    message::v0,
};

use std::net::IpAddr;

const LAMPORTS_PER_SIGNATURE: u64 = 5000;
const COMPUTE_BUDGET_PROGRAM_ID: Pubkey = solana_sdk::compute_budget::id();
const VOTE_PROGRAM_ID: Pubkey = solana_sdk::vote::program::id();  // Assuming the ID is imported correctly


trait MessageTrait {
    fn account_keys(&self) -> &[Pubkey];
    fn instructions(&self) -> &[CompiledInstruction];
    fn num_required_signatures(&self) -> u8;
    fn is_signer(&self, index: usize) -> bool;
}

impl MessageTrait for LegacyMessage {
    fn account_keys(&self) -> &[Pubkey] {
        &self.account_keys
    }

    fn instructions(&self) -> &[CompiledInstruction] {
        &self.instructions
    }

    fn num_required_signatures(&self) -> u8 {
        self.header.num_required_signatures
    }

    fn is_signer(&self, index: usize) -> bool {
        self.is_signer(index)
    }
}

impl MessageTrait for v0::Message {
    fn account_keys(&self) -> &[Pubkey] {
        &self.account_keys
    }

    fn instructions(&self) -> &[CompiledInstruction] {
        &self.instructions
    }

    fn num_required_signatures(&self) -> u8 {
        self.header.num_required_signatures
    }

    fn is_signer(&self, index: usize) -> bool {
        self.is_signer(index)
    }
}

pub fn log_tx_and_ip(tx: &VersionedTransaction, ip_addr: &IpAddr) {
    let message: &dyn MessageTrait = match &tx.message {
        VersionedMessage::Legacy(message) => message,
        VersionedMessage::V0(message) => message,
    };


    let num_signatures = tx.clone().message.header().num_required_signatures as u64;
    let hash = tx.signatures[0].to_string();


    let compute_budget_index = message.account_keys().iter().position(|key| key == &COMPUTE_BUDGET_PROGRAM_ID);

    let mut compute_unit_limit: u32 = 200_000; // default is 200k if not set
    let mut compute_unit_price: u64 = 0;

    if let Some(index) = compute_budget_index {
        for instruction in message.instructions() {
            if instruction.program_id_index as usize == index {
                match instruction.data.first() {
                    Some(&2) if instruction.data.len() >= 5 => {
                        compute_unit_limit = u32::from_le_bytes([
                            instruction.data[1], instruction.data[2], instruction.data[3], instruction.data[4]
                        ]);
                    }
                    Some(&3) if instruction.data.len() >= 9 => {
                        compute_unit_price = u64::from_le_bytes([
                            instruction.data[1], instruction.data[2], instruction.data[3], instruction.data[4],
                            instruction.data[5], instruction.data[6], instruction.data[7], instruction.data[8]
                        ]);
                    }
                    _ => {}
                }
            }
        }
    }

    // let compute_unit_price = compute_unit_price / 1_000_000u64;
    let priority_fee = (compute_unit_limit as u64 * compute_unit_price) / 1_000_000u64;
    let transaction_fee = priority_fee + (num_signatures * LAMPORTS_PER_SIGNATURE);
    let compute_fee_ratio = transaction_fee as f64 / (1f64 + compute_unit_limit as f64);


    // log the transaction to a file with the following format:
    let ip = ip_addr.to_string();

    // timestamp, hash, signer, compute_unit_limit, transaction_fee, compute_fee_ratio, ip_addr
    let log_line = format!("txlog,{},{},{},{},{}", hash, compute_unit_limit, transaction_fee, compute_fee_ratio, ip);
    info!("{}", log_line);
}
