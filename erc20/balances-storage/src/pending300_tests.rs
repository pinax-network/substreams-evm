use super::*;
use prost::Message;
use serde_json::Value;

#[test]
fn pending300_qualified_profiles_match_captured_independent_rpc() {
    let layouts = layout::parse(include_str!("../tests/fixtures/pending300/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 17);
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/pending300/cases.json")).unwrap();
    assert_eq!(cases.len(), 17);
    let mut tokens = BTreeSet::new();
    let mut checked = 0;
    for case in cases {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == token).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pending300")
            .join(case["fixture"].as_str().unwrap());
        let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
        assert_eq!(block.number, case["block"].as_u64().unwrap());
        assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        checked += expected.len();
        let events = project(&block, std::slice::from_ref(layout)).unwrap();
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&token)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        assert!(tokens.insert(token));
    }
    assert_eq!(tokens.len(), 17);
    assert_eq!(checked, 45);
}

fn list_fixture(name: &str) -> (eth::Block, Vec<VerifiedLayout>, Value) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    let block = eth::Block::decode(std::fs::read(dir.join("block.pb")).unwrap().as_slice()).unwrap();
    let layouts = layout::parse(&std::fs::read_to_string(dir.join("layouts.json")).unwrap()).unwrap();
    let case: Value = serde_json::from_slice(&std::fs::read(dir.join("case.json")).unwrap()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    assert_eq!(
        block.header.as_ref().unwrap().parent_hash,
        hex_bytes(case["parent_hash"].as_str().unwrap()).unwrap()
    );
    (block, layouts, case)
}

#[test]
fn captured_dia_append_and_etz_swap_removal_match_rpc() {
    for (name, expected_writes) in [("dia-holder-append", 2), ("etz-swap-removal", 3)] {
        let (block, layouts, case) = list_fixture(name);
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        let events = project(&block, &layouts).unwrap();
        assert_eq!(events.balances.len(), 2);
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&layouts[0].contract)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        let writes = case["writes"].as_array().unwrap();
        assert_eq!(writes.len(), expected_writes);
        for witness in writes {
            let tx = block
                .transaction_traces
                .iter()
                .find(|t| hex::encode(&t.hash) == witness["transaction"])
                .unwrap();
            let call = tx.calls.iter().find(|c| u64::from(c.index) == witness["call"].as_u64().unwrap()).unwrap();
            let row = call.storage_changes.iter().find(|s| s.ordinal == witness["ordinal"].as_u64().unwrap()).unwrap();
            assert_eq!(row.address, layouts[0].contract);
            assert_eq!(hex::encode(&row.key), witness["key"]);
            assert_eq!(hex::encode(&row.old_value), witness["old"]);
            assert_eq!(hex::encode(&row.new_value), witness["new"]);
        }
        let mut missing = layouts;
        missing[0].address_lists.clear();
        assert!(project(&block, &missing).is_err());
    }
}

fn change(block: &mut eth::Block, ordinal: u64) -> &mut eth::StorageChange {
    block
        .transaction_traces
        .iter_mut()
        .flat_map(|t| &mut t.calls)
        .flat_map(|c| &mut c.storage_changes)
        .find(|s| s.ordinal == ordinal)
        .unwrap()
}

#[test]
fn captured_etz_swap_rejects_inconsistent_or_missing_witnesses() {
    for variant in 0..7 {
        let (mut block, layouts, case) = list_fixture("etz-swap-removal");
        let writes = case["writes"].as_array().unwrap();
        let copy = writes[0]["ordinal"].as_u64().unwrap();
        let tail = writes[1]["ordinal"].as_u64().unwrap();
        let length = writes[2]["ordinal"].as_u64().unwrap();
        match variant {
            0 => change(&mut block, copy).new_value = vec![1; 20],
            1 => change(&mut block, tail).old_value = vec![2; 20],
            2 => change(&mut block, tail).new_value = vec![3; 20],
            3 => change(&mut block, copy).ordinal = length + 1,
            4 => change(&mut block, length).new_value = vec![75],
            5 | 6 => {
                let removed = if variant == 5 { tail } else { length };
                for call in block.transaction_traces.iter_mut().flat_map(|t| &mut t.calls) {
                    call.storage_changes.retain(|s| s.ordinal != removed);
                }
            }
            _ => unreachable!(),
        }
        assert!(project(&block, &layouts).is_err(), "variant {variant}");
    }
}

#[test]
fn captured_dia_append_rejects_inconsistent_or_missing_witnesses() {
    for variant in 0..5 {
        let (mut block, layouts, case) = list_fixture("dia-holder-append");
        let writes = case["writes"].as_array().unwrap();
        let length = writes.iter().find(|w| w["index"] == "length").unwrap()["ordinal"].as_u64().unwrap();
        let element = writes.iter().find(|w| w["index"] == "228").unwrap()["ordinal"].as_u64().unwrap();
        match variant {
            0 => change(&mut block, element).old_value = vec![1],
            1 => change(&mut block, element).new_value = vec![1; 21],
            2 => change(&mut block, element).ordinal = length - 1,
            3 | 4 => {
                let removed = if variant == 3 { length } else { element };
                for call in block.transaction_traces.iter_mut().flat_map(|t| &mut t.calls) {
                    call.storage_changes.retain(|s| s.ordinal != removed);
                }
            }
            _ => unreachable!(),
        }
        assert!(project(&block, &layouts).is_err(), "variant {variant}");
    }
}
