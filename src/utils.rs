use core::option;
use eyre::Result;

use ethers::types::{
    transaction::{
        eip2718::TypedTransaction as EthersTypedTransaction,
        eip2930::{
            AccessList as EthersAccessList, AccessListItem as EthersAccessListItem,
            AccessListWithGasUsed as EthersAccessListWithGasUsed,
        },
    },
    Address as EthersAddress, BlockId as EthersBlockId, BlockNumber as EthersBlockNumber,
    Bloom as EthersBloom, Filter as EthersFilter, FilterBlockOption as EthersFilterBlockOption,
    Log as EthersLog, NameOrAddress as EthersNameOrAddress, OtherFields, Topic as EthersTopic,
    Transaction as EthersTransaction, TransactionReceipt as EthersTransactionReceipt,
    ValueOrArray as EthersValueOrArray, H256 as EthersH256, U256 as EthersU256, U64 as EthersU64,EIP1186ProofResponse as EthersEIP1186ProofResponse,
    FeeHistory as EthersFeeHistory
};

use reth_primitives::{
    serde_helper::JsonStorageKey, AccessList, AccessListItem, AccessListWithGasUsed, Address,
    BlockHash, BlockId, BlockNumberOrTag, Bloom, Bytes, H256, U256, U8, U128, U64
};

use reth_rpc_types::{
    CallRequest, Filter, FilterBlockOption, Log, Topic, TransactionReceipt, ValueOrArray, Transaction
};

use reth_revm::{
    precompile::B160,
    primitives::ruint::{self, Bits, Uint},
};
use reth_rpc_types::EIP1186AccountProofResponse;

pub trait ToEthers<T> {
    /// Reth -> Ethers
    fn into_ethers(self) -> T;
}

pub trait ToReth<T> {
    /// Reth -> Ethers
    fn into_reth(self) -> T;
}


impl ToEthers<EthersU64> for U256 {
    fn into_ethers(self) -> EthersU64 {
        self.to_le_bytes().into()
    }
}

impl ToEthers<EthersU64> for U8 {
    fn into_ethers(self) -> EthersU64 {
        self.to_le_bytes().into()
    }
}

impl ToEthers<EthersU256> for U128 {
    fn into_ethers(self) -> EthersU256 {
        self.to_le_bytes().into()
    }
}

impl ToEthers<EthersU256> for U64 {
    fn into_ethers(self) -> EthersU256 {
        self.as_u64().into()
    }
}

impl ToEthers<EthersBloom> for Bloom {
    fn into_ethers(self) -> EthersBloom {
        self.to_fixed_bytes().into()
    }
}

impl ToEthers<EthersLog> for Log {
    fn into_ethers(self) -> EthersLog {
        EthersLog {
            address: self.address.into(),
            topics: self.topics.into_iter().map(|topic| topic.into()).collect(),
            data: self.data.to_vec().into(),
            block_hash: self.block_hash.map(|hash| hash.into()),
            block_number: self.block_number.map(|num| num.to_le_bytes().into()),
            transaction_hash: self.transaction_hash.map(|hash| hash.into()),
            transaction_index: self.transaction_index.map(|idx| idx.to_le_bytes().into()),
            log_index: self.log_index.map(|idx| idx.into()),
            transaction_log_index: todo!(),
            log_type: todo!(),
            removed: Some(self.removed),
        }
    }
}



pub fn ethers_block_id_to_reth_block_id(block_id: EthersBlockId) -> BlockId {
    match block_id {
        EthersBlockId::Hash(hash) => BlockId::Hash(BlockHash::from_slice(hash.as_bytes()).into()),
        EthersBlockId::Number(number) => {
            BlockId::Number(BlockNumberOrTag::Number(number.as_number().unwrap().as_u64()))
        }
    }
}

//Access List Conversion
pub fn ethers_access_list_to_reth_access_list(access_list: EthersAccessList) -> AccessList {
    AccessList(
        access_list
            .0
            .into_iter()
            .map(|item| AccessListItem {
                address: Address::from_slice(item.address.as_bytes()),
                storage_keys: item
                    .storage_keys
                    .into_iter()
                    .map(|key| H256::from_slice(key.as_bytes()))
                    .collect(),
            })
            .collect(),
    )
}

pub fn reth_access_list_to_ethers_access_list(access_list: AccessList) -> EthersAccessList {
    EthersAccessList(
        access_list
            .0
            .into_iter()
            .map(|item| EthersAccessListItem {
                address: EthersAddress::from_slice(item.address.as_bytes()),
                storage_keys: item
                    .storage_keys
                    .into_iter()
                    .map(|key| EthersH256::from_slice(key.as_bytes()))
                    .collect(),
            })
            .collect(),
    )
}

