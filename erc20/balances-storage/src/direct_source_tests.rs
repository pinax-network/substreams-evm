use super::*;
use prost::Message;
use serde_json::Value;

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/bsc-direct-source-layouts.json")).unwrap()
}

#[test]
fn thirteen_captured_direct_tokens_match_independent_historical_rpc_balances() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/direct-source/cases.json")).unwrap();
    assert_eq!(cases.len(), 13);
    let layouts = layouts();
    let mut contracts = BTreeSet::new();
    for case in cases {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        assert!(contracts.insert(contract.clone()));
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/direct-source")
            .join(case["fixture"].as_str().unwrap());
        let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
        assert_eq!(block.number, case["block"].as_u64().unwrap());
        assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let events = project(&block, std::slice::from_ref(layout)).unwrap();
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| (hex_bytes(row["address"].as_str().unwrap()).unwrap(), row["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        assert!(events.balances.iter().all(|b| b.contract.as_ref() == Some(&contract)));
        let actual = events.balances.into_iter().map(|b| (b.address, b.amount)).collect::<BTreeMap<_, _>>();
        assert_eq!(actual, expected, "contract {}", case["contract"]);
    }
}

#[test]
fn ain_six_minter_array_words_are_explicit_and_cannot_hide_adjacent_storage() {
    let layouts = layouts();
    let layout = layouts
        .iter()
        .find(|l| hex::encode(&l.contract) == "9558a9254890b2a8b057a789f413631b9084f4a3")
        .unwrap();
    let root = hash(&word(&[6]).unwrap());
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/direct-source/cases.json")).unwrap();
    let fixture = cases.iter().find(|c| c["rank"] == 38).unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/direct-source")
        .join(fixture["fixture"].as_str().unwrap());
    let mut block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    block.transaction_traces.clear();
    block.system_calls.clear();
    block.code_changes.clear();
    let mut key = root;
    for index in 0..7 {
        block.transaction_traces = vec![eth::TransactionTrace {
            status: 1,
            calls: vec![eth::Call {
                storage_changes: vec![eth::StorageChange {
                    address: layout.contract.clone(),
                    key: key.to_vec(),
                    old_value: vec![],
                    new_value: word(&[0xbb; 20]).unwrap().to_vec(),
                    ordinal: 10,
                }],
                ..Default::default()
            }],
            ..Default::default()
        }];
        let result = project(&block, std::slice::from_ref(layout));
        if index < 6 {
            assert!(result.unwrap().balances.is_empty());
        } else {
            assert!(result.unwrap_err().to_string().contains("unresolved storage"));
        }
        for byte in key.iter_mut().rev() {
            let (next, carry) = byte.overflowing_add(1);
            *byte = next;
            if !carry {
                break;
            }
        }
    }
}
