#!/usr/bin/env python3
"""Investigate mapping layouts across observed ERC-20-shaped Transfer contracts.

RPC agreement here is evidence for adapter development, not automatic support.
Unknown layouts, rebases, proxy/code changes and unobserved holders remain open.
"""
import argparse
import hashlib
import json
from pathlib import Path

from compare import (Rpc, PACKAGE_DIR, balance_request, balance_result,
                     batch_responses, binary, read_stream, require, stream, uint)


def classify_layout(stats):
    if stats["code_changed"]:
        return "code_change_requires_review"
    if stats["rpc_errors"]:
        return "rpc_unresolved"
    if stats["mismatches"]:
        return "not_direct_balance_mapping"
    if stats["nonzero_holders"] < 2 or stats["changed_observations"] < 2:
        return "insufficient_evidence"
    return "candidate_matches_rpc_not_qualified"


def analyze(rpc, blocks, output):
    layouts, tokens = {}, {}
    with (output/"checks.jsonl").open("w") as raw:
        for number, block in blocks.items():
            digest = binary(block["hash"], 32)
            parent = binary(block["parentHash"], 32)
            require(rpc.header(number)["hash"].lower() == digest, "probe RPC block differs")
            for token in block.get("tokens", []):
                contract = binary(token["contract"], 20)
                entry = tokens.setdefault(contract, {"contract": contract, "blocks": 0, "storage_changes": 0,
                    "unclassified_storage_changes": 0, "code_changed": False, "holders": set(), "rpc_validated_holders": set()})
                entry["blocks"] += 1
                entry["storage_changes"] += int(token.get("storageChanges", 0))
                entry["unclassified_storage_changes"] += int(token.get("unclassifiedStorageChanges", 0))
                entry["code_changed"] |= token.get("codeChanged", False)
                entry["holders"].update(binary(a, 20) for a in token.get("transferHolders", []))
            requests = {}
            rows = block.get("candidates", [])
            for row in rows:
                contract, address = binary(row["contract"], 20), binary(row["address"], 20)
                for boundary, h in (("before", parent), ("after", digest)):
                    requests[contract, address, boundary] = balance_request(contract, address, {"blockHash": h, "requireCanonical": True})
            observations = {}
            items = list(requests.items())
            for offset in range(0, len(items), 25):
                chunk = items[offset:offset+25]
                payload = [{"jsonrpc": "2.0", "id": i, "method": call[0], "params": call[1]} for i, (_, call) in enumerate(chunk)]
                responses = batch_responses(rpc.request(payload), len(chunk))
                for (key, _), response in zip(chunk, responses):
                    if response.get("error") or response.get("result") is None:
                        observations[key] = None
                    else:
                        try: observations[key] = balance_result(response["result"], True)
                        except (ValueError, TypeError): observations[key] = None
            for row in rows:
                contract, address = binary(row["contract"], 20), binary(row["address"], 20)
                slot = binary(row["mappingSlot"], 32)
                before, after = uint(row.get("oldAmount", "0")), uint(row.get("amount", "0"))
                stats = layouts.setdefault((contract, slot), {"contract": contract, "mapping_slot": slot,
                    "observations": 0, "mismatches": 0, "rpc_errors": 0, "changed_observations": 0,
                    "holders": set(), "nonzero_holders": set(), "code_changed": False})
                stats["observations"] += 1
                stats["changed_observations"] += before != after
                stats["holders"].add(address)
                stats["code_changed"] |= tokens[contract]["code_changed"]
                matched = True
                for boundary, expected, h in (("before", before, parent), ("after", after, digest)):
                    actual = observations[contract, address, boundary]
                    match = actual is not None and actual == expected
                    stats["rpc_errors"] += actual is None
                    stats["mismatches"] += actual is not None and not match
                    matched &= match
                    raw.write(json.dumps({"block": number, "hash": h, "boundary": boundary, "contract": contract,
                        "address": address, "mapping_slot": slot, "storage_key": binary(row["storageKey"], 32),
                        "candidate": str(expected), "rpc": str(actual) if actual is not None else None,
                        "match": match})+"\n")
                if matched:
                    tokens[contract]["rpc_validated_holders"].add(address)
                    if before or after: stats["nonzero_holders"].add(address)
            raw.flush()
            require(rpc.header(number)["hash"].lower() == digest, "probe RPC header changed")
            print(f"Probed block {number}: {len(block.get('tokens', []))} token-like contracts, {len(rows)} mapping candidates", flush=True)
    results = []
    for stats in layouts.values():
        stats["holders"] = len(stats["holders"])
        stats["nonzero_holders"] = len(stats["nonzero_holders"])
        stats["code_changed"] |= tokens[stats["contract"]]["code_changed"]
        stats["classification"] = classify_layout(stats)
        results.append(stats)
    token_results = []
    for token in tokens.values():
        matches = [r for r in results if r["contract"] == token["contract"] and r["classification"] == "candidate_matches_rpc_not_qualified"]
        token["matching_layout_count"] = len(matches)
        token["ambiguous_matching_layouts"] = len(matches) > 1
        token["observed_transfer_holders"] = len(token.pop("holders"))
        token["holders_with_some_matching_candidate"] = len(token.pop("rpc_validated_holders"))
        token_results.append(token)
    return {"token_like_contracts": len(tokens), "mapping_layout_candidates": len(results),
            "candidate_contracts_matching_rpc": len({r["contract"] for r in results if r["classification"] == "candidate_matches_rpc_not_qualified"}),
            "ambiguous_matching_contracts": sum(t["ambiguous_matching_layouts"] for t in token_results),
            "candidate_value_checks": sum(r["observations"]*2 for r in results),
            "layouts": sorted(results, key=lambda r: (r["contract"], r["mapping_slot"])),
            "tokens": sorted(token_results, key=lambda r: r["contract"]),
            "checks_sha256": hashlib.sha256((output/"checks.jsonl").read_bytes()).hexdigest()}


