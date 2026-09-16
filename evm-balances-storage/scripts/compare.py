#!/usr/bin/env python3
"""Bounded finalized comparison. RPC is used only by this qualification tool.

Raw Substreams outputs and a separate SQLite evidence database are retained.
Amounts remain uint256 decimal strings; missing values are never zero-filled.
"""
import argparse
import base64
from concurrent.futures import ThreadPoolExecutor
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import time
import urllib.request
import urllib.error

PACKAGE_DIR = Path(__file__).resolve().parents[1]
WBNB = "0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def binary(value, size):
    raw = bytes.fromhex(value[2:]) if value.startswith("0x") else base64.b64decode(value, validate=True)
    require(len(raw) == size, f"expected {size} bytes")
    return "0x" + raw.hex()


def uint(value):
    require(isinstance(value, (int, str)) and not isinstance(value, bool), "invalid integer")
    n = int(value)
    require(0 <= n < 2**256, "amount outside uint256")
    return n


def read_stream(path, start, stop, module):
    blocks = {}
    for line in Path(path).read_text().splitlines():
        row = json.loads(line)
        require(row.get("@module") == module, "unexpected module output")
        number = int(row["@block"])
        require(number not in blocks, "duplicate block output")
        blocks[number] = row["@data"]
    require(sorted(blocks) == list(range(start, stop)), "incomplete or out-of-range stream")
    return blocks


def candidate_rows(block):
    result = {}
    for row in block.get("balances", []):
        contract = binary(row["contract"], 20) if row.get("contract") else ""
        require(contract in ("", WBNB), "unsupported contract in candidate")
        key = contract, binary(row["address"], 20)
        require(key not in result, "duplicate candidate balance")
        result[key] = uint(row.get("oldAmount", "0")), uint(row.get("amount", "0"))
    return result


def reference_rows(block, number, block_hash):
    result = {}
    marker = False
    for row in block.get("tableChanges", []):
        fields = {f["name"]: f.get("value", "") for f in row.get("fields", [])}
        require(uint(fields["block_num"]) == number and fields["block_hash"].lower() == block_hash,
                "reference block identity mismatch")
        table = row["table"]
        if table == "blocks":
            marker = True
            continue
        if table not in ("native_balances", "erc20_balances"):
            continue
        contract = binary(fields["contract"], 20) if table == "erc20_balances" else ""
        if contract not in ("", WBNB):
            continue
        key = contract, binary(fields["address"], 20)
        require(key not in result, "duplicate reference balance")
        result[key] = uint(fields["balance"])
    require(marker or not block.get("tableChanges"), "reference missing block marker")
    return result


def compare(candidate, reference, start, stop, database):
    with closing(sqlite3.connect(database)) as db:
        return compare_in_database(candidate, reference, start, stop, db)


