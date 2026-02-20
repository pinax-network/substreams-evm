use substreams::{
    store::{StoreGet, StoreGetProto},
    Hex,
};

pub fn get_store_by_address<T: prost::Message + Default>(store: &StoreGetProto<T>, address: &Vec<u8>) -> Option<T> {
    store.get_first(Hex::encode(address))
}
