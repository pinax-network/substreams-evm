use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/voting/cases.json")).unwrap()
}
fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/bsc-voting-layouts.json")).unwrap()
}
fn captured(c: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/voting")
        .join(c["fixture"].as_str().unwrap());
    eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap()
}

#[test]
fn captured_voting_tokens_match_historical_rpc_through_deployment_mint_and_transfer() {
    let layouts = layouts();
    assert_eq!(layouts.len(), 89);
    let cases = cases();
    assert_eq!(cases.len(), 7);
    let mut checked = 0;
    for c in cases {
        let contract = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
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
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&contract)));
        checked += expected.len();
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
    }
    assert_eq!(checked, 7);
}

#[test]
fn star_holder_state_survives_empty_outputs_before_and_after_the_mint() {
    let layouts = layouts();
    let cases = cases();
    let before = cases.iter().find(|c| c["block"] == 58597371).unwrap();
    let contract = hex_bytes(before["contract"].as_str().unwrap()).unwrap();
    let l = layouts.iter().find(|l| l.contract == contract).unwrap();
    // This baseline is a captured RPC read after the contract's deployment,
    // before the mint. Neither absent output nor future reference rows seed it.
    let baseline = &before["holder_balances"][0];
    let holder = hex_bytes(baseline["address"].as_str().unwrap()).unwrap();
    let mut amount = baseline["rpc"].as_str().unwrap().to_owned();
    let mut previous = None;
    for height in 58597371..=58597373 {
        let c = cases.iter().find(|c| c["block"] == height).unwrap();
        let b = captured(c);
        if let Some(prior) = previous {
            assert_eq!(b.header.as_ref().unwrap().parent_hash, prior);
        }
        let events = project(&b, std::slice::from_ref(l)).unwrap();
        assert_eq!(events.balances.len(), usize::from(height == 58597372));
        for row in events.balances {
            assert_eq!(row.address, holder);
            amount = row.amount;
        }
        assert_eq!(amount, c["holder_balances"][0]["rpc"].as_str().unwrap());
        previous = Some(b.hash);
    }
    assert_eq!(amount, "1000000000000000000");
}

#[test]
fn voting_history_writes_no_longer_prevent_complete_initial_mint_balances() {
    let layouts = layouts();
    let cases = cases();
    for height in [51995162, 58597372] {
        let c = cases.iter().find(|c| c["block"] == height).unwrap();
        assert!(c["prior_mapper_error"].as_str().unwrap().contains("unresolved storage"));
        let contract = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let b = captured(c);
        let rows = changes(&b, std::slice::from_ref(l)).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].old_amount, "0");
        assert_eq!(rows[0].amount, c["total_supply_rpc"].as_str().unwrap());
        let mut unsupported = l.clone();
        unsupported.voting_checkpoints = None;
        assert!(project(&b, &[unsupported]).unwrap_err().to_string().contains("unresolved storage"));
    }
}

#[test]
fn captured_mints_require_the_persisted_array_length_witness() {
    let layouts = layouts();
    let cases = cases();
    for height in [51995162, 58597372] {
        let c = cases.iter().find(|c| c["block"] == height).unwrap();
        let contract = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let roots = &l.voting_checkpoints.as_ref().unwrap().slots;
        let mut b = captured(c);
        let mut removed = 0;
        for call in b.transaction_traces.iter_mut().flat_map(|tx| &mut tx.calls) {
            call.storage_changes.retain(|s| {
                let keep = s.address != contract || !roots.contains(&word(&s.key).unwrap());
                removed += usize::from(!keep);
                keep
            });
        }
        assert_eq!(removed, 1);
        assert!(project(&b, std::slice::from_ref(l)).is_err());
    }
}
