-- Finalized read view over the AggregatingMergeTree state table
CREATE VIEW IF NOT EXISTS x402_state AS
SELECT
    -- timestamp & block number --
    min(min_timestamp) AS min_timestamp,
    max(max_timestamp) AS max_timestamp,
    min(min_block_num) AS min_block_num,
    max(max_block_num) AS max_block_num,

    -- x402 identity --
    facilitator,
    recipient,
    asset,
    transfer_method,
    settlement_source,
    scheme,

    -- aggregates --
    sum(payments) AS payments,
    sum(amount) AS amount,
    uniqMerge(uniq_payer) AS unique_payers,
    uniqMerge(uniq_tx_from) AS unique_tx_from,
    uniqMerge(uniq_tx_hash) AS unique_tx_hash
FROM state_x402
GROUP BY
    facilitator,
    recipient,
    asset,
    transfer_method,
    settlement_source,
    scheme;
