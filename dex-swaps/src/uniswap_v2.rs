use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::uniswap::v2 as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::{PoolMetadata, PoolMetadataMap};

pub(crate) fn decode_swap(tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    let event = abi::pair::events::Swap::match_and_decode(log)?;
    let pool = pools.get(log.address.as_slice())?;
    normalize_swap(tx.from.as_slice(), log.address.as_slice(), log.ordinal, pool, &event)
}

fn normalize_swap(
    user: &[u8],
    pool_address: &[u8],
    log_ordinal: u64,
    pool: &PoolMetadata,
    event: &abi::pair::events::Swap,
) -> Option<pb::Swap> {
    let token0 = pool.tokens.first()?;
    let token1 = pool.tokens.get(1)?;

    let amount0_in = event.amount0_in.to_string();
    let amount0_out = event.amount0_out.to_string();
    let amount1_in = event.amount1_in.to_string();
    let amount1_out = event.amount1_out.to_string();

    let (input_token, input_amount, output_token, output_amount) = match (
        is_non_zero_amount(&amount0_in),
        is_non_zero_amount(&amount0_out),
        is_non_zero_amount(&amount1_in),
        is_non_zero_amount(&amount1_out),
    ) {
        (true, false, false, true) => (token0.clone(), amount0_in, token1.clone(), amount1_out),
        (false, true, true, false) => (token1.clone(), amount1_in, token0.clone(), amount0_out),
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

    fn swap(amount0_in: u64, amount0_out: u64, amount1_in: u64, amount1_out: u64) -> abi::pair::events::Swap {
        abi::pair::events::Swap {
            sender: Default::default(),
            amount0_in: amount0_in.into(),
            amount0_out: amount0_out.into(),
            amount1_in: amount1_in.into(),
            amount1_out: amount1_out.into(),
            to: Default::default(),
        }
    }

    #[test]
    fn decodes_token0_to_token1_swap() {
        let normalized = normalize_swap(&[0x99], &[0x11], 42, &pool(0xaa, 0xbb, 0xcc), &swap(15, 0, 0, 7)).unwrap();

        assert_eq!(normalized.protocol, pb::Protocol::UniswapV2 as i32);
        assert_eq!(normalized.factory, vec![0xcc]);
        assert_eq!(normalized.pool, vec![0x11]);
        assert_eq!(normalized.user, vec![0x99]);
        assert_eq!(normalized.input_token, vec![0xaa]);
        assert_eq!(normalized.input_amount, "15");
        assert_eq!(normalized.output_token, vec![0xbb]);
        assert_eq!(normalized.output_amount, "7");
    }

    #[test]
    fn skips_ambiguous_swap_shapes() {
        assert!(normalize_swap(&[0x99], &[0x11], 42, &pool(0xaa, 0xbb, 0xcc), &swap(5, 1, 0, 9)).is_none());
    }
}
