import base64
from contextlib import closing
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest

from compare import WBNB, audit_differences, binary, compare, read_stream, uint


ADDRESS = "0x" + "11"*20
HASH = "0x" + "22"*32
NEXT = "0x" + "33"*32


def candidate(number=1, old="5", new="0", contract="", address=ADDRESS):
    return {"number": str(number), "hash": HASH if number == 1 else NEXT,
            "parentHash": "0x"+"00"*32 if number == 1 else HASH,
            "balances": [{"contract": contract, "address": address, "oldAmount": old, "amount": new}]}


def reference(number=1, amount="0", contract="", address=ADDRESS):
    block_hash = HASH if number == 1 else NEXT
    def fields(**data):
        return [{"name": k, "value": v} for k,v in data.items()]
    metadata = {"block_num": str(number), "block_hash": block_hash}
    return {"tableChanges": [
        {"table": "blocks", "fields": fields(**metadata)},
        {"table": "erc20_balances" if contract else "native_balances",
         "fields": fields(**metadata, address=address, contract=contract, balance=amount)}]}


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.db = Path(self.temp.name)/"comparison.sqlite"

    def run_comparison(self, ours, theirs):
        return compare(ours, theirs, 1, len(ours)+1, self.db)[0]

    def test_uint256_and_explicit_zero(self):
        maximum = str(2**256-1)
        ours = {1: candidate(new=maximum, contract=WBNB), 2: candidate(2, old=maximum, contract=WBNB)}
        theirs = {1: reference(amount=maximum, contract=WBNB), 2: reference(2, contract=WBNB)}
        report = self.run_comparison(ours, theirs)
        self.assertEqual(report["status"], "bounded_parity")
        with closing(sqlite3.connect(self.db)) as db:
            self.assertEqual(db.execute("SELECT balance FROM observations WHERE side='storage' ORDER BY block_num").fetchall(), [(maximum,), ("0",)])

    def test_reference_seed_is_not_a_verified_storage_comparison(self):
        ours = candidate()
        ours["balances"] = []
        report = self.run_comparison({1: ours}, {1: reference()})
        self.assertEqual(report["reference_bootstrap_seeds"], 1)
        self.assertEqual(report["snapshot_comparisons"], 0)
        self.assertEqual(report["seed_only_comparisons"], 1)

    def test_missing_reference_is_not_zero(self):
        ref = reference()
        ref["tableChanges"] = ref["tableChanges"][:1]
        report = self.run_comparison({1: candidate()}, {1: ref})
        self.assertEqual(report["status"], "reference_coverage_gap")
        self.assertEqual(report["candidate_only_keys"], 1)

    def test_reference_missing_later_update_is_a_mismatch(self):
        second = reference(2)
        second["tableChanges"] = second["tableChanges"][:1]
        report = self.run_comparison({1: candidate(), 2: candidate(2, old="0", new="9")}, {1: reference(), 2: second})
        self.assertEqual(report["differences"], 1)

    def test_unresolved_storage_cannot_pass(self):
        ours = candidate()
        ours["unresolvedWbnbSlots"] = [{"key": HASH}]
        self.assertEqual(self.run_comparison({1: ours}, {1: reference()})["status"], "mismatch")

    def test_forks_and_gaps_are_rejected(self):
        second = candidate(2, old="0")
        second["parentHash"] = NEXT
        with self.assertRaisesRegex(ValueError, "gap or fork"):
            self.run_comparison({1: candidate(), 2: second}, {1: reference(), 2: reference(2)})

    def test_wrong_reference_block_hash_is_rejected(self):
        ref = reference()
        ref["tableChanges"][1]["fields"][1]["value"] = NEXT
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            self.run_comparison({1: candidate()}, {1: ref})

    def test_cross_block_storage_discontinuity_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "continuity mismatch"):
            self.run_comparison({1: candidate(), 2: candidate(2, old="5")}, {1: reference(), 2: reference(2)})

    def test_duplicate_candidate_rejected(self):
        ours = candidate()
        ours["balances"] *= 2
        with self.assertRaisesRegex(ValueError, "duplicate candidate"):
            self.run_comparison({1: ours}, {1: reference()})

    def test_stream_requires_exact_range_and_no_duplicates(self):
        path = Path(self.temp.name)/"capture.jsonl"
        row = {"@module": "map_balances", "@block": 1, "@data": candidate()}
        path.write_text(json.dumps(row)+"\n")
        with self.assertRaisesRegex(ValueError, "incomplete"):
            read_stream(path, 1, 3, "map_balances")
        path.write_text((json.dumps(row)+"\n")*2)
        with self.assertRaisesRegex(ValueError, "duplicate block"):
            read_stream(path, 1, 2, "map_balances")

    def test_rpc_audit_preserves_disagreement_and_checks_original_height(self):
        self.run_comparison({1: candidate()}, {1: reference(amount="10")})
        calls = []
        class Rpc:
            def header(self, number):
                calls.append(("header", number))
                return {"hash": HASH}
            def balance(self, contract, address, number):
                calls.append(("balance", contract, address, number))
                return 0
        result = audit_differences(Rpc(), self.db, {1: candidate()}, 10)
        self.assertEqual(result["agrees_with_storage"], 1)
        self.assertIn(("balance", "", ADDRESS, 1), calls)
        with closing(sqlite3.connect(self.db)) as db:
            self.assertEqual(db.execute("SELECT COUNT(*) FROM differences").fetchone()[0], 1)
        self.assertEqual(audit_differences(Rpc(), self.db, {1: candidate()}, 0)["unchecked"], 1)

    def test_binary_formats_and_invalid_amounts(self):
        self.assertEqual(binary(base64.b64encode(bytes.fromhex(ADDRESS[2:])).decode(), 20), ADDRESS)
        for value in [-1, str(2**256), 1.5, True, "1.5"]:
            with self.assertRaises((ValueError, TypeError)):
                uint(value)


if __name__ == "__main__":
    unittest.main()
