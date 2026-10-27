# Pool creation fixture

`uniswap-v2-pair-created-receipt.json` is the complete, unmodified JSON result of
`eth_getTransactionReceipt` from the public endpoint
`https://ethereum.publicnode.com`, captured on 2026-10-27. The accompanying
`uniswap-v2-pair-created-provenance.json` records the request source, capture time,
receipt SHA-256, and block metadata returned by `eth_getBlockByHash`.

The transaction is
[`0x62c52cd4c81f190aaa83a4d786ce6a18d2593051e742037e897708c8addc8ca6`](https://etherscan.io/tx/0x62c52cd4c81f190aaa83a4d786ce6a18d2593051e742037e897708c8addc8ca6)
at Ethereum mainnet block 26,283,930, transaction index 114. It succeeded and its
eight receipt logs include one `PairCreated` from the Uniswap V2 factory at block
log index 235. The observed pair is
`0xa64cab535075bb67432d650e6fb9049e7534cec0`; its tokens are WETH
`0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2` followed by
`0xf34fe3655aaef1f0c15547d14ba723eed97e7e1b`.

Discovery was bounded to 1,000 blocks (26,283,819 through 26,284,818) with the exact
factory address and `PairCreated` topic. This fixture contains one receipt, not a
complete Firehose block or call trace. Tests wrap the selected transaction in a
receipt-only `sf.ethereum.type.v2.Block`, preserve the receipt's log order, and
leave unavailable call traces and Firehose ordinals unset. They do not fabricate
execution evidence or claim Curve trace/constructor coverage from this capture.

All other fixtures in `decoder_parity.rs` are explicitly synthetic, based on
the ABI declarations in `substreams-abis` v1.5.0, commit
`62b75b660dc0035bd4767d4e4d8ccdb92fcd98a3`.
