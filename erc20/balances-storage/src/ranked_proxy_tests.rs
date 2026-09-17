use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/ranked-proxies/cases.json")).unwrap()
}
fn captured(case: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ranked-proxies")
        .join(case["fixture"].as_str().unwrap());
    eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap()
}
#[test]
fn seven_ranked_proxies_match_captured_canonical_rpc() {
    let layouts = layout::parse(include_str!("../tests/fixtures/bsc-ranked-proxy-layouts.json")).unwrap();
    assert_eq!(layouts.len(), 140);
    let cases = cases();
    assert_eq!(cases.len(), 7);
    let mut seen = BTreeSet::new();
    let mut proxied_beacons = 0;
    for case in cases {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        assert!(seen.insert(contract.clone()));
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        proxied_beacons += usize::from(l.beacon_proxy.as_ref().is_some_and(|b| b.proxy.is_some()));
        let b = captured(&case);
        assert_eq!(b.number, case["block"].as_u64().unwrap());
        assert_eq!(b.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        let events = project(&b, std::slice::from_ref(l)).unwrap();
        assert!(!events.balances.is_empty());
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&contract)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
    }
    assert_eq!(proxied_beacons, 2);
}
#[test]
fn captured_bridge_balances_do_not_hide_a_beacon_delegate_upgrade() {
    let layouts = layout::parse(include_str!("../tests/fixtures/bsc-ranked-proxy-layouts.json")).unwrap();
    let cases = cases();
    let mut checked = 0;
    for l in layouts.iter().filter(|l| l.beacon_proxy.as_ref().is_some_and(|b| b.proxy.is_some())) {
        let case = cases.iter().find(|c| c["contract"] == format!("0x{}", hex::encode(&l.contract))).unwrap();
        let mut b = captured(case);
        assert!(project(&b, std::slice::from_ref(l)).is_ok());
        let beacon = l.beacon_proxy.as_ref().unwrap();
        let delegate = beacon.proxy.as_ref().unwrap();
        b.system_calls.push(eth::Call {
            address: beacon.beacon.clone(),
            storage_changes: vec![eth::StorageChange {
                address: beacon.beacon.clone(),
                key: delegate.implementation_slot.to_vec(),
                old_value: delegate.implementation.clone(),
                new_value: vec![0xac; 20],
                ordinal: u64::MAX - 1,
            }],
            ..Default::default()
        });
        assert!(project(&b, std::slice::from_ref(l))
            .unwrap_err()
            .to_string()
            .contains("beacon implementation changed"));
        let mut incomplete = l.clone();
        incomplete.beacon_proxy.as_mut().unwrap().proxy = None;
        assert!(project(&b, &[incomplete]).is_ok(), "missing proxy qualification hides this dependency change");
        checked += 1;
    }
    assert_eq!(checked, 2);
}
