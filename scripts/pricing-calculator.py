#!/usr/bin/env python3
"""
Substreams EVM Pricing Calculator

Calculate monthly costs based on processed bytes and blocks.
"""

# ─── Chain Profiles ───────────────────────────────────────────────────────────
# Each profile: (bytes_per_10k_blocks_gb, blocks_per_month)
CHAINS = {
    "eth-mainnet":  {"bytes_per_10k": 1.1,  "blocks_per_month": 216_000},
    "bsc":          {"bytes_per_10k": 0.284, "blocks_per_month": 5_760_000},
    "polygon":      {"bytes_per_10k": 2.5,   "blocks_per_month": 1_296_000},
    "arbitrum":     {"bytes_per_10k": 0.8,   "blocks_per_month": 6_480_000},
    "optimism":     {"bytes_per_10k": 0.4,   "blocks_per_month": 1_296_000},
    "base":         {"bytes_per_10k": 1.9,   "blocks_per_month": 1_296_000},
    "avalanche":    {"bytes_per_10k": 1.0,   "blocks_per_month": 1_296_000},
    "unichain":     {"bytes_per_10k": 0.040, "blocks_per_month": 2_592_000},
    "hypercore":    {"bytes_per_10k": 0.028, "blocks_per_month": 12_960_000},
}

# ─── Pricing ──────────────────────────────────────────────────────────────────
COST_PER_TB = 150.00        # $/month
COST_PER_1M_BLOCKS = 1.75   # $/month


def calc_monthly_bytes_gb(bytes_per_10k_gb: float, blocks_per_month: int) -> float:
    """Total GB processed per month."""
    return bytes_per_10k_gb * (blocks_per_month / 10_000)


def calc_cost(bytes_per_10k_gb: float, blocks_per_month: int,
              cost_per_tb: float = COST_PER_TB,
              cost_per_1m_blocks: float = COST_PER_1M_BLOCKS) -> dict:
    monthly_gb = calc_monthly_bytes_gb(bytes_per_10k_gb, blocks_per_month)
    monthly_tb = monthly_gb / 1_000

    bytes_cost = monthly_tb * cost_per_tb
    blocks_cost = (blocks_per_month / 1_000_000) * cost_per_1m_blocks
    total = bytes_cost + blocks_cost

    return {
        "monthly_gb": monthly_gb,
        "monthly_tb": monthly_tb,
        "bytes_cost": bytes_cost,
        "blocks_cost": blocks_cost,
        "total": total,
    }


def fmt_usd(v: float) -> str:
    return f"${v:,.2f}"


def print_chain(name: str, bytes_per_10k: float, blocks_per_month: int,
                cost_per_tb: float = COST_PER_TB,
                cost_per_1m_blocks: float = COST_PER_1M_BLOCKS):
    r = calc_cost(bytes_per_10k, blocks_per_month, cost_per_tb, cost_per_1m_blocks)
    print(f"  {'Chain':<22} {name}")
    print(f"  {'Bytes / 10k blocks':<22} {bytes_per_10k:.1f} GB")
    print(f"  {'Blocks / month':<22} {blocks_per_month:,}")
    print(f"  {'Monthly data':<22} {r['monthly_gb']:,.1f} GB  ({r['monthly_tb']:.3f} TB)")
    print(f"  {'Bytes cost':<22} {fmt_usd(r['bytes_cost'])}")
    print(f"  {'Blocks cost':<22} {fmt_usd(r['blocks_cost'])}")
    print(f"  {'Total / month':<22} {fmt_usd(r['total'])}")
    print()


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Substreams EVM Pricing Calculator")
    parser.add_argument("--chain", choices=list(CHAINS.keys()), help="Use a preset chain profile")
    parser.add_argument("--bytes-per-10k", type=float, help="GB per 10,000 blocks (custom)")
    parser.add_argument("--blocks-per-month", type=int, help="Blocks per month (custom)")
    parser.add_argument("--cost-per-tb", type=float, default=COST_PER_TB, help=f"Cost per TB/month (default: ${COST_PER_TB})")
    parser.add_argument("--cost-per-1m-blocks", type=float, default=COST_PER_1M_BLOCKS, help=f"Cost per 1M blocks/month (default: ${COST_PER_1M_BLOCKS})")
    parser.add_argument("--all", action="store_true", help="Show pricing for all preset chains")
    args = parser.parse_args()

    cost_per_tb = args.cost_per_tb
    cost_per_1m_blocks = args.cost_per_1m_blocks

    print()
    print("=" * 50)
    print("  Substreams EVM Pricing Calculator")
    print("=" * 50)
    print(f"  Rate: {fmt_usd(cost_per_tb)}/TB   {fmt_usd(cost_per_1m_blocks)}/1M blocks")
    print("=" * 50)
    print()

    if args.all:
        for name, profile in CHAINS.items():
            print_chain(name, profile["bytes_per_10k"], profile["blocks_per_month"], cost_per_tb, cost_per_1m_blocks)
            print("-" * 50)

        # summary table
        print()
        print(f"  {'Chain':<16} {'Data/mo':>10} {'Bytes $':>10} {'Blocks $':>10} {'Total $':>10}")
        print(f"  {'-'*16} {'-'*10} {'-'*10} {'-'*10} {'-'*10}")
        grand_total = 0.0
        for name, profile in CHAINS.items():
            r = calc_cost(profile["bytes_per_10k"], profile["blocks_per_month"], cost_per_tb, cost_per_1m_blocks)
            grand_total += r["total"]
            print(f"  {name:<16} {r['monthly_gb']:>8,.1f}GB {fmt_usd(r['bytes_cost']):>10} {fmt_usd(r['blocks_cost']):>10} {fmt_usd(r['total']):>10}")
        print(f"  {'-'*16} {'-'*10} {'-'*10} {'-'*10} {'-'*10}")
        print(f"  {'TOTAL':<16} {'':>10} {'':>10} {'':>10} {fmt_usd(grand_total):>10}")
        print()

    elif args.chain:
        profile = CHAINS[args.chain]
        print_chain(args.chain, profile["bytes_per_10k"], profile["blocks_per_month"], cost_per_tb, cost_per_1m_blocks)

    elif args.bytes_per_10k and args.blocks_per_month:
        print_chain("custom", args.bytes_per_10k, args.blocks_per_month, cost_per_tb, cost_per_1m_blocks)

    else:
        # Default: show eth-mainnet as example
        print("  No chain specified — showing Eth Mainnet example:\n")
        profile = CHAINS["eth-mainnet"]
        print_chain("eth-mainnet", profile["bytes_per_10k"], profile["blocks_per_month"], cost_per_tb, cost_per_1m_blocks)
        print("  Usage:")
        print("    python pricing-calculator.py --chain eth-mainnet")
        print("    python pricing-calculator.py --bytes-per-10k 1.1 --blocks-per-month 219000")
        print("    python pricing-calculator.py --all")
        print("    python pricing-calculator.py --all --cost-per-tb 120 --cost-per-1m-blocks 1.50")
        print()


if __name__ == "__main__":
    main()
