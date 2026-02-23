-- Seaport Order Fulfilled --
CREATE TABLE IF NOT EXISTS seaport_order_fulfilled AS TEMPLATE_LOG
COMMENT 'Seaport OrderFulfilled events';
ALTER TABLE seaport_order_fulfilled
    -- event --
    ADD COLUMN IF NOT EXISTS order_hash           String,
    ADD COLUMN IF NOT EXISTS offerer              String,
    ADD COLUMN IF NOT EXISTS zone                 String,
    ADD COLUMN IF NOT EXISTS recipient            String,

    -- event (JSON) --
    ADD COLUMN IF NOT EXISTS offer_raw            String,
    ADD COLUMN IF NOT EXISTS offer Array(Tuple(
        UInt8,             -- item_type
        FixedString(42),   -- token
        UInt256,           -- identifier
        UInt256            -- amount
    )) MATERIALIZED (
        arrayMap(
            x -> tuple(
                toUInt8(JSONExtract(x, 'item_type', 'UInt8')),
                JSONExtract(x, 'token', 'FixedString(42)'),
                toUInt256(JSONExtract(x, 'identifier', 'String')),
                toUInt256(JSONExtract(x, 'amount', 'String'))
            ),
            JSONExtractArrayRaw(offer_raw)
        )
    ),
    ADD COLUMN IF NOT EXISTS consideration_raw       String,
    ADD COLUMN IF NOT EXISTS consideration Array(Tuple(
        UInt8,             -- item_type
        FixedString(42),   -- token
        UInt256,           -- identifier
        UInt256,           -- amount
        FixedString(42)    -- recipient
    )) MATERIALIZED (
        arrayMap(
            x -> tuple(
                toUInt8(JSONExtract(x, 'item_type', 'UInt8')),
                JSONExtract(x, 'token', 'FixedString(42)'),
                toUInt256(JSONExtract(x, 'identifier', 'String')),
                toUInt256(JSONExtract(x, 'amount', 'String')),
                JSONExtract(x, 'recipient', 'FixedString(42)')
            ),
            JSONExtractArrayRaw(consideration_raw)
        )
    );

-- Seaport Orders Matched --
CREATE TABLE IF NOT EXISTS seaport_orders_matched AS TEMPLATE_LOG
COMMENT 'Seaport OrdersMatched events';
ALTER TABLE seaport_orders_matched
    -- event --
    ADD COLUMN IF NOT EXISTS order_hashes_raw       String,
    ADD COLUMN IF NOT EXISTS order_hashes           Array(String) MATERIALIZED splitByChar(',', order_hashes_raw);

-- Seaport Order Cancelled --
CREATE TABLE IF NOT EXISTS seaport_order_cancelled AS TEMPLATE_LOG
COMMENT 'Seaport OrderCancelled events';
ALTER TABLE seaport_order_cancelled
    -- event --
    ADD COLUMN IF NOT EXISTS order_hash           String,
    ADD COLUMN IF NOT EXISTS offerer              String,
    ADD COLUMN IF NOT EXISTS zone                 String;
