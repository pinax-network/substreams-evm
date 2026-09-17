// SPDX-License-Identifier: MIT
// Integer port of the runtime-bound PowMath.pow998 and PRBMath UD60x18
// log2/pow/exp2 implementation. Source binding: fixture manifest. PRBMath's
// copyright and permission notice are in the adjacent PRBMath-LICENSE file.
// This is specialized to base 0.998 and integer exponents 0..=10000.
use super::{add, mul, unit};
use anyhow::{ensure, Result};
use primitive_types::U256;

// PRBMath Common.sol exp2 factors, in descending fractional-bit order.
const FACTORS: [u128; 64] = [
    0x16A09E667F3BCC909,
    0x1306FE0A31B7152DF,
    0x1172B83C7D517ADCE,
    0x10B5586CF9890F62A,
    0x1059B0D31585743AE,
    0x102C9A3E778060EE7,
    0x10163DA9FB33356D8,
    0x100B1AFA5ABCBED61,
    0x10058C86DA1C09EA2,
    0x1002C605E2E8CEC50,
    0x100162F3904051FA1,
    0x1000B175EFFDC76BA,
    0x100058BA01FB9F96D,
    0x10002C5CC37DA9492,
    0x1000162E525EE0547,
    0x10000B17255775C04,
    0x1000058B91B5BC9AE,
    0x100002C5C89D5EC6D,
    0x10000162E43F4F831,
    0x100000B1721BCFC9A,
    0x10000058B90CF1E6E,
    0x1000002C5C863B73F,
    0x100000162E430E5A2,
    0x1000000B172183551,
    0x100000058B90C0B49,
    0x10000002C5C8601CC,
    0x1000000162E42FFF0,
    0x10000000B17217FBB,
    0x1000000058B90BFCE,
    0x100000002C5C85FE3,
    0x10000000162E42FF1,
    0x100000000B17217F8,
    0x10000000058B90BFC,
    0x1000000002C5C85FE,
    0x100000000162E42FF,
    0x1000000000B17217F,
    0x100000000058B90C0,
    0x10000000002C5C860,
    0x1000000000162E430,
    0x10000000000B17218,
    0x1000000000058B90C,
    0x100000000002C5C86,
    0x10000000000162E43,
    0x100000000000B1721,
    0x10000000000058B91,
    0x1000000000002C5C8,
    0x100000000000162E4,
    0x1000000000000B172,
    0x100000000000058B9,
    0x10000000000002C5D,
    0x1000000000000162E,
    0x10000000000000B17,
    0x1000000000000058C,
    0x100000000000002C6,
    0x10000000000000163,
    0x100000000000000B1,
    0x10000000000000059,
    0x1000000000000002C,
    0x10000000000000016,
    0x1000000000000000B,
    0x10000000000000006,
    0x10000000000000003,
    0x10000000000000001,
    0x10000000000000001,
];

pub(super) fn pow998(day: u64) -> Result<U256> {
    let u = unit();
    if day == 0 {
        return Ok(u);
    }
    if day == 1 {
        return Ok(U256::from(998_000_000_000_000_000u64));
    }
    if day > 10000 {
        return Ok(U256::zero());
    }
    let inverse = mul(u, u)? / U256::from(998_000_000_000_000_000u64);
    let integer = (inverse / u).bits() - 1;
    let mut log = mul(integer.into(), u)?;
    let mut y = inverse >> integer;
    let mut delta = u / 2;
    while !delta.is_zero() {
        y = mul(y, y)? / u;
        if y >= mul(u, 2.into())? {
            log = add(log, delta)?;
            y >>= 1;
        }
        delta >>= 1;
    }
    let exponent = mul(log, day.into())?;
    let x = mul(exponent, U256::one() << 64)? / u;
    ensure!((x >> 64) < 192.into(), "exp2 input out of range");
    let mut result = U256::one() << 191;
    for (i, factor) in FACTORS.iter().enumerate() {
        if x.bit(63 - i) {
            result = mul(result, (*factor).into())? >> 64;
        }
    }
    result = mul(result, u)? >> (191 - (x >> 64).as_usize());
    Ok(mul(u, u)? / result)
}
