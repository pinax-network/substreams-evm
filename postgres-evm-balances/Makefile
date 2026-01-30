ENDPOINT ?= eth.substreams.pinax.network:443
START_BLOCK ?= 23145742
STOP_BLOCK ?= 23145745
PARALLEL_JOBS ?= 500

# PostgreSQL connection string
# Format: psql://<user>:<password>@<host>:<port>/<database>?sslmode=disable
PG_DSN ?= psql://dev-node:insecure-change-me-in-prod@localhost:5432/dev-node?sslmode=disable

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: pack
pack: build
	substreams pack

.PHONY: noop
noop: build
	substreams-sink-noop $(ENDPOINT) substreams.yaml db_out -H "X-Sf-Substreams-Parallel-Jobs: $(PARALLEL_JOBS)" $(START_BLOCK):

.PHONY: gui
gui: build
	substreams gui -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) -t $(STOP_BLOCK)

.PHONY: run
run: build
	substreams run -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) -t $(STOP_BLOCK) --production-mode

.PHONY: prod
prod: build
	substreams gui -e $(ENDPOINT) substreams.yaml db_out -s $(START_BLOCK) -t 0 --limit-processed-blocks 0 --production-mode  -H "X-Sf-Substreams-Parallel-Jobs: $(PARALLEL_JOBS)"

.PHONY: dev
dev: build
	substreams-sink-sql run $(PG_DSN) substreams.yaml -e $(ENDPOINT) $(START_BLOCK):$(STOP_BLOCK) --undo-buffer-size 1 --on-module-hash-mistmatch=warn --batch-block-flush-interval 1 --infinite-retry --development-mode

.PHONY: dev-tron
dev-tron: build
	substreams-sink-sql run $(PG_DSN) substreams.yaml -e tronevm.substreams.pinax.network:443 79000000: --undo-buffer-size 1 --on-module-hash-mistmatch=warn --batch-block-flush-interval 1 --infinite-retry --development-mode -H "X-substreams-acknowledge-non-deterministic: true" -p "db_out=tron_base58"

.PHONY: setup
setup: pack
	substreams-sink-sql setup $(PG_DSN) substreams.yaml
