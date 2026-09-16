ENDPOINT ?= bsc.substreams.pinax.network:443
START_BLOCK ?= 122260950
STOP_BLOCK ?= +64
SPKG := ../spkg/evm-balances-storage-v0.1.0.spkg
.DEFAULT_GOAL := pack
.PHONY: build pack test run protogen clickhouse
build:
	cargo build --locked -p evm-balances-storage --target wasm32-unknown-unknown --release
pack: build
	substreams pack substreams.yaml -o $(SPKG)
test:
	cargo test --locked -p evm-balances-storage
	python3 -m unittest discover -s scripts -p 'test_*.py'
run: pack
	substreams run $(SPKG) map_balances -e $(ENDPOINT) -s $(START_BLOCK) -t $(STOP_BLOCK) --final-blocks-only -o jsonl
protogen:
	substreams protogen substreams.yaml -o out/generated --exclude-paths sf/substreams,sf/ethereum,google
	cp out/generated/evm.balances.storage.v1.rs src/pb.rs
	cargo fmt -p evm-balances-storage
clickhouse: pack
	substreams pack clickhouse/substreams.yaml -o ../spkg/evm-clickhouse-balances-storage-v0.1.0.spkg
