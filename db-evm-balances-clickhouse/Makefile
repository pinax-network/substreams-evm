ENDPOINT ?= eth.substreams.pinax.network:443
START_BLOCK ?= 24399522
STOP_BLOCK ?= 0
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
	substreams pack

.PHONY: dev
dev: pack
	substreams-sink-sql run clickhouse://default:@localhost:9000/default substreams.yaml -e $(ENDPOINT) $(START_BLOCK): --undo-buffer-size 0 --on-module-hash-mistmatch=warn --batch-block-flush-interval 1 --live-block-flush-interval 1 --infinite-retry --development-mode

.PHONY: dev-tron
dev-tron: pack
	substreams-sink-sql run clickhouse://default:@localhost:9000/default substreams.yaml -e tronevm.substreams.pinax.network:443 79000000: --undo-buffer-size 0 --on-module-hash-mistmatch=warn --batch-block-flush-interval 1 --live-block-flush-interval 1 --infinite-retry --development-mode -H "X-substreams-acknowledge-non-deterministic: true" -p "db_out=tron_base58"

.PHONY: setup
setup: pack
	substreams-sink-sql setup clickhouse://default:@localhost:9000/default substreams.yaml