pub fn opt_reth_access_list_to_ethers_access_list(
    opt_access_list: Option<Vec<reth_primitives::AccessListItem>>,
) -> EthersAccessList {
    let access_list = opt_access_list.unwrap_or_else(Vec::new);
    EthersAccessList(
        access_list
            .into_iter()
            .map(|item| EthersAccessListItem {
                address: EthersAddress::from_slice(item.address.as_bytes()),
                storage_keys: item
                    .storage_keys
                    .into_iter()
                    .map(|key| EthersH256::from_slice(key.as_bytes()))
                    .collect(),
            })
            .collect(),
    )
}

pub fn reth_access_list_with_gas_used_to_ethers(
    access_list_with_gas_used: AccessListWithGasUsed,
) -> EthersAccessListWithGasUsed {
    EthersAccessListWithGasUsed {
        access_list: EthersAccessList(
            access_list_with_gas_used
                .access_list
                .0
                .into_iter()
                .map(|item| EthersAccessListItem {
                    address: ethers::types::Address::from_slice(item.address.as_bytes()),
                    storage_keys: item
                        .storage_keys
                        .into_iter()
                        .map(|key| ethers::types::H256::from_slice(key.as_bytes()))
                        .collect(),
                })
                .collect(),
        ),
        gas_used: access_list_with_gas_used.gas_used.into(),
    }
}

pub fn ethers_typed_transaction_to_reth_call_request(tx: &EthersTypedTransaction) -> CallRequest {
    CallRequest {
        from: Some(tx.from.into()),
        to: tx.to.map(|addr| addr.into()),
        gas_price: tx.gas_price.map(|gas| gas.into()),
        max_fee_per_gas: tx.max_fee_per_gas.map(|gas| gas.into()),
        max_priority_fee_per_gas: tx.max_priority_fee_per_gas.map(|gas| gas.into()),
        gas: Some(tx.gas.into()),
        value: Some(tx.value.into()),
        data: Some(tx.input.to_vec().into()),
        nonce: Some(tx.nonce.into()),
        chain_id: tx.chain_id.map(|id| id.as_u64().into()),
        access_list: tx
            .access_list
            .map(|list| ethers_access_list_to_reth_access_list(list.clone())),
        transaction_type: tx.transaction_type.map(|t| t.into())
    }
}

pub fn reth_rpc_transaction_to_ethers(reth_tx: Transaction) -> EthersTransaction {
    let v = reth_tx.signature.map_or(0.into(), |sig| sig.v.into_ethers());
    let r = reth_tx.signature.map_or(0.into(), |sig| sig.r.into());
    let s = reth_tx.signature.map_or(0.into(), |sig| sig.s.into());

    EthersTransaction {
        hash: reth_tx.hash.into(),
        nonce: reth_tx.nonce.into(),
        block_hash: reth_tx.block_hash.map(|hash| hash.into()),
        block_number: reth_tx.block_number.map(|n| n.into_ethers()),
        transaction_index: reth_tx.transaction_index.map(|n| n.into_ethers()),
        from: reth_tx.from.into(),
        to: reth_tx.to.map(|t| t.into()),
        value: reth_tx.value.into(),
        gas_price: reth_tx.gas_price.map(|p| p.into_ethers()),
        gas: reth_tx.gas.into(),
        input: reth_tx.input.to_vec().into(),
        v,
        r,
        s,
        transaction_type: reth_tx.transaction_type,
        access_list: Some(opt_reth_access_list_to_ethers_access_list(reth_tx.access_list)),
        max_priority_fee_per_gas: reth_tx.max_priority_fee_per_gas.map(|p| p.into_ethers()),
        max_fee_per_gas: reth_tx.max_fee_per_gas.map(|p| p.into_ethers()),
        chain_id: reth_tx.chain_id.map(|id| id.into_ethers()),
        ..Default::default()
    }
}



fn convert_block_number_to_block_number_or_tag(
    block: EthersBlockNumber,
) -> Result<BlockNumberOrTag> {
    match block {
        ethers::types::BlockNumber::Latest => Ok(BlockNumberOrTag::Latest),
        ethers::types::BlockNumber::Finalized => Ok(BlockNumberOrTag::Finalized),
        ethers::types::BlockNumber::Safe => Ok(BlockNumberOrTag::Safe),
        ethers::types::BlockNumber::Earliest => Ok(BlockNumberOrTag::Earliest),
        ethers::types::BlockNumber::Pending => Ok(BlockNumberOrTag::Pending),
        ethers::types::BlockNumber::Number(n) => Ok(BlockNumberOrTag::Number(n.as_u64())),
    }
}

fn convert_topics(topics: [Option<EthersTopic>; 4]) -> [Option<Topic>; 4] {
    let mut new_topics: Vec<Option<Topic>> = Vec::new();

    for (i, topic) in topics.into_iter().enumerate() {
        new_topics[i] = topic.as_ref().map(&option_convert_valueORarray).clone();
    }

    new_topics.try_into().unwrap()
}

