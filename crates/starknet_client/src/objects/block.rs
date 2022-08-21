use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use starknet_api::{
    BlockHash, BlockNumber, BlockTimestamp, ClassHash, ContractAddress, DeployedContract, GasPrice,
    GlobalRoot, StorageDiff, StorageEntry,
};

use super::transaction::{Transaction, TransactionReceipt};

/// A block as returned by the starknet gateway.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct Block {
    // TODO(dan): Currently should be Option<BlockHash> (due to pending blocks).
    // Figure out if we want this in the internal representation as well.
    pub block_hash: BlockHash,
    pub block_number: BlockNumber,
    pub gas_price: GasPrice,
    pub parent_block_hash: BlockHash,
    #[serde(default)]
    pub sequencer_address: ContractAddress,
    pub state_root: GlobalRoot,
    pub status: BlockStatus,
    #[serde(default)]
    pub timestamp: BlockTimestamp,
    pub transactions: Vec<Transaction>,
    pub transaction_receipts: Vec<TransactionReceipt>,
}

/// A state update derived from a single block as returned by the starknet gateway.
#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct BlockStateUpdate {
    pub block_hash: BlockHash,
    pub new_root: GlobalRoot,
    pub old_root: GlobalRoot,
    pub state_diff: StateDiff,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub enum BlockStatus {
    #[serde(rename(deserialize = "ABORTED", serialize = "ABORTED"))]
    Aborted,
    #[serde(rename(deserialize = "ACCEPTED_ON_L1", serialize = "ACCEPTED_ON_L1"))]
    AcceptedOnL1,
    #[serde(rename(deserialize = "ACCEPTED_ON_L2", serialize = "ACCEPTED_ON_L2"))]
    AcceptedOnL2,
    #[serde(rename(deserialize = "PENDING", serialize = "PENDING"))]
    Pending,
    #[serde(rename(deserialize = "REVERTED", serialize = "REVERTED"))]
    Reverted,
}
impl Default for BlockStatus {
    fn default() -> Self {
        BlockStatus::AcceptedOnL2
    }
}

impl From<BlockStatus> for starknet_api::BlockStatus {
    fn from(status: BlockStatus) -> Self {
        match status {
            BlockStatus::Aborted => starknet_api::BlockStatus::Rejected,
            BlockStatus::AcceptedOnL1 => starknet_api::BlockStatus::AcceptedOnL1,
            BlockStatus::AcceptedOnL2 => starknet_api::BlockStatus::AcceptedOnL2,
            BlockStatus::Pending => starknet_api::BlockStatus::Pending,
            BlockStatus::Reverted => starknet_api::BlockStatus::Rejected,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct StateDiff {
    // BTreeMap is serialized as a mapping in json, keeps ordering and is efficiently iterable.
    pub storage_diffs: BTreeMap<ContractAddress, Vec<StorageEntry>>,
    pub deployed_contracts: Vec<DeployedContract>,
    #[serde(default)]
    pub declared_classes: Vec<ClassHash>,
}
impl StateDiff {
    pub fn class_hashes(&self) -> Vec<ClassHash> {
        // TODO(yair): both vectors should be sorted, so merge sorted vecs instead of appending and
        // then sorting.
        let mut res = self.declared_classes.clone();
        res.append(
            &mut self.deployed_contracts.iter().map(|contract| contract.class_hash).collect(),
        );
        res.sort();
        res.dedup();
        res
    }
}

/// Converts the client representation of [`BlockStateUpdate`] storage diffs to a [`starknet_api`]
/// [`StorageDiff`].
pub fn client_to_starknet_api_storage_diff(
    storage_diffs: BTreeMap<ContractAddress, Vec<StorageEntry>>,
) -> Vec<StorageDiff> {
    storage_diffs.into_iter().map(|(address, diff)| StorageDiff { address, diff }).collect()
}
