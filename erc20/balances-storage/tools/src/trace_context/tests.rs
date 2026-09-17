use super::*;

const ROOT: &str = "0x1111111111111111111111111111111111111111";
const PROXY: &str = "0x2222222222222222222222222222222222222222";
const IMPLEMENTATION: &str = "0x3333333333333333333333333333333333333333";

fn step(depth: u64, op: &str, stack: &[&str]) -> Value {
    json!({"pc":0,"depth":depth,"op":op,"stack":stack})
}

#[test]
fn proxy_reads_use_proxy_storage_and_unwind_to_each_caller() {
    let trace = json!({"structLogs":[
        step(1,"STATICCALL",&[PROXY,"ffff"]),
        step(2,"DELEGATECALL",&[IMPLEMENTATION,"ffff"]),
        step(3,"SLOAD",&["7"]),step(3,"POP",&["abc"]),
        step(2,"SLOAD",&["8"]),step(2,"POP",&["def"]),
        step(1,"CALL",&[IMPLEMENTATION,"ffff"]),
        step(2,"SLOAD",&["7"]),step(2,"POP",&["123"]),
        step(1,"CALLCODE",&[IMPLEMENTATION,"ffff"]),
        step(2,"SLOAD",&["7"]),step(2,"POP",&["456"]),
        step(1,"SLOAD",&["7"]),step(1,"STOP",&["789"])
    ]});
    let result = inspect(&trace, ROOT).unwrap();
    let reads = result["storage_reads"].as_array().unwrap();
    assert_eq!(reads.len(), 5);
    for (read, (storage, code, value)) in reads.iter().zip([
        (PROXY, IMPLEMENTATION, 0xabc),
        (PROXY, PROXY, 0xdef),
        (IMPLEMENTATION, IMPLEMENTATION, 0x123),
        (ROOT, IMPLEMENTATION, 0x456),
        (ROOT, ROOT, 0x789),
    ]) {
        assert_eq!(read["storage_address"], storage);
        assert_eq!(read["code_address"], code);
        assert_eq!(read["value"], format!("0x{value:064x}"));
    }
    assert!(result["calls"].as_array().unwrap().iter().all(|c| c["entered"] == true));
}

#[test]
fn calls_without_child_steps_do_not_change_context_and_targets_use_low_160_bits() {
    let high_target = format!("0x{}{}", "ff".repeat(12), &PROXY[2..]);
    let trace = json!({"structLogs":[
        step(1,"CALL",&[&high_target,"ffff"]),
        step(1,"SLOAD",&["0x1"]),step(1,"STOP",&["0x2"])
    ]});
    let result = inspect(&trace, ROOT).unwrap();
    assert_eq!(result["calls"][0]["target"], PROXY);
    assert_eq!(result["calls"][0]["entered"], false);
    assert_eq!(result["storage_reads"][0]["storage_address"], ROOT);
}

#[test]
fn missing_or_unresolvable_trace_evidence_is_rejected() {
    for steps in [
        vec![],
        vec![step(2, "STOP", &[])],
        vec![step(1, "CALL", &[])],
        vec![step(1, "CALL", &[PROXY, "ffff"]), step(2, "STOP", &[])],
        vec![step(1, "CALL", &[PROXY, "ffff"]), step(3, "STOP", &[]), step(1, "STOP", &[])],
        vec![step(1, "CREATE", &["0", "0", "0"]), step(2, "STOP", &[]), step(1, "STOP", &[])],
        vec![step(1, "POP", &[]), step(2, "STOP", &[]), step(1, "STOP", &[])],
        vec![step(1, "SLOAD", &["1"])],
        vec![step(1, "SLOAD", &["1"]), step(2, "STOP", &["2"]), step(1, "STOP", &[])],
    ] {
        assert!(inspect(&json!({"structLogs":steps}), ROOT).is_err());
    }
}

fn captured(name: &str) -> Value {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/external-getter-contexts");
    let cases: Vec<Value> = serde_json::from_slice(&std::fs::read(folder.join("cases.json")).unwrap()).unwrap();
    let case = cases.into_iter().find(|case| case["name"] == name).unwrap();
    let path = folder.join(case["trace"].as_str().unwrap());
    assert_eq!(sha256(&path).unwrap(), case["trace_sha256"]);
    let trace: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let context = inspect(&trace, case["contract"].as_str().unwrap()).unwrap();
    assert_eq!(context, case["expected"]);
    // Expectations were checked independently against canonical storage and
    // explicit overrides, with callTracer and each account's historical code.
    for read in context["storage_reads"].as_array().unwrap() {
        let independent = case["independent_storage_words"]
            .as_array()
            .unwrap()
            .iter()
            .find(|word| word["address"] == read["storage_address"] && word["key"] == read["key"])
            .unwrap();
        assert_eq!(read["value"], independent["value"]);
    }
    context
}

