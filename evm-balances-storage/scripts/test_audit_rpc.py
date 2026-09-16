import unittest

from audit_rpc import audit_block
from compare import WBNB, balance_result, batch_results
from test_compare import HASH, candidate


class RpcAuditTests(unittest.TestCase):
    def test_batches_match_by_id_not_response_order(self):
        self.assertEqual(batch_results([{"id": 1, "result": "0xb"}, {"id": 0, "result": "0xa"}], 2), ["0xa", "0xb"])

    def test_missing_duplicate_error_null_and_unexpected_id_fail(self):
        for data in [[], [{"id": 0, "result": "0x0"}]*2,
                     [{"id": 0, "error": {"code": -1}}], [{"id": 0, "result": None}],
                     [{"id": 2, "result": "0x0"}], [{"id": True, "result": "0x0"}]]:
            with self.subTest(data=data), self.assertRaises(ValueError):
                batch_results(data, 2 if len(data) == 2 else 1)

    def test_token_result_requires_full_abi_word(self):
        self.assertEqual(balance_result("0x"+"ff"*32, True), 2**256-1)
        for value in ("0x", "0x0", "0x"+"00"*64, "bad"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                balance_result(value, True)

    def test_every_balance_uses_exact_parent_and_current_hash(self):
        requests = []
        class Rpc:
            def header(self, number):
                return {"hash": HASH}
            def batch(self, calls):
                requests.extend(calls)
                return ["0x"+"00"*31+"05", "0x"+"00"*32]
        block = candidate(contract=WBNB)
        checks = audit_block(Rpc(), block, 50)
        self.assertEqual(len(checks), 2)
        self.assertTrue(all(c["match"] for c in checks))
        self.assertEqual([c["boundary"] for c in checks], ["before", "after"])
        for request, digest in zip(requests, (block["parentHash"], HASH)):
            self.assertEqual(request[0], "eth_call")
            self.assertEqual(request[1][-1], {"blockHash": digest, "requireCanonical": True})

    def test_a_disagreement_is_retained(self):
        class Rpc:
            def header(self, number): return {"hash": HASH}
            def batch(self, calls): return ["0x5", "0x1"]
        checks = audit_block(Rpc(), candidate(), 50)
        self.assertTrue(checks[0]["match"])
        self.assertFalse(checks[1]["match"])
        self.assertEqual(checks[1]["rpc"], "1")

    def test_wrong_hash_and_unresolved_slots_fail_before_rpc_balances(self):
        class Rpc:
            def header(self, number): return {"hash": HASH}
            def batch(self, calls): raise AssertionError("must not request balances")
        block = candidate()
        block["unresolvedWbnbSlots"] = [{"key": "0x01"}]
        with self.assertRaisesRegex(ValueError, "unresolved"):
            audit_block(Rpc(), block, 50)
        block = candidate()
        block["hash"] = "0x"+"ff"*32
        with self.assertRaisesRegex(ValueError, "identity"):
            audit_block(Rpc(), block, 50)


if __name__ == "__main__":
    unittest.main()
