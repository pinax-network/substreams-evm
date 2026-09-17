# BSC candidates ranked 301–350

The next 50 candidates have an exploratory survey and getter controls, with
**no new production profiles promoted**. The qualified fixture remains at 296.
Selection uses the original immutable RPC ranking for blocks
**122288006–122289029**; it does not rerank a newer interval.

The [investigation evidence](evidence/ranks301-350-investigation.json) records
every candidate, historical runtime, available source binding and survey
outcome. Across 299 sampled Extended blocks, these candidates account for 2,013
reference observations in the full interval. The survey performs **2,106
successful RPC value checks with zero value mismatches**. Forty-seven candidates
have a unique matching mapping hypothesis. None yet has complete mapper parity
or packaged-WASM holder qualification.

Getter inspections complete for 48 candidates. Their tested word controls show
neither a zero-word fallback nor a nonzero-word transformation. These results
are exploratory controls, not proof of complete getter behavior or permission
to ignore other storage writes. Ten historical runtime hashes match previously
reviewed families; token-specific dependencies and full-window checks are still
required.

## Exceptions retained

| Rank | Token | Result |
| --- | --- | --- |
| 319 | 金麟人生 (`0x5b0b5ed3b7d29813ac0c9c4890f5bf1fc4302b0e`) | One empty RPC result at the parent of its deployment block. A fresh recheck confirms empty parent code and nonempty code at block 122288321. It is an unavailable predeployment getter, never a zero or passing value check. |
| 324 | WMAI (`0x1b282603099647e2ec3461b2714acf38009d66b6`) | Slot 1 has eight matching training observations but only one nonzero holder, below the discovery requirement of two. Historical runtime is bound to verified source with a direct inherited balance getter at slot 1; that source alone does not qualify all observed storage writes. |
| 327 | Tracker (`0x76096218ea7e6e1a3c6a549c88e2928c633b5f63`) | Both slots 0 and 12 match six training observations across five nonzero holders. Source is unavailable. Separate follow-up controls identify slot 0 as the observed getter input. |

The [getter follow-up](evidence/ranks301-350-getter-followup.json) resolves these
two sampled mapping gaps without relaxing discovery thresholds. WMAI's getter
reads slot 1, consistent with its verified source. Tracker's getter reads slot 0;
overriding it with 0, 1, 123 and uint256 max returns exactly those values.
Overriding the corresponding slot-12 word with the same four values leaves the
historical balance unchanged. The slot-12 hypothesis is a coincident value,
not evidence of a second balance getter. Each trace reads one word and makes no
external call. These three inspections and twelve controls remain separate from
the original survey totals. Full storage-rule review is still required.

The original survey's `rpc_unresolved` status remains unchanged. The separate
[recheck evidence](evidence/ranks301-350-recheck.json) records the predeployment
classification, original response, canonical parent/current hashes and digests.
No RPC value mismatch was discarded or retried into a passing sample.

Next steps are to review source or historical bytecode, configure only justified
storage rules, scan all 1,024 consecutive blocks and compare actual packaged
WASM with historical RPC and initialized holder state. Cold unknown balances,
unchanged holders and global enumeration remain separately measured.
