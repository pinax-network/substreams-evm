ENDPOINT ?= eth.substreams.pinax.network:443
START_BLOCK ?= 20000000
STOP_BLOCK ?= 20500000
PARALLEL_JOBS ?= 1000

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: pack
pack: build
	substreams pack

.PHONY: noop
noop: build
	substreams-sink-noop $(ENDPOINT) substreams.yaml db_out -H "X-Sf-Substreams-Parallel-Jobs: $(PARALLEL_JOBS)" $(START_BLOCK):$(STOP_BLOCK)

.PHONY: gui
gui: build
	substreams gui -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) --network mainnet

.PHONY: prod
prod: build
	substreams gui -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) -t $(STOP_BLOCK) --limit-processed-blocks 0 --production-mode  -H "X-Sf-Substreams-Parallel-Jobs: $(PARALLEL_JOBS)"
