use proto::pb::{aerodrome, balancer, bancor, cow, curvefi, dodo, kyber_elastic, sunpump, traderjoe, uniswap, woofi};
use proto::pb::dex::v1 as dex;
use substreams::Hex;
use substreams::errors::Error;
use substreams::prelude::*;
use substreams_ethereum::NULL_ADDRESS;

// ── map_dex_swaps ─────────────────────────────────────────────────────────────

/// Aggregates and normalizes swap events from all supported DEX protocols into
/// a unified `DexSwaps` output.
///
/// Each `DexSwap` has a single directional `input_contract → output_contract`
/// pair with corresponding amounts.  Bi-directional/flash swaps (e.g. Uniswap
/// V2 flash swaps with both `amount0_in > 0` and `amount1_in > 0`) are emitted
/// as `DexSwapFlash` entries and kept separate from the normalized table.
#[substreams::handlers::map]
#[allow(clippy::too_many_arguments)]
pub fn map_dex_swaps(
    // Tron DEX
    events_sunpump: sunpump::v1::Events,
    store_sunpump: StoreGetProto<sunpump::v1::StorePool>,
    // Ethereum DEX
    events_balancer: balancer::v1::Events,
    events_bancor: bancor::v1::Events,
    store_bancor: StoreGetProto<bancor::v1::StorePool>,
    events_cow: cow::v1::Events,
    events_curvefi: curvefi::v1::Events,
    store_curvefi: StoreGetProto<curvefi::v1::StorePool>,
    // New DEX
    events_aerodrome: aerodrome::v1::Events,
    store_aerodrome: StoreGetProto<aerodrome::v1::StorePool>,
    events_dodo: dodo::v1::Events,
    events_woofi: woofi::v1::Events,
    events_traderjoe: traderjoe::v1::Events,
    store_traderjoe: StoreGetProto<traderjoe::v1::StorePool>,
    events_kyber_elastic: kyber_elastic::v1::Events,
    store_kyber_elastic: StoreGetProto<kyber_elastic::v1::StorePool>,
    // Uniswap DEX
    events_uniswap_v1: uniswap::v1::Events,
    store_uniswap_v1: StoreGetProto<uniswap::v1::StorePool>,
    events_uniswap_v2: uniswap::v2::Events,
    store_uniswap_v2: StoreGetProto<uniswap::v2::StorePool>,
    events_uniswap_v3: uniswap::v3::Events,
    store_uniswap_v3: StoreGetProto<uniswap::v3::StorePool>,
    events_uniswap_v4: uniswap::v4::Events,
    store_uniswap_v4: StoreGetProto<uniswap::v4::StorePool>,
) -> Result<dex::DexSwaps, Error> {
    let mut out = dex::DexSwaps::default();

    extract_sunpump_swaps(&mut out, &events_sunpump, &store_sunpump);
    extract_balancer_swaps(&mut out.swaps, &events_balancer);
    extract_bancor_swaps(&mut out.swaps, &events_bancor, &store_bancor);
    extract_cow_swaps(&mut out.swaps, &events_cow);
    extract_curvefi_swaps(&mut out.swaps, &events_curvefi, &store_curvefi);
    extract_aerodrome_swaps(&mut out, &events_aerodrome, &store_aerodrome);
    extract_dodo_swaps(&mut out.swaps, &events_dodo);
    extract_woofi_swaps(&mut out.swaps, &events_woofi);
    extract_traderjoe_swaps(&mut out, &events_traderjoe, &store_traderjoe);
    extract_kyber_elastic_swaps(&mut out.swaps, &events_kyber_elastic, &store_kyber_elastic);
    extract_uniswap_v1_swaps(&mut out.swaps, &events_uniswap_v1, &store_uniswap_v1);
    extract_uniswap_v2_swaps(&mut out, &events_uniswap_v2, &store_uniswap_v2);
    extract_uniswap_v3_swaps(&mut out.swaps, &events_uniswap_v3, &store_uniswap_v3);
    extract_uniswap_v4_swaps(&mut out.swaps, &events_uniswap_v4, &store_uniswap_v4);

    Ok(out)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn get_pool<T: prost::Message + Default>(store: &StoreGetProto<T>, address: &[u8]) -> Option<T> {
    store.get_first(Hex::encode(address))
}

/// Returns `true` when a string represents a strictly positive uint/int256.
/// Signed negative values start with '-'; empty or zero strings are false.
fn is_positive(s: &str) -> bool {
    !s.is_empty() && !s.starts_with('-') && s != "0"
}

/// Returns `true` when a string represents a strictly negative int256.
fn is_negative(s: &str) -> bool {
    s.starts_with('-')
}

/// Strips a leading '-' and returns the absolute value string.
fn abs_str(s: &str) -> &str {
    s.strip_prefix('-').unwrap_or(s)
}

/// Builds the shared transaction/log context from protocol event types.
/// All DEX protos share an identical `Transaction` / `Log` layout so we
/// accept them via references to their concrete types through a trait.
fn fill_tx_context(swap: &mut dex::DexSwap, tx_hash: &[u8], tx_from: &[u8], tx_to: &Option<Vec<u8>>, tx_nonce: u64, tx_gas_price: &str, tx_gas_limit: u64, tx_gas_used: u64, tx_value: &str, tx_index: usize) {
    swap.tx_index = tx_index as u32;
    swap.tx_hash = tx_hash.to_vec();
    swap.tx_from = tx_from.to_vec();
    swap.tx_to = tx_to.clone();
    swap.tx_nonce = tx_nonce;
    swap.tx_gas_price = tx_gas_price.to_string();
    swap.tx_gas_limit = tx_gas_limit;
    swap.tx_gas_used = tx_gas_used;
    swap.tx_value = tx_value.to_string();
}

fn fill_tx_context_flash(swap: &mut dex::DexSwapFlash, tx_hash: &[u8], tx_from: &[u8], tx_to: &Option<Vec<u8>>, tx_nonce: u64, tx_gas_price: &str, tx_gas_limit: u64, tx_gas_used: u64, tx_value: &str, tx_index: usize) {
    swap.tx_index = tx_index as u32;
    swap.tx_hash = tx_hash.to_vec();
    swap.tx_from = tx_from.to_vec();
    swap.tx_to = tx_to.clone();
    swap.tx_nonce = tx_nonce;
    swap.tx_gas_price = tx_gas_price.to_string();
    swap.tx_gas_limit = tx_gas_limit;
    swap.tx_gas_used = tx_gas_used;
    swap.tx_value = tx_value.to_string();
}

fn fill_log_context(swap: &mut dex::DexSwap, log_index: usize, log_block_index: u32, log_address: &[u8], log_ordinal: u64, log_topics: &[Vec<u8>], log_data: &[u8]) {
    swap.log_index = log_index as u32;
    swap.log_block_index = log_block_index;
    swap.log_address = log_address.to_vec();
    swap.log_ordinal = log_ordinal;
    swap.log_topics = log_topics.to_vec();
    swap.log_data = log_data.to_vec();
}

fn fill_log_context_flash(swap: &mut dex::DexSwapFlash, log_index: usize, log_block_index: u32, log_address: &[u8], log_ordinal: u64, log_topics: &[Vec<u8>], log_data: &[u8]) {
    swap.log_index = log_index as u32;
    swap.log_block_index = log_block_index;
    swap.log_address = log_address.to_vec();
    swap.log_ordinal = log_ordinal;
    swap.log_topics = log_topics.to_vec();
    swap.log_data = log_data.to_vec();
}

macro_rules! fill_call {
    ($swap:expr, $log:expr) => {
        $swap.call = $log.call.as_ref().map(|c| dex::Call {
            index: c.index,
            begin_ordinal: c.begin_ordinal,
            end_ordinal: c.end_ordinal,
            caller: c.caller.clone(),
            address: c.address.clone(),
            value: c.value.clone(),
            gas_consumed: c.gas_consumed,
            gas_limit: c.gas_limit,
            depth: c.depth,
            parent_index: c.parent_index,
            call_type: c.call_type,
        });
    };
}

// ── SunPump ──────────────────────────────────────────────────────────────────

fn extract_sunpump_swaps(out: &mut dex::DexSwaps, events: &sunpump::v1::Events, store: &StoreGetProto<sunpump::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(sunpump::v1::log::Log::TokenPurchased(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        if is_positive(&event.trx_amount) && is_positive(&event.token_amount) {
                            let mut swap = dex::DexSwap::default();
                            fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                            fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                            fill_call!(swap, log);
                            swap.protocol = "sunpump".to_string();
                            swap.factory = pool.factory.clone();
                            swap.pool = log.address.clone();
                            swap.user = event.buyer.clone();
                            swap.input_contract = NULL_ADDRESS.to_vec();
                            swap.input_amount = event.trx_amount.clone();
                            swap.output_contract = event.token.clone();
                            swap.output_amount = event.token_amount.clone();
                            out.swaps.push(swap);
                        }
                    }
                }
                Some(sunpump::v1::log::Log::TokenSold(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        if is_positive(&event.token_amount) && is_positive(&event.trx_amount) {
                            let mut swap = dex::DexSwap::default();
                            fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                            fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                            fill_call!(swap, log);
                            swap.protocol = "sunpump".to_string();
                            swap.factory = pool.factory.clone();
                            swap.pool = log.address.clone();
                            swap.user = event.seller.clone();
                            swap.input_contract = event.token.clone();
                            swap.input_amount = event.token_amount.clone();
                            swap.output_contract = NULL_ADDRESS.to_vec();
                            swap.output_amount = event.trx_amount.clone();
                            out.swaps.push(swap);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// ── Balancer ─────────────────────────────────────────────────────────────────

fn extract_balancer_swaps(out: &mut Vec<dex::DexSwap>, events: &balancer::v1::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(balancer::v1::log::Log::VaultSwap(event)) = &log.log {
                if is_positive(&event.amount_in) && is_positive(&event.amount_out) {
                    let mut swap = dex::DexSwap::default();
                    fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                    fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                    fill_call!(swap, log);
                    swap.protocol = "balancer".to_string();
                    swap.factory = log.address.clone(); // vault address as factory
                    swap.pool = event.pool.clone();
                    swap.user = tx.from.clone();
                    swap.input_contract = event.token_in.clone();
                    swap.input_amount = event.amount_in.clone();
                    swap.output_contract = event.token_out.clone();
                    swap.output_amount = event.amount_out.clone();
                    out.push(swap);
                }
            }
        }
    }
}

// ── Bancor ───────────────────────────────────────────────────────────────────

fn extract_bancor_swaps(out: &mut Vec<dex::DexSwap>, events: &bancor::v1::Events, store: &StoreGetProto<bancor::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(bancor::v1::log::Log::Conversion(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    if is_positive(&event.source_amount) && is_positive(&event.target_amount) {
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "bancor".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.trader.clone();
                        swap.input_contract = event.source_token.clone();
                        swap.input_amount = event.source_amount.clone();
                        swap.output_contract = event.target_token.clone();
                        swap.output_amount = event.target_amount.clone();
                        out.push(swap);
                    }
                }
            }
        }
    }
}

// ── CoW Protocol ─────────────────────────────────────────────────────────────

fn extract_cow_swaps(out: &mut Vec<dex::DexSwap>, events: &cow::v1::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(cow::v1::log::Log::Trade(event)) = &log.log {
                if is_positive(&event.sell_amount) && is_positive(&event.buy_amount) {
                    let mut swap = dex::DexSwap::default();
                    fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                    fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                    fill_call!(swap, log);
                    swap.protocol = "cow".to_string();
                    swap.factory = log.address.clone(); // CoW uses settlement contract as factory
                    swap.pool = log.address.clone();
                    swap.user = event.owner.clone();
                    swap.input_contract = event.sell_token.clone();
                    swap.input_amount = event.sell_amount.clone();
                    swap.output_contract = event.buy_token.clone();
                    swap.output_amount = event.buy_amount.clone();
                    out.push(swap);
                }
            }
        }
    }
}

