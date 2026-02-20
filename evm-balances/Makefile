ENDPOINT ?= eth.substreams.pinax.network:443
START_BLOCK ?= 13000000
STOP_BLOCK ?= +1000
PARALLEL_JOBS ?= 500
.DEFAULT_GOAL := pack

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
	substreams gui -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) -t 0 --limit-processed-blocks 0 --production-mode  -H "X-Sf-Substreams-Parallel-Jobs: $(PARALLEL_JOBS)"
