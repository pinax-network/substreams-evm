ENDPOINT ?= mainnet.eth.streamingfast.io:443
START_BLOCK ?= 0
STOP_BLOCK ?= +1000
PARALLEL_JOBS ?= 500
.DEFAULT_GOAL := pack

.PHONY: schema
schema:
	@echo "-- This file is generated. Do not edit." > schema.sql
	@echo "" >> schema.sql
	@for f in $$(ls schema.*.sql 2>/dev/null | sort); do \
		cat "$$f" >> schema.sql; \
		echo "" >> schema.sql; \
	done

.PHONY: pack
pack: schema
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
