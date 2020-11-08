use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, bucket_read,
                       Bucket, ReadonlyBucket};

pub static CONFIG_KEY: &[u8] = b"config";
static BALANCE_KEY: &[u8] = b"balance";


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Entry {
    pub weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub entries: Vec<(CanonicalAddr, Uint128)>,  // maps address to weight
    pub contract_owner: CanonicalAddr,
    pub seed: Vec<u8>,
    pub entropy: Vec<u8>,
    pub start_time: u64,
    pub winner1: CanonicalAddr,
    pub winner2: CanonicalAddr,
    pub winner3: CanonicalAddr,
    pub staked_tokens: Uint128,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn balance<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(BALANCE_KEY, storage)
}

pub fn balance_read<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(BALANCE_KEY, storage)
}
