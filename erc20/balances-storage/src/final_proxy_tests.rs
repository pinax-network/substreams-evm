use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/final-proxies/cases.json")).unwrap()
}
fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/bsc-final-proxy-layouts.json")).unwrap()
}
fn captured(c: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/final-proxies")
        .join(c["fixture"].as_str().unwrap());
    eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap()
}

#[test]
fn final_two_proxy_candidates_match_captured_historical_rpc() {
    let layouts = layouts();
    assert_eq!(layouts.len(), 99);
    let cases = cases();
    assert_eq!(cases.len(), 4);
    let mut checked = 0;
    let mut contracts = BTreeSet::new();
    for c in cases {
        let contract = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
        contracts.insert(contract.clone());
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let b = captured(&c);
        assert_eq!(b.number, c["block"].as_u64().unwrap());
        assert_eq!(b.hash, hex_bytes(c["hash"].as_str().unwrap()).unwrap());
        let events = project(&b, std::slice::from_ref(l)).unwrap();
        let expected = c["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (hex_bytes(v["address"].as_str().unwrap()).unwrap(), v["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        checked += expected.len();
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&contract)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
    }
    assert_eq!(contracts.len(), 2);
    assert_eq!(checked, 11);
}

#[test]
fn reward_bookkeeping_requires_each_explicitly_reviewed_field() {
    let layouts = layouts();
    let cases = cases();
    for (height, scalar, width) in [
        (122288006, Some(25), None),
        (122288006, Some(37), None),
        (122288164, Some(40), None),
        (122288018, Some(47), None),
        (122288018, None, Some(3)),
    ] {
        let c = cases.iter().find(|c| c["rank"] == 56 && c["block"] == height).unwrap();
        let contract = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let b = captured(c);
        assert!(project(&b, std::slice::from_ref(l)).is_ok());
        let mut unsupported = l.clone();
        if let Some(slot) = scalar {
            assert!(unsupported.other_slots.remove(&word(&[slot]).unwrap()));
        }
        if let Some(width) = width {
            assert_eq!(unsupported.other_mapping_words.insert(word(&[36]).unwrap(), width), Some(4));
        }
        assert!(project(&b, &[unsupported]).unwrap_err().to_string().contains("unresolved storage"));
    }
}

#[test]
fn reward_payments_do_not_become_balance_updates_for_the_distributing_token() {
    let layouts = layouts();
    let cases = cases();
    let c = cases.iter().find(|c| c["rank"] == 56 && c["block"] == 122288018).unwrap();
    let token = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
    let reward = hex_bytes("0x7c8d5502b544ddaf8852fc46d1174e34876d545c").unwrap();
    let mut b = captured(c);
    b.transaction_traces
        .retain(|tx| hex::encode(&tx.hash) == "c0103c132b3f4a944ec9e171e5506ea8fac4bbafc91a64b98f3b5823255aa2a3");
    assert_eq!(b.transaction_traces.len(), 1);
    assert!(b.transaction_traces[0]
        .calls
        .iter()
        .flat_map(|c| &c.storage_changes)
        .any(|s| s.address == token));
    let events = project(&b, &layouts).unwrap();
    assert!(events.balances.iter().all(|r| r.contract.as_ref() != Some(&token)));
    assert!(events.balances.iter().any(|r| r.contract.as_ref() == Some(&reward)));
}

#[test]
fn reward_mapping_review_does_not_allow_the_adjacent_membership_field() {
    let layouts = layouts();
    let cases = cases();
    let c = cases.iter().find(|c| c["rank"] == 56 && c["block"] == 122288018).unwrap();
    let contract = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
    let l = layouts.iter().find(|l| l.contract == contract).unwrap();
    let mut b = captured(c);
    let call = b
        .transaction_traces
        .iter_mut()
        .flat_map(|tx| &mut tx.calls)
        .find(|c| {
            c.storage_changes.iter().any(|s| s.address == contract)
                && c.keccak_preimages.values().any(|p| {
                    let bytes = hex_bytes(p).unwrap();
                    bytes.len() == 64 && bytes[32..] == word(&[36]).unwrap()
                })
        })
        .unwrap();
    let preimage = call
        .keccak_preimages
        .values()
        .find(|p| {
            let bytes = hex_bytes(p).unwrap();
            bytes.len() == 64 && bytes[32..] == word(&[36]).unwrap()
        })
        .unwrap();
    let mut key = hash(&hex_bytes(preimage).unwrap());
    let mut carry = 4_u16;
    for byte in key.iter_mut().rev() {
        carry += u16::from(*byte);
        *byte = carry as u8;
        carry >>= 8;
    }
    call.storage_changes.push(eth::StorageChange {
        address: contract,
        key: key.to_vec(),
        old_value: vec![],
        new_value: vec![1],
        ordinal: u64::MAX - 1,
    });
    assert!(project(&b, std::slice::from_ref(l)).unwrap_err().to_string().contains("unresolved storage"));
}
