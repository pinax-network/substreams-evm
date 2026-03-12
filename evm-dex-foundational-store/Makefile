ENDPOINT ?= eth.substreams.pinax.network:443
START_BLOCK ?= 20000000
STOP_BLOCK ?= 20500000
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
	substreams-sink-noop $(ENDPOINT) substreams.yaml map_entries -H "X-Substreams-Parallel-Workers: $(PARALLEL_JOBS)" 0:

.PHONY: gui
gui: build
	substreams gui -e $(ENDPOINT) substreams.yaml map_entries -s $(START_BLOCK) --limit-processed-blocks 0

.PHONY: prod
prod: build
	substreams gui -e $(ENDPOINT) substreams.yaml map_entries -s $(START_BLOCK) -t $(STOP_BLOCK) --limit-processed-blocks 0 --production-mode  -H "X-Substreams-Parallel-Workers: $(PARALLEL_JOBS)"
