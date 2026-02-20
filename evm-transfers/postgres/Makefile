ENDPOINT ?= eth.substreams.pinax.network:443
START_BLOCK ?= 24226215
STOP_BLOCK ?= +100
PARALLEL_JOBS ?= 500
.DEFAULT_GOAL := pack

# PostgreSQL connection string
# Format: psql://<user>:<password>@<host>:<port>/<database>?sslmode=disable
PG_DSN ?= psql://dev-node:insecure-change-me-in-prod@localhost:5432/dev-node?sslmode=disable

.PHONY: pack
pack:
	substreams pack

.PHONY: dev
dev: pack
	substreams-sink-sql run $(PG_DSN) substreams.yaml -e $(ENDPOINT) $(START_BLOCK):$(STOP_BLOCK) --undo-buffer-size 0 --on-module-hash-mistmatch=warn --batch-block-flush-interval 1 --live-block-flush-interval 1 --infinite-retry --development-mode

.PHONY: setup
setup: pack
	substreams-sink-sql setup $(PG_DSN) substreams.yaml
