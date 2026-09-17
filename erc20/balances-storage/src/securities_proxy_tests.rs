use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/securities-proxy/cases.json")).unwrap()
}

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/bsc-securities-proxy-layouts.json")).unwrap()
}

fn captured(case: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/securities-proxy")
        .join(case["fixture"].as_str().unwrap());
    eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap()
}

#[test]
fn three_captured_proxy_tokens_match_independent_historical_rpc_balances() {
    let cases = cases();
    assert_eq!(cases.len(), 3);
    let layouts = layouts();
    let mut contracts = BTreeSet::new();
    for case in cases {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        assert!(contracts.insert(contract.clone()));
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let block = captured(&case);
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

fn write(address: &[u8], key: [u8; 32], old: &[u8], new: &[u8], ordinal: u64) -> eth::StorageChange {
    eth::StorageChange {
        address: address.to_vec(),
        key: key.to_vec(),
        old_value: old.to_vec(),
        new_value: new.to_vec(),
        ordinal,
    }
}

#[test]
fn securities_ui_schedule_preserves_raw_balances_but_beacon_slot_one_is_guarded() {
    let layouts = layouts();
    for case in cases().iter().filter(|c| c["contract"] != "0xce24439f2d9c6a2289f741120fe202248b666666") {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let mut block = captured(case);
        let expected = project(&block, std::slice::from_ref(layout)).unwrap();
        let changes = (0..3)
            .map(|slot| write(&contract, word(&[slot]).unwrap(), &[], &[2], 1_000_000 + slot as u64))
            .collect();
        block.transaction_traces.push(eth::TransactionTrace {
            status: 1,
            calls: vec![eth::Call {
                storage_changes: changes,
                ..Default::default()
            }],
            ..Default::default()
        });
        assert_eq!(project(&block, std::slice::from_ref(layout)).unwrap(), expected);

        // Slot 1 belongs to the UI schedule on the token and to the implementation
        // pointer on the beacon. Its address determines which rule applies.
        let beacon = layout.beacon_proxy.as_ref().unwrap();
        block.transaction_traces.last_mut().unwrap().calls[0].storage_changes = vec![write(
            &beacon.beacon,
            beacon.implementation_slot,
            &beacon.implementation,
            &[0xbb; 20],
            1_000_010,
        )];
        let error = project(&block, std::slice::from_ref(layout)).unwrap_err();
        assert!(error.to_string().contains("beacon implementation changed"));
    }
}

#[test]
fn securities_unreviewed_role_array_and_stablecoin_admin_writes_stop_processing() {
    let layouts = layouts();
    for case in cases() {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let mut block = captured(&case);
        let mut preimages = std::collections::HashMap::new();
        let key = if layout.beacon_proxy.is_some() {
            // AccessControlEnumerable: first DEFAULT_ADMIN_ROLE member array word.
            let root = word(&hex_bytes("c1f6fe24621ce81ec5827caf0253cadb74709b061630e6b55e82371705932000").unwrap()).unwrap();
            let mut preimage = [0; 64];
            preimage[32..].copy_from_slice(&root);
            let array_base = hash(&preimage);
            let element = hash(&array_base);
            preimages.insert(hex::encode(array_base), hex::encode(preimage));
            preimages.insert(hex::encode(element), hex::encode(array_base));
            element
        } else {
            word(&hex_bytes("b53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103").unwrap()).unwrap()
        };
        block.transaction_traces.push(eth::TransactionTrace {
            status: 1,
            calls: vec![eth::Call {
                storage_changes: vec![write(&contract, key, &[], &[0xbb; 20], 1_000_000)],
                keccak_preimages: preimages,
                ..Default::default()
            }],
            ..Default::default()
        });
        assert!(project(&block, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));
    }
}
