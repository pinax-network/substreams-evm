use proto::pb::{justswap, sunpump, sunswap, uniswap};
use substreams::{
    store::{StoreGet, StoreGetProto},
    Hex,
};

// Generic functions that work based on context
pub fn get_new_exchange<T: prost::Message + Default>(store: &StoreGetProto<T>, address: &Vec<u8>) -> Option<T> {
    store.get_first(Hex::encode(address))
}

pub fn get_pair_created<T: prost::Message + Default>(store: &StoreGetProto<T>, address: &Vec<u8>) -> Option<T> {
    store.get_first(Hex::encode(address))
}

pub fn get_token_create(store: &StoreGetProto<sunpump::v1::TokenCreate>, address: &Vec<u8>) -> Option<sunpump::v1::TokenCreate> {
    store.get_first(Hex::encode(address))
}
