use std::collections::{HashMap, HashSet};

use prost::Message;
use prost_types::Any;
use proto::pb::dex::{foundational_store::v1 as foundational, swaps::v1 as pb};
use substreams::{
    pb::sf::substreams::foundational_store::model::v2::{QueriedEntries, ResponseCode},
    store::FoundationalStore,
};
use substreams_abis::dex::uniswap::v2 as uniswap_v2;
use substreams_ethereum::{pb::eth::v2::Block, Event};

type PoolMetadata = foundational::Pool;
type PoolMetadataMap = HashMap<Vec<u8>, PoolMetadata>;

#[substreams::handlers::map]
pub fn map_events(block: Block, pools: FoundationalStore) -> Result<pb::Events, substreams::errors::Error> {
    let pool_addresses = collect_uniswap_v2_swap_pool_addresses(&block);
    let pools = get_pools_by_address(&pools, &pool_addresses);

    Ok(pb::Events {
        transactions: block
            .transactions()
            .filter_map(|trx| normalize_uniswap_v2_transaction(trx, &pools))
            .collect(),
    })
}

fn normalize_uniswap_v2_transaction(
    trx: &substreams_ethereum::pb::eth::v2::TransactionTrace,
    pools: &PoolMetadataMap,
) -> Option<pb::Transaction> {
    let swaps = trx
        .receipt()
        .logs()
        .filter_map(|log_view| {
            let event = uniswap_v2::pair::events::Swap::match_and_decode(log_view.log)?;
            let pool = pools.get(log_view.log.address.as_slice())?;
            normalize_uniswap_v2_swap(log_view.log.address.as_slice(), log_view.log.ordinal, trx.from.as_slice(), &event, pool)
        })
        .collect::<Vec<_>>();

    (!swaps.is_empty()).then(|| pb::Transaction {
        hash: trx.hash.to_vec(),
        from: trx.from.to_vec(),
        to: (!trx.to.is_empty()).then(|| trx.to.to_vec()),
        nonce: trx.nonce,
        gas_price: trx.clone().gas_price.unwrap_or_default().with_decimal(0).to_string(),
        gas_limit: trx.gas_limit,
        gas_used: trx.receipt().receipt.cumulative_gas_used,
        value: trx.clone().value.unwrap_or_default().with_decimal(0).to_string(),
        swaps,
    })
}

fn normalize_uniswap_v2_swap(
    pool_address: &[u8],
    log_ordinal: u64,
    user: &[u8],
    event: &uniswap_v2::pair::events::Swap,
    pool: &PoolMetadata,
) -> Option<pb::Swap> {
    let token0 = pool.tokens.first()?;
    let token1 = pool.tokens.get(1)?;

    let amount0_in = is_non_zero_amount(&event.amount0_in.to_string());
    let amount0_out = is_non_zero_amount(&event.amount0_out.to_string());
    let amount1_in = is_non_zero_amount(&event.amount1_in.to_string());
    let amount1_out = is_non_zero_amount(&event.amount1_out.to_string());

    let (input_token, input_amount, output_token, output_amount) = match (amount0_in, amount0_out, amount1_in, amount1_out) {
        (true, false, false, true) => (
            token0.clone(),
            event.amount0_in.to_string(),
            token1.clone(),
            event.amount1_out.to_string(),
        ),
        (false, true, true, false) => (
            token1.clone(),
            event.amount1_in.to_string(),
            token0.clone(),
            event.amount0_out.to_string(),
        ),
        _ => return None,
    };

    Some(pb::Swap {
        protocol: pb::Protocol::UniswapV2 as i32,
        factory: pool.factory.clone(),
        pool: pool_address.to_vec(),
        user: user.to_vec(),
        input_token,
        input_amount,
        output_token,
        output_amount,
        log_ordinal,
    })
}

fn collect_uniswap_v2_swap_pool_addresses(block: &Block) -> HashSet<Vec<u8>> {
    block
        .transactions()
        .flat_map(|trx| trx.receipt().logs())
        .filter_map(|log_view| {
            uniswap_v2::pair::events::Swap::match_and_decode(log_view.log).map(|_| log_view.log.address.to_vec())
        })
        .collect()
}