// ── Curve.fi ─────────────────────────────────────────────────────────────────

fn extract_curvefi_swaps(out: &mut Vec<dex::DexSwap>, events: &curvefi::v1::Events, store: &StoreGetProto<curvefi::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(curvefi::v1::log::Log::TokenExchange(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        let sold_id: usize = match event.sold_id.parse() { Ok(v) => v, Err(_) => continue };
                        let bought_id: usize = match event.bought_id.parse() { Ok(v) => v, Err(_) => continue };
                        let input_coin = pool.coins.get(sold_id).cloned().unwrap_or_default();
                        let output_coin = pool.coins.get(bought_id).cloned().unwrap_or_default();
                        if !input_coin.is_empty() && !output_coin.is_empty() && is_positive(&event.tokens_sold) && is_positive(&event.tokens_bought) {
                            let mut swap = dex::DexSwap::default();
                            fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                            fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                            fill_call!(swap, log);
                            swap.protocol = "curvefi".to_string();
                            swap.factory = pool.factory.clone();
                            swap.pool = log.address.clone();
                            swap.user = event.buyer.clone();
                            swap.input_contract = input_coin;
                            swap.input_amount = event.tokens_sold.clone();
                            swap.output_contract = output_coin;
                            swap.output_amount = event.tokens_bought.clone();
                            out.push(swap);
                        }
                    }
                }
                Some(curvefi::v1::log::Log::CryptoswapTokenExchange(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        let sold_id: usize = match event.sold_id.parse() { Ok(v) => v, Err(_) => continue };
                        let bought_id: usize = match event.bought_id.parse() { Ok(v) => v, Err(_) => continue };
                        let input_coin = pool.coins.get(sold_id).cloned().unwrap_or_default();
                        let output_coin = pool.coins.get(bought_id).cloned().unwrap_or_default();
                        if !input_coin.is_empty() && !output_coin.is_empty() && is_positive(&event.tokens_sold) && is_positive(&event.tokens_bought) {
                            let mut swap = dex::DexSwap::default();
                            fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                            fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                            fill_call!(swap, log);
                            swap.protocol = "curvefi".to_string();
                            swap.factory = pool.factory.clone();
                            swap.pool = log.address.clone();
                            swap.user = event.buyer.clone();
                            swap.input_contract = input_coin;
                            swap.input_amount = event.tokens_sold.clone();
                            swap.output_contract = output_coin;
                            swap.output_amount = event.tokens_bought.clone();
                            out.push(swap);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// ── Aerodrome / Velodrome (Solidly fork) ─────────────────────────────────────

fn extract_aerodrome_swaps(out: &mut dex::DexSwaps, events: &aerodrome::v1::Events, store: &StoreGetProto<aerodrome::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(aerodrome::v1::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    let a0_in = is_positive(&event.amount0_in);
                    let a1_in = is_positive(&event.amount1_in);
                    let a0_out = is_positive(&event.amount0_out);
                    let a1_out = is_positive(&event.amount1_out);

                    if a0_in && !a1_in && !a0_out && a1_out {
                        // token0 → token1
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "aerodrome".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency0.clone();
                        swap.input_amount = event.amount0_in.clone();
                        swap.output_contract = pool.currency1.clone();
                        swap.output_amount = event.amount1_out.clone();
                        out.swaps.push(swap);
                    } else if a1_in && !a0_in && a0_out && !a1_out {
                        // token1 → token0
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "aerodrome".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency1.clone();
                        swap.input_amount = event.amount1_in.clone();
                        swap.output_contract = pool.currency0.clone();
                        swap.output_amount = event.amount0_out.clone();
                        out.swaps.push(swap);
                    } else if (a0_in || a1_in) && (a0_out || a1_out) {
                        // Flash swap
                        let mut flash = dex::DexSwapFlash::default();
                        fill_tx_context_flash(&mut flash, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context_flash(&mut flash, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        flash.call = log.call.as_ref().map(|c| dex::Call {
                            index: c.index,
                            begin_ordinal: c.begin_ordinal,
                            end_ordinal: c.end_ordinal,
                            caller: c.caller.clone(),
                            address: c.address.clone(),
                            value: c.value.clone(),
                            gas_consumed: c.gas_consumed,
                            gas_limit: c.gas_limit,
                            depth: c.depth,
                            parent_index: c.parent_index,
                            call_type: c.call_type,
                        });
                        flash.protocol = "aerodrome".to_string();
                        flash.factory = pool.factory.clone();
                        flash.pool = log.address.clone();
                        flash.user = event.sender.clone();
                        flash.token0 = pool.currency0.clone();
                        flash.token1 = pool.currency1.clone();
                        flash.amount0_in = event.amount0_in.clone();
                        flash.amount1_in = event.amount1_in.clone();
                        flash.amount0_out = event.amount0_out.clone();
                        flash.amount1_out = event.amount1_out.clone();
                        out.swaps_flash.push(flash);
                    }
                }
            }
        }
    }
}

// ── DODO ─────────────────────────────────────────────────────────────────────

fn extract_dodo_swaps(out: &mut Vec<dex::DexSwap>, events: &dodo::v1::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(dodo::v1::log::Log::OrderHistory(event)) = &log.log {
                if is_positive(&event.from_amount) && is_positive(&event.return_amount) {
                    let mut swap = dex::DexSwap::default();
                    fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                    fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                    fill_call!(swap, log);
                    swap.protocol = "dodo".to_string();
                    swap.factory = log.address.clone();
                    swap.pool = log.address.clone();
                    swap.user = event.sender.clone();
                    swap.input_contract = event.from_token.clone();
                    swap.input_amount = event.from_amount.clone();
                    swap.output_contract = event.to_token.clone();
                    swap.output_amount = event.return_amount.clone();
                    out.push(swap);
                }
            }
        }
    }
}

// ── WooFi ────────────────────────────────────────────────────────────────────

fn extract_woofi_swaps(out: &mut Vec<dex::DexSwap>, events: &woofi::v1::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(woofi::v1::log::Log::WooSwap(event)) = &log.log {
                if is_positive(&event.from_amount) && is_positive(&event.to_amount) {
                    let mut swap = dex::DexSwap::default();
                    fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                    fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                    fill_call!(swap, log);
                    swap.protocol = "woofi".to_string();
                    swap.factory = log.address.clone();
                    swap.pool = log.address.clone();
                    swap.user = event.from.clone();
                    swap.input_contract = event.from_token.clone();
                    swap.input_amount = event.from_amount.clone();
                    swap.output_contract = event.to_token.clone();
                    swap.output_amount = event.to_amount.clone();
                    out.push(swap);
                }
            }
        }
    }
}

