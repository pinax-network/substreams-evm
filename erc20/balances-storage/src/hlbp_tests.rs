use super::*;
use prost::Message;
use serde_json::Value;

#[test]
fn captured_hlbp_mint_matches_rpc_and_requires_qualified_bookkeeping() {
    let layouts = layout::parse(include_str!("../tests/fixtures/hlbp-active/layouts.json")).unwrap();
    let expected: Value = serde_json::from_str(include_str!("../tests/fixtures/hlbp-active/case.json")).unwrap();
    let block = eth::Block::decode(include_bytes!("../tests/fixtures/hlbp-active/104727184.pb").as_slice()).unwrap();
    assert_eq!(block.number, expected["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(expected["hash"].as_str().unwrap()).unwrap());
    assert_eq!(layouts.len(), 1);
    assert_eq!(layout::parse(include_str!("../tests/fixtures/bsc-hlbp-layouts.json")).unwrap().len(), 199);
    assert_eq!(expected["before_rpc"], "0");
    assert_ne!(expected["after_rpc"], "0");

    let events = project(&block, &layouts).unwrap();
    assert_eq!(events.balances.len(), 1);
    let row = &events.balances[0];
    assert_eq!(row.contract.as_ref(), Some(&layouts[0].contract));
    assert_eq!(row.address, hex_bytes(expected["holder"].as_str().unwrap()).unwrap());
    assert_eq!(row.amount, expected["after_rpc"].as_str().unwrap());
    assert_eq!(expected["mints"].as_array().unwrap().len(), 1);
    assert_eq!(expected["mints"][0]["amount"], expected["after_rpc"]);

    // notifyCredit also updates registeredLp (mapping 5); balance equality does
    // not authorize silently ignoring that separate persisted field.
    let mut missing = layouts[0].clone();
    assert!(missing.other_mapping_slots.remove(&word(&[5]).unwrap()));
    assert!(project(&block, &[missing]).unwrap_err().to_string().contains("unresolved storage"));
}