/// ---------------------------

// need to generalize the following 2 functions

fn option_convert_valueORarray<T, U>(val: &EthersValueOrArray<Option<T>>) -> ValueOrArray<Option<U>>
where
    T: Clone,
    U: From<T>,
{
    match val {
        EthersValueOrArray::Value(Some(addr)) => ValueOrArray::Value(Some(addr.clone().into())),
        EthersValueOrArray::Value(None) => ValueOrArray::Value(None),
        EthersValueOrArray::Array(addrs) => {
            ValueOrArray::Array(addrs.into_iter().map(|a| a.map(U::from)).collect())
        }
    }
}

fn convert_valueORarray<T, U>(val: &EthersValueOrArray<T>) -> ValueOrArray<U>
where
    T: Clone,
    U: From<T>,
{
    match val {
        EthersValueOrArray::Value(addr) => ValueOrArray::Value(addr.clone().into()),
        EthersValueOrArray::Array(addrs) => {
            ValueOrArray::Array(addrs.into_iter().map(|a| Into::<U>::into(a.clone())).collect())
        }
    }
}

/// ---------------------------

pub fn ethers_filter_to_reth_filter(filter: &EthersFilter) -> Filter {
    return Filter {
        block_option: match filter.block_option {
            EthersFilterBlockOption::AtBlockHash(x) => FilterBlockOption::AtBlockHash(x.into()),
            EthersFilterBlockOption::Range { from_block, to_block } => FilterBlockOption::Range {
                from_block: convert_block_number_to_block_number_or_tag(from_block.unwrap()).ok(),
                to_block: convert_block_number_to_block_number_or_tag(to_block.unwrap()).ok(),
            },
        },

        address: match &filter.address {
            Some(addr) => Some(convert_valueORarray(addr)),
            None => None,
        },

        topics: convert_topics(filter.topics),
    }
}

pub fn reth_rpc_log_to_ethers(log: Log) -> EthersLog {
    EthersLog {
        address: log.address.into(),
        topics: log.topics.into_iter().map(|topic| topic.into()).collect(),
        data: log.data.to_vec().into(),
        block_hash: log.block_hash.map(|hash| hash.into()),
        block_number: log.block_number.map(|num| num.to_le_bytes().into()),
        transaction_hash: log.transaction_hash.map(|hash| hash.into()),
        transaction_index: log.transaction_index.map(|idx| idx.to_le_bytes().into()),
        log_index: log.log_index.map(|idx| idx.into()),
        transaction_log_index: todo!(),
        log_type: todo!(),
        removed: Some(log.removed),
    }
}


pub fn reth_transaction_receipt_to_ethers(receipt: TransactionReceipt) -> EthersTransactionReceipt {
    EthersTransactionReceipt {
        transaction_hash: receipt.transaction_hash.unwrap().into(),
        transaction_index: receipt.transaction_index.unwrap().into_ethers(),
        block_hash: receipt.block_hash.map(|hash| hash.into()),
        block_number: receipt.block_number.map(|num| num.into_ethers()),
        from: receipt.from.into(),
        to: receipt.to.map(|t| t.into()),
        cumulative_gas_used: receipt.cumulative_gas_used.into(),
        gas_used: receipt.gas_used.map(|gas| gas.into()),
        contract_address: receipt.contract_address.map(|addr| addr.into()),
        logs: receipt.logs.into_iter().map(|log| log.into_ethers()).collect(),
        status: receipt.status_code.map(|num| num.as_u64().into()),
        root: receipt.state_root.map(|root| root.into()),
        logs_bloom: receipt.logs_bloom.into_ethers(),
        transaction_type: Some(receipt.transaction_type.into_ethers()),
        effective_gas_price: Some(U256::from(receipt.effective_gas_price).into()),
        other: OtherFields::default(),
    }
}


pub fn reth_proof_to_ethers(proof: EIP1186AccountProofResponse) -> EthersEIP1186ProofResponse {}


pub fn reth_fee_history_to_ethers(fee_history: FeeHistory) -> EthersFeeHistory {}


pub fn convert_location_to_json_key(location: EthersH256) -> JsonStorageKey {
    let location = location.to_fixed_bytes();
    let location_u256: U256 = U256::from_be_bytes(location);
    JsonStorageKey::from(location_u256)
}


pub fn convert_Ethers_U256_to_Reth_U64(u256: EthersU256) -> U64 {
    let u256 = u256.as_u64();
    u256.into()
}

pub fn convert_Reth_U256_to_Ethers_U64(u256: U256) -> EthersU64 {
    let u256: EthersU256 = u256.into();
    let u256 = u256.as_u64(); 
    u256.into()
}


pub fn convert_Reth_U64_to_Ethers_U256(u64: U64) -> EthersU256 {
    let u64t = u64.as_u64(); 
    u64t.into()
}

