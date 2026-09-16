ENDPOINT ?= bsc.substreams.pinax.network:443
START_BLOCK ?= 122260950
STOP_BLOCK ?= +64
SPKG := ../../spkg/erc20-balances-storage-v0.1.0.spkg
.DEFAULT_GOAL := pack
.PHONY: build pack test run protogen
build:
	cargo build --locked -p erc20-balances-storage --target wasm32-unknown-unknown --release
pack: build
	substreams pack substreams.yaml -o $(SPKG)
test:
	cargo test --locked -p erc20-balances-storage -p erc20-balances-storage-tools
run: pack
	substreams run $(SPKG) map_events -e $(ENDPOINT) -s $(START_BLOCK) -t $(STOP_BLOCK) --final-blocks-only -o jsonl
protogen:
	substreams protogen substreams.yaml -o out/generated --exclude-paths sf/substreams,sf/ethereum,google
	cp out/generated/evm.balances.storage.v1.rs src/pb.rs
	cargo fmt -p erc20-balances-storage
