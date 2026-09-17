use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/next-proxies/cases.json")).unwrap()
}
fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/bsc-next-proxy-layouts.json")).unwrap()
}
fn captured(case: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/next-proxies")
        .join(case["fixture"].as_str().unwrap());
    eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap()
}

#[test]
fn eleven_more_proxy_tokens_match_independent_historical_rpc() {
    let layouts = layouts();
    assert_eq!(layouts.len(), 87);
    let cases = cases();
    assert_eq!(cases.len(), 11);
    let mut checked = 0;
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
            .map(|b| (hex_bytes(b["address"].as_str().unwrap()).unwrap(), b["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|b| b.contract.as_ref() == Some(&contract)));
        checked += expected.len();
        let actual = events.balances.into_iter().map(|b| (b.address, b.amount)).collect::<BTreeMap<_, _>>();
        assert_eq!(actual, expected, "rank {}", case["rank"]);
    }
    assert_eq!(checked, 64);
}

#[test]
fn both_new_clone_deployments_preserve_the_complete_initial_supply() {
    let layouts = layouts();
    let cases = cases();
    let supplies: Vec<Value> = serde_json::from_str(include_str!("../docs/evidence/next-proxy-initial-supply.json")).unwrap();
    for (rank, expected_holders) in [(59, 7), (65, 23)] {
        let case = cases.iter().find(|c| c["rank"] == rank).unwrap();
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let b = captured(case);
        assert_eq!(l.deployment.as_ref().unwrap().block, b.number);
        let rows = changes(&b, std::slice::from_ref(l)).unwrap();
        assert_eq!(rows.len(), expected_holders);
        assert!(rows.iter().all(|r| r.old_amount == "0"));
        let supply = rows.iter().fold(BigInt::from(0), |sum, r| sum + r.amount.parse::<BigInt>().unwrap());
        let expected = supplies.iter().find(|s| s["rank"] == rank).unwrap();
        assert_eq!(expected["hash"], case["hash"]);
        // FlapTaxTokenV3 mints a fixed billion tokens. TokenV2 accepts an
        // initialization supply; this clone uses 100 million, unlike its
        // previously reviewed billion-token sibling.
        assert_eq!(supply.to_string(), expected["total_supply_rpc"].as_str().unwrap());
        assert_eq!(expected["total_supply_rpc"], expected["total_supply_storage"]);
        assert_eq!(expected["total_supply_rpc"], expected["max_supply_rpc"]);
        let mut unqualified = l.clone();
        unqualified.deployment = None;
        assert!(project(&b, &[unqualified]).unwrap_err().to_string().contains("code changed"));
    }
}

#[test]
fn predeployment_rpc_responses_remain_unavailable_instead_of_becoming_zero() {
    let evidence: Vec<Value> = serde_json::from_str(include_str!("../docs/evidence/next-proxy-predeployment-rpc.json")).unwrap();
    assert_eq!(evidence.len(), 30);
    let layouts = layouts();
    let cases = cases();
    for entry in evidence {
        assert_eq!(entry["classification"], "unavailable_before_verified_first_CREATE");
        let original = &entry["original"];
        assert_eq!(original["boundary"], "before");
        assert_eq!(original["rpc_response"]["result"], "0x");
        assert!(original["rpc"].is_null());
        let case = cases.iter().find(|c| c["contract"] == original["contract"]).unwrap();
        let b = captured(case);
        assert_eq!(b.number, original["block"].as_u64().unwrap());
        assert_eq!(b.header.unwrap().parent_hash, hex_bytes(original["hash"].as_str().unwrap()).unwrap());
        let contract = hex_bytes(original["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        assert_eq!(l.deployment.as_ref().unwrap().block, b.number);
    }
}

fn add_word(mut value: [u8; 32], words: u8) -> [u8; 32] {
    let mut carry = u16::from(words);
    for byte in value.iter_mut().rev() {
        carry += u16::from(*byte);
        *byte = carry as u8;
        carry >>= 8;
    }
    value
}

#[test]
fn clone_metadata_payloads_do_not_permit_adjacent_unknown_storage() {
    let layouts = layouts();
    let cases = cases();
    for (rank, slot, words) in [(59, 264_u16, 2_u8), (65, 255, 2), (77, 264, 3)] {
        let case = cases.iter().find(|c| c["rank"] == rank).unwrap();
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let base = hash(&word(&slot.to_be_bytes()).unwrap());
        for i in 0..words {
            assert!(l.other_slots.contains(&add_word(base, i)));
        }
        let outside = add_word(base, words);
        assert!(!l.other_slots.contains(&outside));
        let mut b = captured(case);
        b.transaction_traces.push(eth::TransactionTrace {
            index: u32::MAX,
            status: 1,
            calls: vec![eth::Call {
                address: contract.clone(),
                storage_changes: vec![eth::StorageChange {
                    address: contract,
                    key: outside.to_vec(),
                    old_value: vec![],
                    new_value: vec![1],
                    ordinal: u64::MAX - 1,
                }],
                ..Default::default()
            }],
            ..Default::default()
        });
        assert!(project(&b, std::slice::from_ref(l)).unwrap_err().to_string().contains("unresolved storage"));
    }
}