// ── TraderJoe ────────────────────────────────────────────────────────────────

fn extract_traderjoe_swaps(out: &mut dex::DexSwaps, events: &traderjoe::v1::Events, store: &StoreGetProto<traderjoe::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(traderjoe::v1::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    let ax_in = is_positive(&event.amount_in_x);
                    let ay_in = is_positive(&event.amount_in_y);

                    if ax_in && !ay_in {
                        // tokenX → tokenY
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "traderjoe".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency0.clone();
                        swap.input_amount = event.amount_in_x.clone();
                        swap.output_contract = pool.currency1.clone();
                        swap.output_amount = event.amount_out_y.clone();
                        out.swaps.push(swap);
                    } else if ay_in && !ax_in {
                        // tokenY → tokenX
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "traderjoe".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency1.clone();
                        swap.input_amount = event.amount_in_y.clone();
                        swap.output_contract = pool.currency0.clone();
                        swap.output_amount = event.amount_out_x.clone();
                        out.swaps.push(swap);
                    }
                    // if both > 0: unusual case, skip
                }
            }
        }
    }
}

// ── KyberSwap Elastic ────────────────────────────────────────────────────────

fn extract_kyber_elastic_swaps(out: &mut Vec<dex::DexSwap>, events: &kyber_elastic::v1::Events, store: &StoreGetProto<kyber_elastic::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(kyber_elastic::v1::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    // delta_qty0 < 0 means token0 flows OUT (to pool) → user swaps token0 IN
                    if is_negative(&event.delta_qty0) && is_positive(&event.delta_qty1) {
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "kyber_elastic".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency0.clone();
                        swap.input_amount = abs_str(&event.delta_qty0).to_string();
                        swap.output_contract = pool.currency1.clone();
                        swap.output_amount = abs_str(&event.delta_qty1).to_string();
                        out.push(swap);
                    } else if is_positive(&event.delta_qty0) && is_negative(&event.delta_qty1) {
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "kyber_elastic".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency1.clone();
                        swap.input_amount = abs_str(&event.delta_qty1).to_string();
                        swap.output_contract = pool.currency0.clone();
                        swap.output_amount = abs_str(&event.delta_qty0).to_string();
                        out.push(swap);
                    }
                }
            }
        }
    }
}

