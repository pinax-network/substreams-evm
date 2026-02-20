ENDPOINT ?= eth.substreams.pinax.network:443
START_BLOCK ?= 23018672
STOP_BLOCK ?= +100
PARALLEL_JOBS ?= 500
.DEFAULT_GOAL := pack

.PHONY: pack
pack:
	substreams pack

.PHONY: dev
dev: pack
	substreams-sink-sql run clickhouse://default:@localhost:9000/default substreams.yaml -e $(ENDPOINT) $(START_BLOCK):$(STOP_BLOCK) --undo-buffer-size 0 --on-module-hash-mistmatch=warn --batch-block-flush-interval 1 --live-block-flush-interval 1 --infinite-retry --development-mode

.PHONY: setup
setup: pack
	substreams-sink-sql setup clickhouse://default:@localhost:9000/default substreams.yaml
