# ERC-4626

Substreams module that extracts **ERC-4626 Tokenized Vault** deposit/withdraw events.

Matched by event signature (`topic0`), so **every** ERC-4626 vault on a chain is captured with
no address list to maintain — Morpho / MetaMorpho, Yearn v3, Euler v2, Aave stata-aTokens,
LST/LRT wrappers, tokenized-RWA vaults, etc.

## Events

| Event | Signature |
|---|---|
| `Deposit` | `Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares)` |
| `Withdraw` | `Withdraw(address indexed sender, address indexed receiver, address indexed owner, uint256 assets, uint256 shares)` |

Decoded via [`substreams-abis`](https://github.com/pinax-network/substreams-abis) (`standard::erc4626`).

## Output

`map_events` → `proto:erc4626.v1.Events` — per transaction, a list of `Deposit` / `Withdraw`
logs carrying `sender`, `owner`(/`receiver`), `assets` (underlying) and `shares`, plus the shared
block/log/tx/call metadata. The **share price** (`assets / shares`) is a downstream derivation
(e.g. a ClickHouse view in an aggregator) — the module emits both amounts so the price series and
net flows can be computed per vault.

## Run

```bash
make gui                 # defaults to Base
make gui ENDPOINT=mainnet.substreams.pinax.network:443 START_BLOCK=21000000
```

ERC-4626 vault activity is heaviest on **Ethereum** and **Base** (Morpho dominates), then
**Arbitrum**; the module is chain-agnostic and runs on any EVM firehose.

## Notes

- **Fee spread.** Per [EIP-4626](https://eips.ethereum.org/EIPS/eip-4626), the `Deposit` event's
  `assets` includes any entry fee, while `Withdraw` reports the assets received *after* exit fees.
  So the `assets / shares` rate implied by deposits vs. withdrawals differs by exactly the vault's
  fee spread — both amounts are emitted raw so consumers can compute either side.
- **Virtual offset.** Many vaults use OpenZeppelin's virtual-shares/offset defense against inflation
  attacks, so share decimals can exceed the underlying's decimals and `share_price` need not be ~1.0
  on a fresh vault. The module does not normalize decimals; that belongs in the serving layer.
- **Signature-only match.** `Deposit`/`Withdraw` are matched by `topic0`; a non-4626 contract could
  in principle emit a same-signature event. Disambiguate downstream by requiring both signatures to
  co-occur on an address, or a one-shot `asset()` probe.