fn get_pools_by_address(store: &FoundationalStore, addresses: &HashSet<Vec<u8>>) -> PoolMetadataMap {
    if addresses.is_empty() {
        return PoolMetadataMap::default();
    }

    let keys = addresses.iter().cloned().collect::<Vec<_>>();
    let queried = store.get_first(&keys);
    decode_queried_pools(keys, queried)
}

fn decode_queried_pools(keys: Vec<Vec<u8>>, queried: QueriedEntries) -> PoolMetadataMap {
    keys.into_iter()
        .zip(queried.entries)
        .filter_map(|(key, queried)| decode_queried_pool(queried).map(|pool| (key, pool)))
        .collect()
}

fn decode_queried_pool(queried: substreams::pb::sf::substreams::foundational_store::model::v2::QueriedEntry) -> Option<PoolMetadata> {
    if queried.code != ResponseCode::Found as i32 {
        return None;
    }

    queried.entry.and_then(|entry| entry.value).and_then(decode_pool)
}

fn decode_pool(value: Any) -> Option<PoolMetadata> {
    (value.type_url == "type.googleapis.com/dex.foundational_store.v1.Pool").then_some(())?;
    PoolMetadata::decode(value.value.as_slice()).ok()
}

fn is_non_zero_amount(value: &str) -> bool {
    !value.is_empty() && value.bytes().any(|byte| byte != b'0')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(token0: u8, token1: u8, factory: u8) -> PoolMetadata {
        PoolMetadata {
            tokens: vec![vec![token0], vec![token1]],
            factory: vec![factory],
        }
    }

    fn swap(amount0_in: u64, amount0_out: u64, amount1_in: u64, amount1_out: u64) -> uniswap_v2::pair::events::Swap {
        uniswap_v2::pair::events::Swap {
            sender: Default::default(),
            amount0_in: amount0_in.into(),
            amount0_out: amount0_out.into(),
            amount1_in: amount1_in.into(),
            amount1_out: amount1_out.into(),
            to: Default::default(),
        }
    }

    #[test]
    fn normalizes_token0_to_token1_swaps() {
        let pool = pool(0xaa, 0xbb, 0xcc);
        let event = swap(15, 0, 0, 7);

        let normalized = normalize_uniswap_v2_swap(&[0x11], 42, &[0x99], &event, &pool).unwrap();

        assert_eq!(normalized.protocol, pb::Protocol::UniswapV2 as i32);
        assert_eq!(normalized.factory, vec![0xcc]);
        assert_eq!(normalized.pool, vec![0x11]);
        assert_eq!(normalized.user, vec![0x99]);
        assert_eq!(normalized.input_token, vec![0xaa]);
        assert_eq!(normalized.input_amount, "15");
        assert_eq!(normalized.output_token, vec![0xbb]);
        assert_eq!(normalized.output_amount, "7");
        assert_eq!(normalized.log_ordinal, 42);
    }

    #[test]
    fn normalizes_token1_to_token0_swaps() {
        let pool = pool(0xaa, 0xbb, 0xcc);
        let event = swap(0, 9, 4, 0);

        let normalized = normalize_uniswap_v2_swap(&[0x11], 42, &[0x99], &event, &pool).unwrap();

        assert_eq!(normalized.input_token, vec![0xbb]);
        assert_eq!(normalized.input_amount, "4");
        assert_eq!(normalized.output_token, vec![0xaa]);
        assert_eq!(normalized.output_amount, "9");
    }

    #[test]
    fn skips_ambiguous_uniswap_v2_swaps() {
        let pool = pool(0xaa, 0xbb, 0xcc);
        let event = swap(5, 1, 0, 9);

        assert!(normalize_uniswap_v2_swap(&[0x11], 42, &[0x99], &event, &pool).is_none());
    }

    #[test]
    fn skips_swaps_when_pool_tokens_are_incomplete() {
        let pool = PoolMetadata { tokens: vec![vec![0xaa]], factory: vec![0xcc] };
        let event = swap(15, 0, 0, 7);

        assert!(normalize_uniswap_v2_swap(&[0x11], 42, &[0x99], &event, &pool).is_none());
    }

    #[test]
    fn treats_all_zero_strings_as_zero_amounts() {
        assert!(!is_non_zero_amount("0"));
        assert!(!is_non_zero_amount("0000"));
        assert!(is_non_zero_amount("10"));
    }
}
