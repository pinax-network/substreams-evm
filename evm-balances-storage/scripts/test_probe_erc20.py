import unittest

from probe_erc20 import classify_layout


class DiscoveryGates(unittest.TestCase):
    def test_zero_only_and_single_holder_agreement_is_insufficient(self):
        stats = {"code_changed": False, "rpc_errors": 0, "mismatches": 0,
                 "nonzero_holders": 0, "changed_observations": 10}
        self.assertEqual(classify_layout(stats), "insufficient_evidence")
        stats["nonzero_holders"] = 1
        self.assertEqual(classify_layout(stats), "insufficient_evidence")
        stats["nonzero_holders"] = 2
        self.assertEqual(classify_layout(stats), "candidate_matches_rpc_not_qualified")

    def test_code_changes_errors_and_disagreement_prevent_matching_label(self):
        stats = {"code_changed": False, "rpc_errors": 0, "mismatches": 0,
                 "nonzero_holders": 10, "changed_observations": 100}
        for field, status in [("code_changed", "code_change_requires_review"),
                              ("rpc_errors", "rpc_unresolved"), ("mismatches", "not_direct_balance_mapping")]:
            with self.subTest(field=field):
                self.assertEqual(classify_layout(dict(stats, **{field: 1})), status)


if __name__ == "__main__":
    unittest.main()
