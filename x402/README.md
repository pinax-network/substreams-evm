# x402

`x402` extracts settled x402 payment events from EVM chains.

It currently supports:

- EIP-3009 settlements by joining `AuthorizationUsed(authorizer, nonce)`, a matching ERC-20 `Transfer`, and decoded `transferWithAuthorization` calldata when traces are available.
- Permit2 settlements from the canonical `x402ExactPermit2Proxy` events `Settled()` and `SettledWithPermit()`, joined to the token transfer in the same transaction.

The module only sees onchain settlement. HTTP resources, verification attempts, KYT failures, and fulfillment status require facilitator or resource-server logs.

`map_events` does not apply facilitator filtering. It emits all onchain settlement candidates so stricter facilitator rules can be applied later in ClickHouse queries.