#[test]
fn captured_titan_delegate_reads_belong_to_proxy_storage() {
    let context = captured("titan");
    let reads = context["storage_reads"].as_array().unwrap();
    assert_eq!(reads.len(), 29);
    assert_eq!(context["calls"].as_array().unwrap().len(), 4);
    let delegate = "0x0541727a60316913f78eb7fd5524700acf89264c";
    let proxy = "0x4dd028f2cddeb2e3da83efce41703844dfdb4b64";
    let delegated = reads.iter().filter(|r| r["code_address"] == delegate).collect::<Vec<_>>();
    assert!(!delegated.is_empty());
    assert!(delegated.iter().all(|r| r["storage_address"] == proxy));
    assert!(!reads.iter().any(|r| r["storage_address"] == delegate));
}

#[test]
fn captured_ord_branch_preserves_nested_and_sibling_account_contexts() {
    let context = captured("ord-participation");
    let reads = context["storage_reads"].as_array().unwrap();
    assert_eq!(reads.len(), 26);
    assert!(reads.iter().all(|r| r["code_address"] == r["storage_address"]));
    let accounts = reads
        .iter()
        .map(|r| r["storage_address"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        accounts,
        std::collections::BTreeSet::from([
            "0x1ed4fd5ca00d302e038920119ebfbef90b052ce6",
            "0x885584e6ee2f8bdcdd7be5898da49f63050e8899",
            "0xca8bf23801341c3ba2e696d06e9679647f5b0f9b",
            "0x16b9a82891338f9ba80e2d6970fdda79d1eb0dae",
        ])
    );
    let calls = context["calls"].as_array().unwrap();
    assert_eq!(calls.len(), 4);
    // Unwinding the token -> registry call must restore the dependency frame,
    // then give the router -> pool branch its own storage account.
    assert_eq!(calls[2]["storage_address"], "0x1ed4fd5ca00d302e038920119ebfbef90b052ce6");
    assert_eq!(calls[2]["target"], "0x10ed43c718714eb63d5aa57b78b54704e256024e");
    assert_eq!(calls[3]["storage_address"], "0x10ed43c718714eb63d5aa57b78b54704e256024e");
    assert_eq!(calls[3]["target"], "0x16b9a82891338f9ba80e2d6970fdda79d1eb0dae");
}

#[test]
fn code_attribution_requires_matching_historical_opcodes_and_resolved_runtime() {
    struct CodeRpc(&'static str);
    impl crate::rpc::Rpc for CodeRpc {
        fn request(&self, request: Value) -> Result<Value> {
            assert_eq!(request["method"], "eth_getCode");
            assert_eq!(request["params"][0], ROOT);
            assert_eq!(request["params"][1]["requireCanonical"], true);
            Ok(json!({"id":request["id"],"result":self.0}))
        }
    }
    let context = json!({"storage_reads":[{"code_address":ROOT,"pc":0}],"calls":[{"code_address":ROOT,"pc":1,"opcode":"STATICCALL"}]});
    let reference = crate::rpc::block_ref(&format!("0x{}", "ab".repeat(32)));
    let identities = validate_code(&CodeRpc("0x54fa"), &reference, &context).unwrap();
    assert_eq!(identities[0]["checked_instructions"], 2);
    assert!(validate_code(&CodeRpc("0x55fa"), &reference, &context).is_err());
    assert!(validate_code(&CodeRpc("0x54f1"), &reference, &context).is_err());
    let mut only_call = context.clone();
    only_call["storage_reads"] = json!([]);
    assert!(
        validate_code(&CodeRpc("0x60fa"), &reference, &only_call).is_err(),
        "PUSH data is not a call instruction"
    );
    assert!(
        validate_code(&CodeRpc("0xef01003333333333333333333333333333333333333333"), &reference, &context)
            .unwrap_err()
            .to_string()
            .contains("EIP-7702")
    );
}
