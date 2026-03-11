/// Swap extraction helpers used by the `map_dex_swaps` handler.
/// Extracts swap events from each DEX protocol and normalizes them into the
/// unified `DexSwap` / `DexSwapFlash` format.
use proto::pb::{aerodrome, balancer, bancor, cow, curvefi, dodo, kyber_elastic, sunpump, traderjoe, uniswap, woofi};
use proto::pb::dex::v1 as dex;
use substreams::Hex;
use substreams::prelude::*;
use substreams_ethereum::NULL_ADDRESS;

fn get_pool<T: prost::Message + Default>(store: &StoreGetProto<T>, address: &[u8]) -> Option<T> {
    store.get_first(Hex::encode(address))
}

fn is_positive(s: &str) -> bool {
    !s.is_empty() && !s.starts_with('-') && s != "0"
}

fn is_negative(s: &str) -> bool {
    s.starts_with('-')
}

fn abs_str(s: &str) -> &str {
    s.strip_prefix('-').unwrap_or(s)
}

macro_rules! make_swap {
    () => {
        dex::DexSwap::default()
    };
}

macro_rules! fill_tx {
    ($swap:expr, $tx:expr, $tx_index:expr) => {
        $swap.tx_index = $tx_index as u32;
        $swap.tx_hash = $tx.hash.clone();
        $swap.tx_from = $tx.from.clone();
        $swap.tx_to = $tx.to.clone();
        $swap.tx_nonce = $tx.nonce;
        $swap.tx_gas_price = $tx.gas_price.clone();
        $swap.tx_gas_limit = $tx.gas_limit;
        $swap.tx_gas_used = $tx.gas_used;
        $swap.tx_value = $tx.value.clone();
    };
}