def compare_in_database(candidate, reference, start, stop, db):
    require(sorted(candidate) == sorted(reference) == list(range(start, stop)), "different block ranges")
    db.executescript("""
      CREATE TABLE observations(side TEXT, block_num INTEGER, block_hash TEXT,
        contract TEXT, address TEXT, balance TEXT,
        PRIMARY KEY(side,block_num,contract,address));
      CREATE TABLE differences(block_num INTEGER, contract TEXT, address TEXT,
        candidate TEXT, reference TEXT, kind TEXT);
    """)
    ours, theirs, changed = {}, {}, set()
    summary = {"start": start, "stop_exclusive": stop, "blocks": stop-start,
               "native_updates": 0, "wbnb_updates": 0, "wbnb_storage_changes": 0,
               "preimages": 0, "unresolved_slots": 0, "reference_bootstrap_seeds": 0,
               "snapshot_comparisons": 0, "seed_only_comparisons": 0,
               "differences": 0, "candidate_only_keys": 0}
    previous = None
    for number in range(start, stop):
        block = candidate[number]
        require(int(block["number"]) == number, "candidate number mismatch")
        block_hash = binary(block["hash"], 32)
        parent = binary(block["parentHash"], 32)
        require(previous is None or parent == previous, "candidate gap or fork")
        previous = block_hash
        unresolved = block.get("unresolvedWbnbSlots", [])
        summary["unresolved_slots"] += len(unresolved)
        new, old = candidate_rows(block), reference_rows(reference[number], number, block_hash)
        summary["preimages"] += int(block.get("preimageCount", 0))
        summary["wbnb_storage_changes"] += int(block.get("wbnbStorageChanges", 0))
        for key, (before, after) in new.items():
            if key in ours:
                require(ours[key] == before, f"candidate state continuity mismatch at {number}: {key}")
            ours[key] = after
            changed.add(key)
            summary["wbnb_updates" if key[0] else "native_updates"] += 1
        # Reference-only initial observations seed unknown unchanged accounts.
        # These seeds are counted separately, never reported as verified updates.
        for key, value in old.items():
            theirs[key] = value
            if key not in ours:
                ours[key] = value
                summary["reference_bootstrap_seeds"] += 1
        for side, observations in (("storage", {k:v[1] for k,v in new.items()}), ("rpc-package", old)):
            db.executemany("INSERT INTO observations VALUES(?,?,?,?,?,?)",
                           [(side, number, block_hash, *key, str(value)) for key, value in observations.items()])
        for key in sorted(ours.keys() & theirs.keys()):
            summary["snapshot_comparisons" if key in changed else "seed_only_comparisons"] += 1
            if ours[key] != theirs[key]:
                summary["differences"] += 1
                db.execute("INSERT INTO differences VALUES(?,?,?,?,?,?)", (number, *key, str(ours[key]), str(theirs[key]), "value"))
    missing = sorted(ours.keys() - theirs.keys())
    summary["candidate_only_keys"] = len(missing)
    for key in missing:
        db.execute("INSERT INTO differences VALUES(?,?,?,?,?,?)", (stop-1, *key, str(ours[key]), None, "reference_unobserved"))
    summary["distinct_storage_keys"] = len(changed)
    summary["final_hash"] = previous
    summary["status"] = ("mismatch" if summary["differences"] or summary["unresolved_slots"]
                         else "reference_coverage_gap" if missing else "bounded_parity")
    db.commit()
    return summary, ours, changed


