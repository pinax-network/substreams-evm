ENDPOINT ?= mainnet.eth.streamingfast.io:443
START_BLOCK ?= 0
STOP_BLOCK ?= +1000
PARALLEL_JOBS ?= 500
.DEFAULT_GOAL := pack

.PHONY: pack
pack:
	substreams pack substreams.yaml -o ../../spkg/{spkgDefaultName}

.PHONY: noop
noop:
	substreams-sink-noop $(ENDPOINT) substreams.yaml db_out -H "X-Substreams-Parallel-Workers: $(PARALLEL_JOBS)" 0:

.PHONY: gui
gui:
	substreams gui -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) --limit-processed-blocks 0

.PHONY: prod
prod:
	substreams gui -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) -t $(STOP_BLOCK) --limit-processed-blocks 0 --production-mode -H "X-Substreams-Parallel-Workers: $(PARALLEL_JOBS)"
