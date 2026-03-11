use common::{bytes_to_string, Encoding};
use prost_types::Any;
use proto::pb::dex::foundational_store::v1::Pool;
use substreams::{
    pb::sf::substreams::foundational_store::model::v2::ResponseCode,
    store::FoundationalStore,
};

pub type PoolMetadata = Pool;

pub fn get_pool_by_address(store: &FoundationalStore, address: &Vec<u8>) -> Option<PoolMetadata> {
    store
        .get_first(&[address.as_slice()])
        .entries
        .into_iter()
        .next()
        .and_then(|queried| {
            if queried.code != ResponseCode::Found as i32 {
                return None;
            }

            queried.entry.and_then(|entry| entry.value).and_then(decode_pool)
        })
}

pub fn token(pool: &PoolMetadata, index: usize) -> &[u8] {
    pool.tokens.get(index).map(Vec::as_slice).unwrap_or_default()
}

pub fn tokens_csv(encoding: &Encoding, pool: &PoolMetadata) -> String {
    pool.tokens.iter().map(|token| bytes_to_string(token, encoding)).collect::<Vec<_>>().join(",")
}

fn decode_pool(value: Any) -> Option<PoolMetadata> {
    value.to_msg::<PoolMetadata>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_pool_accepts_foundational_any_payload() {
        let pool = PoolMetadata { tokens: vec![vec![0xaa], vec![0xbb]], factory: vec![0xcc] };
        let any = Any::from_msg(&pool).unwrap();

        let decoded = decode_pool(any).unwrap();
        assert_eq!(decoded.tokens, pool.tokens);
        assert_eq!(decoded.factory, pool.factory);
    }

    #[test]
    fn decode_pool_rejects_wrong_type_url() {
        let any = Any { type_url: "type.googleapis.com/google.protobuf.Empty".into(), value: vec![] };
        assert!(decode_pool(any).is_none());
    }
}
