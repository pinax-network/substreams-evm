use common::{bytes_to_string, Encoding};
use proto::pb::ohlc::v1::{Events, Ohlcv};
use proto::pb::uniswap;
use std::collections::HashMap;
use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams::prelude::*;

#[substreams::handlers::map]
pub fn ohlc_out(
    clock: Clock,
    // Tron DEX
    events_sunpump: proto::pb::sunpump::v1::Events,
    store_sunpump: StoreGetProto<proto::pb::sunpump::v1::StorePool>,
    // Ethereum DEX
    events_balancer: proto::pb::balancer::v1::Events,
    events_bancor: proto::pb::bancor::v1::Events,
    events_curvefi: proto::pb::curvefi::v1::Events,
    events_cow: proto::pb::cow::v1::Events,
    store_balancer: StoreGetProto<proto::pb::balancer::v1::StorePool>,
    store_bancor: StoreGetProto<proto::pb::bancor::v1::StorePool>,
    store_curvefi: StoreGetProto<proto::pb::curvefi::v1::StorePool>,
    // Uniswap DEX
    events_uniswap_v1: uniswap::v1::Events,
    events_uniswap_v2: uniswap::v2::Events,
    events_uniswap_v3: uniswap::v3::Events,
    events_uniswap_v4: uniswap::v4::Events,
    store_uniswap_v1: StoreGetProto<uniswap::v1::StorePool>,
    store_uniswap_v2: StoreGetProto<uniswap::v2::StorePool>,
    store_uniswap_v3: StoreGetProto<uniswap::v3::StorePool>,
    store_uniswap_v4: StoreGetProto<uniswap::v4::StorePool>,
) -> Result<Events, Error> {
    let encoding = Encoding::Hex;
    let mut ohlc_data: HashMap<PoolKey, OhlcAccumulator> = HashMap::new();

    // Process events from all DEX protocols
    process_sunpump_events(&encoding, &clock, &events_sunpump, &store_sunpump, &mut ohlc_data);
    process_balancer_events(&encoding, &clock, &events_balancer, &store_balancer, &mut ohlc_data);
    process_bancor_events(&encoding, &clock, &events_bancor, &store_bancor, &mut ohlc_data);
    process_curvefi_events(&encoding, &clock, &events_curvefi, &store_curvefi, &mut ohlc_data);
    process_cow_events(&encoding, &clock, &events_cow, &mut ohlc_data);
    process_uniswap_v1_events(&encoding, &clock, &events_uniswap_v1, &store_uniswap_v1, &mut ohlc_data);
    process_uniswap_v2_events(&encoding, &clock, &events_uniswap_v2, &store_uniswap_v2, &mut ohlc_data);
    process_uniswap_v3_events(&encoding, &clock, &events_uniswap_v3, &store_uniswap_v3, &mut ohlc_data);
    process_uniswap_v4_events(&encoding, &clock, &events_uniswap_v4, &store_uniswap_v4, &mut ohlc_data);

    // Convert accumulated data to OHLCV events
    let mut events = Events { ohlcv: vec![] };
    
    for (key, acc) in ohlc_data.iter() {
        if let Some(ohlcv) = acc.to_ohlcv(&encoding, &clock, key) {
            events.ohlcv.push(ohlcv);
        }
    }

    Ok(events)
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct PoolKey {
    protocol: String,
    factory: Vec<u8>,
    pool: Vec<u8>,
    token0: Vec<u8>,
    token1: Vec<u8>,
}

struct OhlcAccumulator {
    prices: Vec<f64>,
    first_price: Option<f64>,
    last_price: Option<f64>,
    gross_volume0: i128,
    gross_volume1: i128,
    net_flow0: i128,
    net_flow1: i128,
    transactions: u64,
    unique_users: std::collections::HashSet<Vec<u8>>,
    unique_tx_from: std::collections::HashSet<Vec<u8>>,
}

impl OhlcAccumulator {
    fn new() -> Self {
        Self {
            prices: vec![],
            first_price: None,
            last_price: None,
            gross_volume0: 0,
            gross_volume1: 0,
            net_flow0: 0,
            net_flow1: 0,
            transactions: 0,
            unique_users: std::collections::HashSet::new(),
            unique_tx_from: std::collections::HashSet::new(),
        }
    }

    fn add_swap(
        &mut self,
        input_token: &[u8],
        _output_token: &[u8],
        input_amount: i128,
        output_amount: i128,
        token0: &[u8],
        user: &[u8],
        tx_from: &[u8],
    ) {
        // Calculate price as output/input (using absolute values to handle signed amounts)
        let abs_input = input_amount.abs();
        let abs_output = output_amount.abs();
        
        let price = if abs_input > 0 {
            abs_output as f64 / abs_input as f64
        } else {
            0.0
        };

        // Add price to list for quantile calculation
        if price > 0.0 {
            self.prices.push(price);
            if self.first_price.is_none() {
                self.first_price = Some(price);
            }
            self.last_price = Some(price);
        }

        // Determine direction: token0 -> token1 or token1 -> token0
        let is_token0_to_token1 = input_token == token0;

        // Use absolute values for volume tracking
        if is_token0_to_token1 {
            // Selling token0 for token1
            self.gross_volume0 += abs_input;
            self.gross_volume1 += abs_output;
            self.net_flow0 -= abs_input; // token0 flows out
            self.net_flow1 += abs_output; // token1 flows in
        } else {
            // Selling token1 for token0
            self.gross_volume1 += abs_input;
            self.gross_volume0 += abs_output;
            self.net_flow1 -= abs_input; // token1 flows out
            self.net_flow0 += abs_output; // token0 flows in
        }

        self.transactions += 1;
        self.unique_users.insert(user.to_vec());
        self.unique_tx_from.insert(tx_from.to_vec());
    }

    fn to_ohlcv(&self, _encoding: &Encoding, clock: &Clock, key: &PoolKey) -> Option<Ohlcv> {
        if self.prices.is_empty() {
            return None;
        }

        let mut sorted_prices = self.prices.clone();
        sorted_prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let open = self.first_price.unwrap_or(0.0);
        let close = self.last_price.unwrap_or(0.0);
        
        // Calculate high as quantile 0.95 and low as quantile 0.05
        let high = quantile(&sorted_prices, 0.95);
        let low = quantile(&sorted_prices, 0.05);

        let timestamp = clock.timestamp.as_ref()?.seconds as u64;
        // Decode block hash from hex, log info if it fails
        let block_hash = match hex::decode(&clock.id) {
            Ok(hash) => hash,
            Err(_) => {
                substreams::log::info!("Failed to decode block hash: {}", clock.id);
                vec![]
            }
        };

        Some(Ohlcv {
            timestamp,
            block_num: clock.number,
            block_hash,
            protocol: key.protocol.clone(),
            factory: key.factory.clone(),
            pool: key.pool.clone(),
            token0: key.token0.clone(),
            token1: key.token1.clone(),
            open,
            high,
            low,
            close,
            gross_volume0: self.gross_volume0.to_string(),
            gross_volume1: self.gross_volume1.to_string(),
            net_flow0: self.net_flow0.to_string(),
            net_flow1: self.net_flow1.to_string(),
            transactions: self.transactions,
            unique_users: self.unique_users.len() as u64,
            unique_tx_from: self.unique_tx_from.len() as u64,
        })
    }
}

fn quantile(sorted_data: &[f64], q: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }
    
    let pos = (sorted_data.len() as f64 - 1.0) * q;
    let base = pos.floor() as usize;
    let rest = pos - base as f64;

    if base + 1 < sorted_data.len() {
        sorted_data[base] + rest * (sorted_data[base + 1] - sorted_data[base])
    } else {
        sorted_data[base]
    }
}