def run(args):
    require(1 <= args.blocks <= 128 and args.start > 0, "choose a positive start and 1..128 blocks")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {"status": "incomplete", "start": args.start, "blocks": args.blocks,
              "scope": "Contracts with ERC-20-shaped Transfer logs in this window; direct mapping hypotheses only",
              "promoted_adapters": 0}
    try:
        rpc = Rpc()
        require(int(rpc.call("eth_chainId", []), 16) == 56, "BSC required")
        require(args.start+args.blocks-1 <= int(rpc.call("eth_getBlockByNumber", ["finalized", False])["number"], 16), "range not finalized")
        report["capture"] = stream(args.package, "map_erc20_candidates", args.endpoint, args.start, args.start+args.blocks,
                                   output/"candidates.jsonl", 300)
        blocks = read_stream(output/"candidates.jsonl", args.start, args.start+args.blocks, "map_erc20_candidates")
        prior = None
        for number, block in blocks.items():
            require(int(block["number"]) == number, "probe number mismatch")
            require(prior is None or binary(block["parentHash"], 32) == prior, "probe gap/fork")
            prior = binary(block["hash"], 32)
        report.update(analyze(rpc, blocks, output))
        report["status"] = "discovery_only"
    except Exception as exc:
        report["failure"] = f"{type(exc).__name__}: {exc}"
    (output/"report.json").write_text(json.dumps(report, indent=2)+"\n")
    print(json.dumps({k:v for k,v in report.items() if k not in ("layouts", "tokens")}, indent=2))
    return report["status"] == "discovery_only"


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--start", required=True, type=int)
    p.add_argument("--blocks", type=int, default=16)
    p.add_argument("--output", required=True, type=Path)
    p.add_argument("--endpoint", default="bsc.substreams.pinax.network:443")
    p.add_argument("--package", type=Path, default=PACKAGE_DIR.parent/"spkg/evm-balances-storage-v0.1.0.spkg")
    raise SystemExit(0 if run(p.parse_args()) else 1)
