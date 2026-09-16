#!/usr/bin/env python3
"""Check every emitted balance against historical RPC before and after its block.

Uses EIP-1898 blockHash/requireCanonical; no fallback to latest or height-only.
This validates emitted changes, not undiscovered holders or complete state.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import time

from compare import (Rpc, WBNB, PACKAGE_DIR, balance_request, balance_result,
                     binary, candidate_rows, read_stream, require, stream)


def audit_block(rpc, block, batch_size):
    number = int(block["number"])
    block_hash, parent = binary(block["hash"], 32), binary(block["parentHash"], 32)
    require(rpc.header(number)["hash"].lower() == block_hash, "RPC block identity mismatch")
    require(not block.get("unresolvedWbnbSlots"), "unresolved storage prevents qualification")
    pending = []
    for (contract, address), (before, after) in candidate_rows(block).items():
        for boundary, expected, height, digest in (("before", before, number-1, parent), ("after", after, number, block_hash)):
            pending.append(({"block": number, "at_height": height, "hash": digest,
                             "boundary": boundary, "contract": contract, "address": address,
                             "storage": str(expected)}, balance_request(contract, address, {"blockHash": digest, "requireCanonical": True})))
    checks = []
    for offset in range(0, len(pending), batch_size):
        chunk = pending[offset:offset+batch_size]
        values = rpc.batch([call for _, call in chunk])
        for (check, _), value in zip(chunk, values):
            actual = balance_result(value, bool(check["contract"]))
            checks.append(dict(check, rpc=str(actual), match=str(actual) == check["storage"]))
    require(rpc.header(number)["hash"].lower() == block_hash, "RPC header changed during audit")
    return checks


def run(args):
    require(1 <= args.blocks <= 2048 and args.start > 0, "choose a positive start and 1..2048 blocks")
    require(1 <= args.workers <= 4 and 1 <= args.batch_size <= 100, "workers 1..4; batch size 1..100")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {"status": "incomplete", "start": args.start, "blocks": args.blocks, "checks": 0,
              "native_checks": 0, "token_checks": 0, "zero_checks": 0, "mismatches": 0,
              "before_checks": 0, "after_checks": 0, "checked_blocks": 0,
              "rpc_block_binding": "EIP-1898 blockHash, requireCanonical=true",
              "scope": "Every emitted native BNB/WBNB balance before and after each block; not full holder discovery"}
    started = time.monotonic()
    try:
        stop = args.start+args.blocks
        rpc = Rpc()
        require(int(rpc.call("eth_chainId", []), 16) == 56, "BSC chain ID required")
        finalized = rpc.call("eth_getBlockByNumber", ["finalized", False])
        require(stop-1 <= int(finalized["number"], 16), "range is not finalized")
        timing = stream(args.package, "map_balances", args.endpoint, args.start, stop, output/"storage.jsonl", args.timeout)
        report["capture"] = timing
        candidate = read_stream(output/"storage.jsonl", args.start, stop, "map_balances")
        runtime = (PACKAGE_DIR/"tests/fixtures/wbnb-runtime.hex").read_text().strip().lower()
        for number in (args.start-1, stop-1):
            h = rpc.header(number)
            ref = {"blockHash": h["hash"], "requireCanonical": True}
            require(rpc.call("eth_getCode", [WBNB, ref]).lower() == runtime, "unqualified WBNB runtime")
        previous = None
        for number, block in candidate.items():
            require(int(block["number"]) == number, "candidate number mismatch")
            parent = binary(block["parentHash"], 32)
            require(previous is None or parent == previous, "candidate gap/fork")
            previous = binary(block["hash"], 32)
        report["first_hash"] = binary(candidate[args.start]["hash"], 32)
        report["last_hash"] = previous
        with (output/"rpc-checks.jsonl").open("w") as raw, ThreadPoolExecutor(max_workers=args.workers) as pool:
            for checks in pool.map(lambda b: audit_block(rpc, b, args.batch_size), candidate.values()):
                for check in checks:
                    raw.write(json.dumps(check, separators=(",", ":"))+"\n")
                    report["checks"] += 1
                    report["token_checks" if check["contract"] else "native_checks"] += 1
                    report[check["boundary"]+"_checks"] += 1
                    report["zero_checks"] += check["storage"] == "0"
                    report["mismatches"] += not check["match"]
                raw.flush()
                report["checked_blocks"] += 1
                if report["checked_blocks"] % 16 == 0:
                    print(f"Audited {report['checked_blocks']}/{args.blocks} blocks: {report['checks']} RPC balances, {report['mismatches']} mismatches", flush=True)
        report["checks_sha256"] = hashlib.sha256((output/"rpc-checks.jsonl").read_bytes()).hexdigest()
        report["status"] = "rpc_parity" if report["checks"] and not report["mismatches"] else "mismatch"
    except Exception as exc:
        report["failure"] = f"{type(exc).__name__}: {exc}"
    report["elapsed_seconds"] = time.monotonic()-started
    (output/"report.json").write_text(json.dumps(report, indent=2)+"\n")
    print(json.dumps(report, indent=2))
    return report["status"] == "rpc_parity"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--start", required=True, type=int)
    parser.add_argument("--blocks", type=int, default=64)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--package", type=Path, default=PACKAGE_DIR.parent/"spkg/evm-balances-storage-v0.1.0.spkg")
    parser.add_argument("--endpoint", default="bsc.substreams.pinax.network:443")
    parser.add_argument("--timeout", type=int, default=300)
    parser.add_argument("--workers", type=int, default=1)
    parser.add_argument("--batch-size", type=int, default=25)
    raise SystemExit(0 if run(parser.parse_args()) else 1)