fn get_or_create_accumulator<'a>(
    map: &'a mut HashMap<PoolKey, OhlcAccumulator>,
    protocol: &str,
    factory: &[u8],
    pool: &[u8],
    token0: &[u8],
    token1: &[u8],
) -> &'a mut OhlcAccumulator {
    let key = PoolKey {
        protocol: protocol.to_string(),
        factory: factory.to_vec(),
        pool: pool.to_vec(),
        token0: token0.to_vec(),
        token1: token1.to_vec(),
    };
    
    map.entry(key).or_insert_with(OhlcAccumulator::new)
}

fn normalize_token_pair(token_a: &[u8], token_b: &[u8]) -> (Vec<u8>, Vec<u8>) {
    if token_a <= token_b {
        (token_a.to_vec(), token_b.to_vec())
    } else {
        (token_b.to_vec(), token_a.to_vec())
    }
}

// Process functions for each DEX protocol
fn process_sunpump_events(
    _encoding: &Encoding,
    _clock: &Clock,
    _events: &proto::pb::sunpump::v1::Events,
    _store: &StoreGetProto<proto::pb::sunpump::v1::StorePool>,
    _ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    // TODO: Implement SunPump event processing
}

fn process_balancer_events(
    _encoding: &Encoding,
    _clock: &Clock,
    _events: &proto::pb::balancer::v1::Events,
    _store: &StoreGetProto<proto::pb::balancer::v1::StorePool>,
    _ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    // TODO: Implement Balancer event processing
}