// ── Uniswap V1 ───────────────────────────────────────────────────────────────

fn extract_uniswap_v1_swaps(out: &mut Vec<dex::DexSwap>, events: &uniswap::v1::Events, store: &StoreGetProto<uniswap::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(uniswap::v1::log::Log::TokenPurchase(event)) => {
                    // ETH → Token
                    if let Some(pool) = get_pool(store, &log.address) {
                        if is_positive(&event.eth_sold) && is_positive(&event.tokens_bought) {
                            let mut swap = dex::DexSwap::default();
                            fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                            fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                            fill_call!(swap, log);
                            swap.protocol = "uniswap_v1".to_string();
                            swap.factory = pool.factory.clone();
                            swap.pool = log.address.clone();
                            swap.user = event.buyer.clone();
                            swap.input_contract = NULL_ADDRESS.to_vec(); // ETH
                            swap.input_amount = event.eth_sold.clone();
                            swap.output_contract = pool.currency0.clone(); // the ERC20 token
                            swap.output_amount = event.tokens_bought.clone();
                            out.push(swap);
                        }
                    }
                }
                Some(uniswap::v1::log::Log::EthPurchase(event)) => {
                    // Token → ETH
                    if let Some(pool) = get_pool(store, &log.address) {
                        if is_positive(&event.tokens_sold) && is_positive(&event.eth_bought) {
                            let mut swap = dex::DexSwap::default();
                            fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                            fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                            fill_call!(swap, log);
                            swap.protocol = "uniswap_v1".to_string();
                            swap.factory = pool.factory.clone();
                            swap.pool = log.address.clone();
                            swap.user = event.buyer.clone();
                            swap.input_contract = pool.currency0.clone(); // the ERC20 token
                            swap.input_amount = event.tokens_sold.clone();
                            swap.output_contract = NULL_ADDRESS.to_vec(); // ETH
                            swap.output_amount = event.eth_bought.clone();
                            out.push(swap);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// ── Uniswap V2 ───────────────────────────────────────────────────────────────

fn extract_uniswap_v2_swaps(out: &mut dex::DexSwaps, events: &uniswap::v2::Events, store: &StoreGetProto<uniswap::v2::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(uniswap::v2::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    let a0_in = is_positive(&event.amount0_in);
                    let a1_in = is_positive(&event.amount1_in);
                    let a0_out = is_positive(&event.amount0_out);
                    let a1_out = is_positive(&event.amount1_out);

                    if a0_in && !a1_in && !a0_out && a1_out {
                        // token0 → token1
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "uniswap_v2".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency0.clone();
                        swap.input_amount = event.amount0_in.clone();
                        swap.output_contract = pool.currency1.clone();
                        swap.output_amount = event.amount1_out.clone();
                        out.swaps.push(swap);
                    } else if a1_in && !a0_in && a0_out && !a1_out {
                        // token1 → token0
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "uniswap_v2".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency1.clone();
                        swap.input_amount = event.amount1_in.clone();
                        swap.output_contract = pool.currency0.clone();
                        swap.output_amount = event.amount0_out.clone();
                        out.swaps.push(swap);
                    } else if (a0_in || a1_in) && (a0_out || a1_out) {
                        // Flash swap - both directions active
                        let mut flash = dex::DexSwapFlash::default();
                        fill_tx_context_flash(&mut flash, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context_flash(&mut flash, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        flash.call = log.call.as_ref().map(|c| dex::Call {
                            index: c.index,
                            begin_ordinal: c.begin_ordinal,
                            end_ordinal: c.end_ordinal,
                            caller: c.caller.clone(),
                            address: c.address.clone(),
                            value: c.value.clone(),
                            gas_consumed: c.gas_consumed,
                            gas_limit: c.gas_limit,
                            depth: c.depth,
                            parent_index: c.parent_index,
                            call_type: c.call_type,
                        });
                        flash.protocol = "uniswap_v2".to_string();
                        flash.factory = pool.factory.clone();
                        flash.pool = log.address.clone();
                        flash.user = event.sender.clone();
                        flash.token0 = pool.currency0.clone();
                        flash.token1 = pool.currency1.clone();
                        flash.amount0_in = event.amount0_in.clone();
                        flash.amount1_in = event.amount1_in.clone();
                        flash.amount0_out = event.amount0_out.clone();
                        flash.amount1_out = event.amount1_out.clone();
                        out.swaps_flash.push(flash);
                    }
                }
            }
        }
    }
}

// ── Uniswap V3 ───────────────────────────────────────────────────────────────

fn extract_uniswap_v3_swaps(out: &mut Vec<dex::DexSwap>, events: &uniswap::v3::Events, store: &StoreGetProto<uniswap::v3::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(uniswap::v3::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    // In V3: negative amount = tokens leaving the pool (user receives),
                    // positive amount = tokens entering the pool (user sends)
                    if is_positive(&event.amount0) && is_negative(&event.amount1) {
                        // user swaps token0 in, receives token1 out
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "uniswap_v3".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency0.clone();
                        swap.input_amount = abs_str(&event.amount0).to_string();
                        swap.output_contract = pool.currency1.clone();
                        swap.output_amount = abs_str(&event.amount1).to_string();
                        out.push(swap);
                    } else if is_negative(&event.amount0) && is_positive(&event.amount1) {
                        // user swaps token1 in, receives token0 out
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "uniswap_v3".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = log.address.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency1.clone();
                        swap.input_amount = abs_str(&event.amount1).to_string();
                        swap.output_contract = pool.currency0.clone();
                        swap.output_amount = abs_str(&event.amount0).to_string();
                        out.push(swap);
                    }
                }
            }
        }
    }
}