class Rpc:
    def __init__(self):
        self.url = os.getenv("RPC_URL", "https://bsc.rpc.pinax.network")
        self.key = os.getenv("RPC_API_KEY") or os.getenv("SUBSTREAMS_API_KEY")

    def call(self, method, params):
        result = self.request({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
        require(isinstance(result, dict) and result.get("id") == 1, "invalid RPC response ID")
        require(not result.get("error") and result.get("result") is not None, f"RPC {method} returned an error/null")
        return result["result"]

    def request(self, payload):
        headers = {"Content-Type": "application/json"}
        if self.key:
            headers["X-Api-Key"] = self.key
        request = urllib.request.Request(self.url, headers=headers, data=json.dumps(payload).encode())
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                result = json.load(response)
        except urllib.error.HTTPError as exc:
            raise RuntimeError(f"RPC HTTP {exc.code}") from None
        except Exception:
            raise RuntimeError("RPC transport failed") from None
        return result

    def batch(self, calls):
        require(bool(calls), "empty RPC batch")
        payload = [{"jsonrpc": "2.0", "id": i, "method": method, "params": params}
                   for i, (method, params) in enumerate(calls)]
        return batch_results(self.request(payload), len(calls))

    def header(self, number):
        h = self.call("eth_getBlockByNumber", [hex(number), False])
        require(int(h["number"], 16) == number, "RPC returned wrong height")
        return h

    def balance(self, contract, address, number):
        method, params = balance_request(contract, address, hex(number))
        return balance_result(self.call(method, params), bool(contract))


def balance_request(contract, address, block_reference):
    if contract:
        return "eth_call", [{"to": contract, "data": "0x70a08231"+address[2:].rjust(64, "0")}, block_reference]
    return "eth_getBalance", [address, block_reference]


def balance_result(value, token):
    require(isinstance(value, str) and value.startswith("0x"), "invalid RPC balance encoding")
    if token:
        require(len(value) == 66, "balanceOf must return exactly one uint256 word")
    return uint(int(value, 16))


def batch_results(response, count):
    rows = batch_responses(response, count)
    for row in rows:
        require(not row.get("error") and row.get("result") is not None, "RPC batch error/null result")
    return [row["result"] for row in rows]


def batch_responses(response, count):
    require(isinstance(response, list) and len(response) == count, "incomplete RPC batch")
    results = {}
    for row in response:
        require(isinstance(row, dict), "invalid RPC batch row")
        key = row.get("id")
        require(type(key) is int and 0 <= key < count and key not in results, "duplicate or invalid RPC batch ID")
        results[key] = row
    return [results[i] for i in range(count)]


def audit_differences(rpc, database, candidate, limit):
    """Preserve disagreements and check each at its original block, up to limit."""
    with closing(sqlite3.connect(database)) as db:
        differences = db.execute("SELECT * FROM differences WHERE kind='value' ORDER BY block_num,contract,address").fetchall()
    checks = []
    for number, contract, address, ours, theirs, _ in differences[:limit]:
        block_hash = binary(candidate[number]["hash"], 32)
        require(rpc.header(number)["hash"].lower() == block_hash, "mismatch audit block differs from RPC")
        actual = rpc.balance(contract, address, number)
        require(rpc.header(number)["hash"].lower() == block_hash, "mismatch audit RPC header changed")
        checks.append({"block": number, "hash": block_hash, "contract": contract, "address": address,
                       "storage": ours, "reference": theirs, "rpc": str(actual),
                       "agrees_with": "storage" if actual == uint(ours) else "reference" if actual == uint(theirs) else "neither"})
    return {"checks": checks, "unchecked": len(differences)-len(checks),
            "agrees_with_storage": sum(c["agrees_with"] == "storage" for c in checks),
            "agrees_with_reference": sum(c["agrees_with"] == "reference" for c in checks),
            "agrees_with_neither": sum(c["agrees_with"] == "neither" for c in checks)}


def stream(package, module, endpoint, start, stop, output, timeout):
    command = ["substreams", "run", str(package), module, "-e", endpoint, "-s", str(start), "-t", str(stop),
               "--final-blocks-only", "--max-retries", "0", "-o", "jsonl"]
    if endpoint.endswith(":80") or endpoint.startswith("http://"):
        command.append("--plaintext")
    began = time.monotonic()
    with output.open("w") as stdout, output.with_suffix(".log").open("w") as stderr:
        try:
            process = subprocess.run(command, stdout=stdout, stderr=stderr, timeout=timeout)
        except subprocess.TimeoutExpired:
            raise RuntimeError(f"{module} capture timed out; see {output.with_suffix('.log')}") from None
    require(process.returncode == 0, f"{module} failed; see {output.with_suffix('.log')}")
    elapsed = time.monotonic()-began
    return {"seconds_including_startup": elapsed, "blocks_per_second_including_startup": (stop-start)/elapsed,
            "package_sha256": hashlib.sha256(Path(package).read_bytes()).hexdigest()}


def run(args):
    start, stop = args.start, args.start+args.blocks
    require(start > 0 and 1 <= args.blocks <= 10000, "choose 1..10000 blocks and a positive start")
    require(args.rpc_samples >= 2, "at least two independent RPC samples required")
    require(args.audit_mismatches >= 0, "audit limit must be nonnegative")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    rpc = Rpc()
    require(int(rpc.call("eth_chainId", []), 16) == 56, "BSC chain ID required")
    finalized = rpc.call("eth_getBlockByNumber", ["finalized", False])
    require(stop-1 <= int(finalized["number"], 16), "range extends beyond finality")
    first, last = rpc.header(start), rpc.header(stop-1)
    runtime = (PACKAGE_DIR/"tests/fixtures/wbnb-runtime.hex").read_text().strip().lower()
    for number in (start, stop-1):
        require(rpc.call("eth_getCode", [WBNB, hex(number)]).lower() == runtime, "WBNB runtime differs from pinned adapter")
    print(f"Comparing finalized BSC blocks {start}..{stop-1}", flush=True)
    with ThreadPoolExecutor(max_workers=2) as pool:
        jobs = {"storage": pool.submit(stream, args.package, "map_balances", args.endpoint, start, stop, output/"storage.jsonl", args.timeout),
                "reference": pool.submit(stream, args.reference, "db_out", args.endpoint, start, stop, output/"reference.jsonl", args.timeout)}
        timing = {name: job.result() for name, job in jobs.items()}
    candidate = read_stream(output/"storage.jsonl", start, stop, "map_balances")
    reference = read_stream(output/"reference.jsonl", start, stop, "db_out")
    require(binary(candidate[start]["hash"], 32) == first["hash"].lower(), "first block differs from RPC")
    require(binary(candidate[stop-1]["hash"], 32) == last["hash"].lower(), "last block differs from RPC")
    report, state, changed = compare(candidate, reference, start, stop, output/"comparison.sqlite")
    # Deterministic, stratified samples include native and WBNB; use keys whose
    # balances were actually derived from state changes, excluding bootstrap-only seeds.
    groups = [sorted(k for k in changed if bool(k[0]) == token) for token in (False, True)]
    selected = []
    for group in groups:
        count = min(len(group), args.rpc_samples//2)
        selected.extend(group[i*len(group)//count] for i in range(count))
    checks = []
    for contract, address in selected:
        actual = rpc.balance(contract, address, stop-1)
        checks.append({"contract": contract, "address": address, "block": stop-1, "hash": report["final_hash"],
                       "storage": str(state[contract,address]), "rpc": str(actual), "match": state[contract,address] == actual})
    require(rpc.header(stop-1)["hash"].lower() == report["final_hash"], "RPC header changed during comparison")
    require(rpc.header(start)["hash"].lower() == first["hash"].lower(), "RPC start header changed")
    audit = audit_differences(rpc, output/"comparison.sqlite", candidate, args.audit_mismatches)
    report.update({"timing": timing, "independent_rpc_checks": checks, "mismatch_rpc_audit": audit,
                   "scope": "Native BNB and pinned WBNB changed keys; reference-seeded unknown accounts; not complete holder bootstrap",
                   "rpc_in_ingestion": False, "chain_id": 56, "finality_trust": "RPC provider finalized header"})
    if not checks or any(not c["match"] for c in checks):
        report["status"] = "mismatch"
    elif (report["differences"] and not report["unresolved_slots"] and not report["candidate_only_keys"]
          and not audit["unchecked"] and audit["agrees_with_storage"] == report["differences"]):
        report["status"] = "reference_disagreement"
    (output/"report.json").write_text(json.dumps(report, indent=2)+"\n")
    print(json.dumps({k:({a:b for a,b in v.items() if a != "checks"} if k == "mismatch_rpc_audit" else v)
                      for k,v in report.items() if k != "independent_rpc_checks"}, indent=2))
    return report["status"] == "bounded_parity"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--start", required=True, type=int)
    parser.add_argument("--blocks", type=int, default=64)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--package", type=Path, default=PACKAGE_DIR.parent/"spkg/evm-balances-storage-v0.1.0.spkg")
    parser.add_argument("--reference", type=Path, default=PACKAGE_DIR.parent/"spkg/evm-balances-v0.3.3.spkg")
    parser.add_argument("--endpoint", default="bsc.substreams.pinax.network:443")
    parser.add_argument("--timeout", type=int, default=300)
    parser.add_argument("--rpc-samples", type=int, default=20)
    parser.add_argument("--audit-mismatches", type=int, default=256)
    args = parser.parse_args()
    try:
        good = run(args)
    except Exception as exc:
        # Raw captures remain on failure; never print endpoint credentials.
        print(f"Comparison failed: {type(exc).__name__}: {exc}")
        raise SystemExit(1)
    raise SystemExit(0 if good else 1)


if __name__ == "__main__":
    main()
