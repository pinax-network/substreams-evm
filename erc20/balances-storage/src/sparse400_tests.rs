use super::*;
use prost::Message;
use serde_json::Value;

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/sparse400/layouts.json")).unwrap()
}

#[test]
fn ar_arz_and_ars_captured_blocks_match_independent_rpc() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sparse400");
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/sparse400/cases.json")).unwrap();
    let layouts = layouts();
    assert_eq!(cases.len(), 3);
    let mut tokens = BTreeSet::new();
    let mut checked = 0;
    for case in cases {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let block = eth::Block::decode(std::fs::read(root.join(case["fixture"].as_str().unwrap())).unwrap().as_slice()).unwrap();
        assert_eq!(block.number, case["block"].as_u64().unwrap());
        assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        let events = project(&block, std::slice::from_ref(layout)).unwrap();
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&contract)));
        checked += expected.len();
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        assert!(tokens.insert(contract));
    }
    assert_eq!(tokens.len(), 3);
    assert_eq!(checked, 6);
}

#[test]
fn sparse_getter_controls_match_zero_dead_contract_and_maximum_addresses() {
    let controls: Value = serde_json::from_str(include_str!("../tests/fixtures/sparse400/controls.json")).unwrap();
    let layouts = layouts();
    let mut checked = 0;
    let mut emitted = 0;
    let mut excluded_null = 0;
    for token in controls["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        for control in token["raw_controls"].as_array().unwrap() {
            let holder = hex_bytes(control["address"].as_str().unwrap()).unwrap();
            let mut preimage = [0; 64];
            preimage[12..32].copy_from_slice(&holder);
            preimage[32..].copy_from_slice(&layout.balance_slot);
            let key = hash(&preimage);
            assert_eq!(key.to_vec(), hex_bytes(control["storage_key"].as_str().unwrap()).unwrap());
            let raw: BigInt = control["raw"].as_str().unwrap().parse().unwrap();
            let value = raw.to_bytes_be().1;
            let number = token["block"].as_u64().unwrap();
            let block = eth::Block {
                ver: 5,
                number,
                hash: hex_bytes(token["hash"].as_str().unwrap()).unwrap(),
                detail_level: eth::block::DetailLevel::DetaillevelExtended as i32,
                header: Some(eth::BlockHeader {
                    number,
                    parent_hash: vec![2; 32],
                    state_root: vec![3; 32],
                    ..Default::default()
                }),
                transaction_traces: vec![eth::TransactionTrace {
                    status: eth::TransactionTraceStatus::Succeeded as i32,
                    calls: vec![eth::Call {
                        address: contract.clone(),
                        storage_changes: vec![eth::StorageChange {
                            address: contract.clone(),
                            key: key.to_vec(),
                            old_value: if raw == BigInt::zero() { vec![1] } else { vec![] },
                            new_value: value,
                            ordinal: 1,
                        }],
                        keccak_preimages: std::collections::HashMap::from([(hex::encode(key), hex::encode(preimage))]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            };
            let raw_changes = changes(&block, std::slice::from_ref(layout)).unwrap();
            assert_eq!(raw_changes.len(), 1);
            assert_eq!(raw_changes[0].address, holder);
            assert_eq!(raw_changes[0].amount, control["rpc"].as_str().unwrap());
            let events = project(&block, std::slice::from_ref(layout)).unwrap();
            if holder.iter().all(|b| *b == 0) {
                // erc20/balances excludes the null address before making RPC calls,
                // even when the token's raw getter can return a nonzero value.
                assert!(events.balances.is_empty());
                excluded_null += 1;
            } else {
                assert_eq!(events.balances.len(), 1);
                assert_eq!(events.balances[0].contract.as_ref(), Some(&contract));
                assert_eq!(events.balances[0].address, holder);
                assert_eq!(events.balances[0].amount, control["rpc"].as_str().unwrap());
                emitted += 1;
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 56);
    assert_eq!(emitted, 44);
    assert_eq!(excluded_null, 12);
}