// ── Uniswap V4 ───────────────────────────────────────────────────────────────

fn extract_uniswap_v4_swaps(out: &mut Vec<dex::DexSwap>, events: &uniswap::v4::Events, store: &StoreGetProto<uniswap::v4::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(uniswap::v4::log::Log::Swap(event)) = &log.log {
                // V4 pool id is stored in event.id; use it to look up pool metadata
                if let Some(pool) = get_pool(store, &event.id) {
                    if is_positive(&event.amount0) && is_negative(&event.amount1) {
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "uniswap_v4".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = event.id.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency0.clone();
                        swap.input_amount = abs_str(&event.amount0).to_string();
                        swap.output_contract = pool.currency1.clone();
                        swap.output_amount = abs_str(&event.amount1).to_string();
                        out.push(swap);
                    } else if is_negative(&event.amount0) && is_positive(&event.amount1) {
                        let mut swap = dex::DexSwap::default();
                        fill_tx_context(&mut swap, &tx.hash, &tx.from, &tx.to, tx.nonce, &tx.gas_price, tx.gas_limit, tx.gas_used, &tx.value, tx_index);
                        fill_log_context(&mut swap, log_index, log.block_index, &log.address, log.ordinal, &log.topics, &log.data);
                        fill_call!(swap, log);
                        swap.protocol = "uniswap_v4".to_string();
                        swap.factory = pool.factory.clone();
                        swap.pool = event.id.clone();
                        swap.user = event.sender.clone();
                        swap.input_contract = pool.currency1.clone();
                        swap.input_amount = abs_str(&event.amount1).to_string();
                        swap.output_contract = pool.currency0.clone();
                        swap.output_amount = abs_str(&event.amount0).to_string();
                        out.push(swap);
                    }
                }
            }
        }
    }
}
