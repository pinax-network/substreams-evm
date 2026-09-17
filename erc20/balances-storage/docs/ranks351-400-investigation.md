# BSC candidates ranked 351–400

The [investigation report](evidence/ranks351-400-investigation.json) selects the
next 50 candidates from the original immutable RPC stream for blocks
122288006–122289029. They account for **1,685 reference observations** across
the complete interval. The exploratory survey uses **269 sampled Extended
blocks** and records **1,654 successful RPC value comparisons**, with no RPC
errors or historical value mismatches. It preserves mapper errors and missing
rows; matching candidate values are not equivalent to mapper qualification.

Forty-four candidates have a unique matching mapping hypothesis. The
[getter controls](evidence/ranks351-400-getters.json) inspect those 44 and find
one nonzero-word transformation: XVS's `uint96` getter. Its separately verified
[width correction and holder audit](xvs-uint96.md) add one qualified profile.
The other 49 candidates remain outside the latest combined configuration.

Eleven historical runtime hashes match an already reviewed family. Each still
needs its own source/getter/dependency binding, explicit storage rules and
complete interval checks; matching a shell or a sampled value does not promote
a profile.

The six candidates with no direct-mapping hypothesis are AR (351), ARZ (354),
ARS (364), 10SET (377), MUSD (386) and GAIX (394). That status is a discovery
gap, not a claimed zero balance or a proven computed getter. MUSD and GAIX have
source-bound direct getters and need independent follow-up without relaxing
discovery thresholds. 10SET's verified source uses reflection accounting.
Baby Doge (372) also has a reflection getter; a sample on its excluded-account
branch is not proof of a direct mapping for all holders.

All source/runtime and diagnostic evidence remains separate from production
qualification. Cold unknowns, emitted rows, initialized retained holders and
global enumeration retain their distinct scopes. Ethereum, Base, HyperEVM and
Arc still require their own qualification after the BSC work.