fn process_bancor_events(
    _encoding: &Encoding,
    _clock: &Clock,
    _events: &proto::pb::bancor::v1::Events,
    _store: &StoreGetProto<proto::pb::bancor::v1::StorePool>,
    _ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    // TODO: Implement Bancor event processing
}

fn process_curvefi_events(
    _encoding: &Encoding,
    _clock: &Clock,
    _events: &proto::pb::curvefi::v1::Events,
    _store: &StoreGetProto<proto::pb::curvefi::v1::StorePool>,
    _ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    // TODO: Implement Curve.fi event processing
}

fn process_cow_events(
    _encoding: &Encoding,
    _clock: &Clock,
    _events: &proto::pb::cow::v1::Events,
    _ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    // TODO: Implement CoW Protocol event processing
}

fn process_uniswap_v1_events(
    encoding: &Encoding,
    _clock: &Clock,
    events: &uniswap::v1::Events,
    store: &StoreGetProto<uniswap::v1::StorePool>,
    ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    for tx in &events.transactions {
        for log in &tx.logs {
            match &log.log {
                Some(uniswap::v1::log::Log::TokenPurchase(purchase)) => {
                    // TokenPurchase: ETH -> Token
                    if let Some(pool_info) = store.get_last(&bytes_to_string(&log.address, encoding)) {
                        let eth_sold = purchase.eth_sold.parse::<i128>().unwrap_or(0);
                        let tokens_bought = purchase.tokens_bought.parse::<i128>().unwrap_or(0);

                        // ETH is typically stored as a special address or empty
                        let eth_address = vec![]; // Representing ETH as empty bytes
                        let token_address = &pool_info.currency0;

                        let (token0, token1) = normalize_token_pair(&eth_address, token_address);

                        let acc = get_or_create_accumulator(
                            ohlc_data,
                            "uniswap_v1",
                            &pool_info.factory,
                            &log.address,
                            &token0,
                            &token1,
                        );

                        acc.add_swap(
                            &eth_address,
                            token_address,
                            eth_sold,
                            tokens_bought,
                            &token0,
                            &purchase.buyer,
                            &tx.from,
                        );
                    }
                }
                Some(uniswap::v1::log::Log::EthPurchase(purchase)) => {
                    // EthPurchase: Token -> ETH
                    if let Some(pool_info) = store.get_last(&bytes_to_string(&log.address, encoding)) {
                        let tokens_sold = purchase.tokens_sold.parse::<i128>().unwrap_or(0);
                        let eth_bought = purchase.eth_bought.parse::<i128>().unwrap_or(0);

                        let eth_address = vec![]; // Representing ETH as empty bytes
                        let token_address = &pool_info.currency0;

                        let (token0, token1) = normalize_token_pair(&eth_address, token_address);

                        let acc = get_or_create_accumulator(
                            ohlc_data,
                            "uniswap_v1",
                            &pool_info.factory,
                            &log.address,
                            &token0,
                            &token1,
                        );

                        acc.add_swap(
                            token_address,
                            &eth_address,
                            tokens_sold,
                            eth_bought,
                            &token0,
                            &purchase.buyer,
                            &tx.from,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

fn process_uniswap_v2_events(
    encoding: &Encoding,
    _clock: &Clock,
    events: &uniswap::v2::Events,
    store: &StoreGetProto<uniswap::v2::StorePool>,
    ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    for tx in &events.transactions {
        for log in &tx.logs {
            if let Some(uniswap::v2::log::Log::Swap(swap)) = &log.log {
                if let Some(pool_info) = store.get_last(&bytes_to_string(&log.address, encoding)) {
                    // Parse amounts
                    let amount0_in = swap.amount0_in.parse::<i128>().unwrap_or(0);
                    let amount1_in = swap.amount1_in.parse::<i128>().unwrap_or(0);
                    let amount0_out = swap.amount0_out.parse::<i128>().unwrap_or(0);
                    let amount1_out = swap.amount1_out.parse::<i128>().unwrap_or(0);

                    // Determine swap direction and amounts
                    let (input_token, _output_token, input_amount, output_amount) = if amount0_in > 0 && amount1_out > 0 {
                        // token0 -> token1
                        (&pool_info.currency0, &pool_info.currency1, amount0_in, amount1_out)
                    } else if amount1_in > 0 && amount0_out > 0 {
                        // token1 -> token0
                        (&pool_info.currency1, &pool_info.currency0, amount1_in, amount0_out)
                    } else {
                        continue; // Skip invalid swaps
                    };

                    // Normalize token pair (alphabetical ordering)
                    let (token0, token1) = normalize_token_pair(&pool_info.currency0, &pool_info.currency1);

                    let acc = get_or_create_accumulator(
                        ohlc_data,
                        "uniswap_v2",
                        &pool_info.factory,
                        &log.address,
                        &token0,
                        &token1,
                    );

                    acc.add_swap(
                        input_token,
                        _output_token,
                        input_amount,
                        output_amount,
                        &token0,
                        &swap.sender,
                        &tx.from,
                    );
                }
            }
        }
    }
}

fn process_uniswap_v3_events(
    encoding: &Encoding,
    _clock: &Clock,
    events: &uniswap::v3::Events,
    store: &StoreGetProto<uniswap::v3::StorePool>,
    ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    for tx in &events.transactions {
        for log in &tx.logs {
            if let Some(uniswap::v3::log::Log::Swap(swap)) = &log.log {
                if let Some(pool_info) = store.get_last(&bytes_to_string(&log.address, encoding)) {
                    // Parse amounts (signed integers as strings)
                    let amount0 = swap.amount0.parse::<i128>().unwrap_or(0);
                    let amount1 = swap.amount1.parse::<i128>().unwrap_or(0);

                    // In V3, negative means input, positive means output
                    let (input_token, _output_token, input_amount, output_amount) = if amount0 < 0 {
                        // token0 -> token1
                        (&pool_info.currency0, &pool_info.currency1, amount0.abs(), amount1.abs())
                    } else {
                        // token1 -> token0
                        (&pool_info.currency1, &pool_info.currency0, amount1.abs(), amount0.abs())
                    };

                    // Normalize token pair (alphabetical ordering)
                    let (token0, token1) = normalize_token_pair(&pool_info.currency0, &pool_info.currency1);

                    let acc = get_or_create_accumulator(
                        ohlc_data,
                        "uniswap_v3",
                        &pool_info.factory,
                        &log.address,
                        &token0,
                        &token1,
                    );

                    acc.add_swap(
                        input_token,
                        _output_token,
                        input_amount,
                        output_amount,
                        &token0,
                        &swap.sender,
                        &tx.from,
                    );
                }
            }
        }
    }
}

fn process_uniswap_v4_events(
    encoding: &Encoding,
    _clock: &Clock,
    events: &uniswap::v4::Events,
    store: &StoreGetProto<uniswap::v4::StorePool>,
    ohlc_data: &mut HashMap<PoolKey, OhlcAccumulator>,
) {
    for tx in &events.transactions {
        for log in &tx.logs {
            if let Some(uniswap::v4::log::Log::Swap(swap)) = &log.log {
                // In V4, the pool ID is stored in the swap event
                if let Some(pool_info) = store.get_last(&bytes_to_string(&swap.id, encoding)) {
                    // Parse amounts (signed integers as strings)
                    let amount0 = swap.amount0.parse::<i128>().unwrap_or(0);
                    let amount1 = swap.amount1.parse::<i128>().unwrap_or(0);

                    // In V4, like V3, negative means input, positive means output
                    let (input_token, _output_token, input_amount, output_amount) = if amount0 < 0 {
                        // currency0 -> currency1
                        (&pool_info.currency0, &pool_info.currency1, amount0.abs(), amount1.abs())
                    } else {
                        // currency1 -> currency0
                        (&pool_info.currency1, &pool_info.currency0, amount1.abs(), amount0.abs())
                    };

                    // Normalize token pair (alphabetical ordering)
                    let (token0, token1) = normalize_token_pair(&pool_info.currency0, &pool_info.currency1);

                    let acc = get_or_create_accumulator(
                        ohlc_data,
                        "uniswap_v4",
                        &pool_info.factory,
                        &swap.id, // Use pool ID as the pool address
                        &token0,
                        &token1,
                    );

                    acc.add_swap(
                        input_token,
                        _output_token,
                        input_amount,
                        output_amount,
                        &token0,
                        &swap.sender,
                        &tx.from,
                    );
                }
            }
        }
    }
}
