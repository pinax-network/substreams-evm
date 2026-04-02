use common::bigint_to_u64;
use substreams::scalar::BigInt;

pub(crate) fn is_non_zero(value: &str) -> bool {
    !value.is_empty() && value.bytes().any(|byte| byte != b'0')
}

pub(crate) fn fixed_1e18_to_bps(value: &BigInt) -> u32 {
    bigint_to_u64(value).unwrap_or_default().saturating_div(100_000_000_000_000) as u32
}