macro_rules! fill_log {
    ($swap:expr, $log:expr, $log_index:expr) => {
        $swap.log_index = $log_index as u32;
        $swap.log_block_index = $log.block_index;
        $swap.log_address = $log.address.clone();
        $swap.log_ordinal = $log.ordinal;
        $swap.log_topics = $log.topics.clone();
        $swap.log_data = $log.data.clone();
    };
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

pub fn extract_sunpump_swaps(out: &mut dex::DexSwaps, events: &sunpump::v1::Events, store: &StoreGetProto<sunpump::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(sunpump::v1::log::Log::TokenPurchased(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        if is_positive(&event.trx_amount) && is_positive(&event.token_amount) {
                            let mut swap = make_swap!();
                            fill_tx!(swap, tx, tx_index);
                            fill_log!(swap, log, log_index);
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
                            let mut swap = make_swap!();
                            fill_tx!(swap, tx, tx_index);
                            fill_log!(swap, log, log_index);
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

pub fn extract_balancer_swaps(out: &mut Vec<dex::DexSwap>, events: &balancer::v1::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(balancer::v1::log::Log::VaultSwap(event)) = &log.log {
                if is_positive(&event.amount_in) && is_positive(&event.amount_out) {
                    let mut swap = make_swap!();
                    fill_tx!(swap, tx, tx_index);
                    fill_log!(swap, log, log_index);
                    fill_call!(swap, log);
                    swap.protocol = "balancer".to_string();
                    swap.factory = log.address.clone();
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

pub fn extract_bancor_swaps(out: &mut Vec<dex::DexSwap>, events: &bancor::v1::Events, store: &StoreGetProto<bancor::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(bancor::v1::log::Log::Conversion(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    if is_positive(&event.source_amount) && is_positive(&event.target_amount) {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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

pub fn extract_cow_swaps(out: &mut Vec<dex::DexSwap>, events: &cow::v1::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(cow::v1::log::Log::Trade(event)) = &log.log {
                if is_positive(&event.sell_amount) && is_positive(&event.buy_amount) {
                    let mut swap = make_swap!();
                    fill_tx!(swap, tx, tx_index);
                    fill_log!(swap, log, log_index);
                    fill_call!(swap, log);
                    swap.protocol = "cow".to_string();
                    swap.factory = log.address.clone();
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

pub fn extract_curvefi_swaps(out: &mut Vec<dex::DexSwap>, events: &curvefi::v1::Events, store: &StoreGetProto<curvefi::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(curvefi::v1::log::Log::TokenExchange(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        try_push_curvefi_swap(out, tx, log, tx_index, log_index, &pool, &event.buyer, &event.sold_id, &event.tokens_sold, &event.bought_id, &event.tokens_bought);
                    }
                }
                Some(curvefi::v1::log::Log::CryptoswapTokenExchange(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        try_push_curvefi_swap(out, tx, log, tx_index, log_index, &pool, &event.buyer, &event.sold_id, &event.tokens_sold, &event.bought_id, &event.tokens_bought);
                    }
                }
                _ => {}
            }
        }
    }
}

fn try_push_curvefi_swap(out: &mut Vec<dex::DexSwap>, tx: &curvefi::v1::Transaction, log: &curvefi::v1::Log, tx_index: usize, log_index: usize, pool: &curvefi::v1::StorePool, buyer: &[u8], sold_id: &str, tokens_sold: &str, bought_id: &str, tokens_bought: &str) {
    let sold_idx: usize = match sold_id.parse() { Ok(v) => v, Err(_) => return };
    let bought_idx: usize = match bought_id.parse() { Ok(v) => v, Err(_) => return };
    let input_coin = pool.coins.get(sold_idx).cloned().unwrap_or_default();
    let output_coin = pool.coins.get(bought_idx).cloned().unwrap_or_default();
    if !input_coin.is_empty() && !output_coin.is_empty() && is_positive(tokens_sold) && is_positive(tokens_bought) {
        let mut swap = make_swap!();
        fill_tx!(swap, tx, tx_index);
        fill_log!(swap, log, log_index);
        fill_call!(swap, log);
        swap.protocol = "curvefi".to_string();
        swap.factory = pool.factory.clone();
        swap.pool = log.address.clone();
        swap.user = buyer.to_vec();
        swap.input_contract = input_coin;
        swap.input_amount = tokens_sold.to_string();
        swap.output_contract = output_coin;
        swap.output_amount = tokens_bought.to_string();
        out.push(swap);
    }
}

// ── Aerodrome / Velodrome ─────────────────────────────────────────────────────

pub fn extract_aerodrome_swaps(out: &mut dex::DexSwaps, events: &aerodrome::v1::Events, store: &StoreGetProto<aerodrome::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(aerodrome::v1::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    let a0i = is_positive(&event.amount0_in);
                    let a1i = is_positive(&event.amount1_in);
                    let a0o = is_positive(&event.amount0_out);
                    let a1o = is_positive(&event.amount1_out);

                    if a0i && !a1i && !a0o && a1o {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                    } else if a1i && !a0i && a0o && !a1o {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                    } else if (a0i || a1i) && (a0o || a1o) {
                        push_v2_flash(&mut out.swaps_flash, tx, log, tx_index, log_index, "aerodrome", &pool.factory, &log.address, &event.sender, &pool.currency0, &pool.currency1, &event.amount0_in, &event.amount1_in, &event.amount0_out, &event.amount1_out);
                    }
                }
            }
        }
    }
}

fn push_v2_flash(out: &mut Vec<dex::DexSwapFlash>, tx: &aerodrome::v1::Transaction, log: &aerodrome::v1::Log, tx_index: usize, log_index: usize, protocol: &str, factory: &[u8], pool: &[u8], user: &[u8], token0: &[u8], token1: &[u8], amount0_in: &str, amount1_in: &str, amount0_out: &str, amount1_out: &str) {
    let mut flash = dex::DexSwapFlash::default();
    flash.tx_index = tx_index as u32;
    flash.tx_hash = tx.hash.clone();
    flash.tx_from = tx.from.clone();
    flash.tx_to = tx.to.clone();
    flash.tx_nonce = tx.nonce;
    flash.tx_gas_price = tx.gas_price.clone();
    flash.tx_gas_limit = tx.gas_limit;
    flash.tx_gas_used = tx.gas_used;
    flash.tx_value = tx.value.clone();
    flash.log_index = log_index as u32;
    flash.log_block_index = log.block_index;
    flash.log_address = log.address.clone();
    flash.log_ordinal = log.ordinal;
    flash.log_topics = log.topics.clone();
    flash.log_data = log.data.clone();
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
    flash.protocol = protocol.to_string();
    flash.factory = factory.to_vec();
    flash.pool = pool.to_vec();
    flash.user = user.to_vec();
    flash.token0 = token0.to_vec();
    flash.token1 = token1.to_vec();
    flash.amount0_in = amount0_in.to_string();
    flash.amount1_in = amount1_in.to_string();
    flash.amount0_out = amount0_out.to_string();
    flash.amount1_out = amount1_out.to_string();
    out.push(flash);
}

// ── DODO ─────────────────────────────────────────────────────────────────────

pub fn extract_dodo_swaps(out: &mut Vec<dex::DexSwap>, events: &dodo::v1::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(dodo::v1::log::Log::OrderHistory(event)) = &log.log {
                if is_positive(&event.from_amount) && is_positive(&event.return_amount) {
                    let mut swap = make_swap!();
                    fill_tx!(swap, tx, tx_index);
                    fill_log!(swap, log, log_index);
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

pub fn extract_woofi_swaps(out: &mut Vec<dex::DexSwap>, events: &woofi::v1::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(woofi::v1::log::Log::WooSwap(event)) = &log.log {
                if is_positive(&event.from_amount) && is_positive(&event.to_amount) {
                    let mut swap = make_swap!();
                    fill_tx!(swap, tx, tx_index);
                    fill_log!(swap, log, log_index);
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

pub fn extract_traderjoe_swaps(out: &mut dex::DexSwaps, events: &traderjoe::v1::Events, store: &StoreGetProto<traderjoe::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(traderjoe::v1::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    let axi = is_positive(&event.amount_in_x);
                    let ayi = is_positive(&event.amount_in_y);
                    if axi && !ayi {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                    } else if ayi && !axi {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                }
            }
        }
    }
}

// ── KyberSwap Elastic ────────────────────────────────────────────────────────

pub fn extract_kyber_elastic_swaps(out: &mut Vec<dex::DexSwap>, events: &kyber_elastic::v1::Events, store: &StoreGetProto<kyber_elastic::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(kyber_elastic::v1::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    if is_negative(&event.delta_qty0) && is_positive(&event.delta_qty1) {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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

pub fn extract_uniswap_v1_swaps(out: &mut Vec<dex::DexSwap>, events: &uniswap::v1::Events, store: &StoreGetProto<uniswap::v1::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(uniswap::v1::log::Log::TokenPurchase(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        if is_positive(&event.eth_sold) && is_positive(&event.tokens_bought) {
                            let mut swap = make_swap!();
                            fill_tx!(swap, tx, tx_index);
                            fill_log!(swap, log, log_index);
                            fill_call!(swap, log);
                            swap.protocol = "uniswap_v1".to_string();
                            swap.factory = pool.factory.clone();
                            swap.pool = log.address.clone();
                            swap.user = event.buyer.clone();
                            swap.input_contract = NULL_ADDRESS.to_vec();
                            swap.input_amount = event.eth_sold.clone();
                            swap.output_contract = pool.currency0.clone();
                            swap.output_amount = event.tokens_bought.clone();
                            out.push(swap);
                        }
                    }
                }
                Some(uniswap::v1::log::Log::EthPurchase(event)) => {
                    if let Some(pool) = get_pool(store, &log.address) {
                        if is_positive(&event.tokens_sold) && is_positive(&event.eth_bought) {
                            let mut swap = make_swap!();
                            fill_tx!(swap, tx, tx_index);
                            fill_log!(swap, log, log_index);
                            fill_call!(swap, log);
                            swap.protocol = "uniswap_v1".to_string();
                            swap.factory = pool.factory.clone();
                            swap.pool = log.address.clone();
                            swap.user = event.buyer.clone();
                            swap.input_contract = pool.currency0.clone();
                            swap.input_amount = event.tokens_sold.clone();
                            swap.output_contract = NULL_ADDRESS.to_vec();
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

pub fn extract_uniswap_v2_swaps(out: &mut dex::DexSwaps, events: &uniswap::v2::Events, store: &StoreGetProto<uniswap::v2::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(uniswap::v2::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    let a0i = is_positive(&event.amount0_in);
                    let a1i = is_positive(&event.amount1_in);
                    let a0o = is_positive(&event.amount0_out);
                    let a1o = is_positive(&event.amount1_out);

                    if a0i && !a1i && !a0o && a1o {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                    } else if a1i && !a0i && a0o && !a1o {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                    } else if (a0i || a1i) && (a0o || a1o) {
                        // Flash swap
                        let mut flash = dex::DexSwapFlash::default();
                        flash.tx_index = tx_index as u32;
                        flash.tx_hash = tx.hash.clone();
                        flash.tx_from = tx.from.clone();
                        flash.tx_to = tx.to.clone();
                        flash.tx_nonce = tx.nonce;
                        flash.tx_gas_price = tx.gas_price.clone();
                        flash.tx_gas_limit = tx.gas_limit;
                        flash.tx_gas_used = tx.gas_used;
                        flash.tx_value = tx.value.clone();
                        flash.log_index = log_index as u32;
                        flash.log_block_index = log.block_index;
                        flash.log_address = log.address.clone();
                        flash.log_ordinal = log.ordinal;
                        flash.log_topics = log.topics.clone();
                        flash.log_data = log.data.clone();
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

pub fn extract_uniswap_v3_swaps(out: &mut Vec<dex::DexSwap>, events: &uniswap::v3::Events, store: &StoreGetProto<uniswap::v3::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(uniswap::v3::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &log.address) {
                    if is_positive(&event.amount0) && is_negative(&event.amount1) {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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

pub fn extract_uniswap_v4_swaps(out: &mut Vec<dex::DexSwap>, events: &uniswap::v4::Events, store: &StoreGetProto<uniswap::v4::StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(uniswap::v4::log::Log::Swap(event)) = &log.log {
                if let Some(pool) = get_pool(store, &event.id) {
                    if is_positive(&event.amount0) && is_negative(&event.amount1) {
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
                        let mut swap = make_swap!();
                        fill_tx!(swap, tx, tx_index);
                        fill_log!(swap, log, log_index);
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
