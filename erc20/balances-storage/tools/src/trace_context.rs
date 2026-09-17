//! Account attribution for diagnostic EVM traces. A call depth alone does not
//! identify storage: DELEGATECALL and CALLCODE retain their caller's account.
use crate::{
    data::*,
    rpc::{quantity, Rpc},
};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

#[derive(Clone)]
struct Frame {
    storage: String,
    code: String,
}

fn stack_word(step: &Value, from_top: usize) -> Result<primitive_types::U256> {
    let stack = items(step, "stack")?;
    let index = stack.len().checked_sub(from_top + 1).context("missing trace stack word")?;
    let value = text(&stack[index])?;
    quantity(&json!(format!("0x{}", value.strip_prefix("0x").unwrap_or(value))))
}

fn call_target(step: &Value) -> Result<Option<String>> {
    if !matches!(text(&step["op"])?, "CALL" | "STATICCALL" | "DELEGATECALL" | "CALLCODE") {
        return Ok(None);
    }
    let mut word = [0; 32];
    stack_word(step, 1)?.to_big_endian(&mut word);
    // The EVM interprets only the low 160 bits of a call target.
    Ok(Some(format!("0x{}", hex::encode(&word[12..]))))
}

/// Inspect an execution trace, without promoting its observed path to a layout.
/// Account creation frames are rejected because their storage address cannot be
/// recovered from just the pre-op stack. Missing depth/stack evidence is an error.
pub fn inspect(trace: &Value, contract: &str) -> Result<Value> {
    let root = binary(&json!(contract), 20)?;
    let steps = items(trace, "structLogs")?;
    ensure!(!steps.is_empty(), "empty execution trace");
    ensure!(number(&steps[0]["depth"])? == 1, "trace must start at root depth one");
    ensure!(number(&steps.last().unwrap()["depth"])? == 1, "trace ends before returning to the root frame");
    let mut frames = vec![Frame {
        storage: root.clone(),
        code: root,
    }];
    let mut reads = vec![];
    let mut calls = vec![];
    for (index, step) in steps.iter().enumerate() {
        let pc = number(&step["pc"])?;
        let depth = usize::try_from(number(&step["depth"])?)?;
        ensure!(depth > 0 && depth <= frames.len() + 1, "invalid trace depth transition");
        if depth == frames.len() + 1 {
            let previous = steps.get(index.checked_sub(1).context("missing caller step")?).context("missing caller step")?;
            let target = call_target(previous)?.context("entered frame without a supported call opcode")?;
            let parent = frames.last().context("missing parent frame")?;
            let storage = if matches!(text(&previous["op"])?, "DELEGATECALL" | "CALLCODE") {
                parent.storage.clone()
            } else {
                target.clone()
            };
            frames.push(Frame { storage, code: target });
        } else {
            frames.truncate(depth);
        }
        let frame = frames.last().context("missing execution frame")?;
        if let Some(target) = call_target(step)? {
            let entered = steps.get(index + 1).is_some_and(|next| next["depth"].as_u64() == Some(depth as u64 + 1));
            calls.push(
                json!({"pc":pc,"depth":depth,"opcode":step["op"],"storage_address":frame.storage,"code_address":frame.code,"target":target,"entered":entered}),
            );
        }
        if step["op"] == "SLOAD" {
            let next = steps.get(index + 1).context("missing SLOAD result")?;
            ensure!(next["depth"] == step["depth"], "SLOAD trace depth changed");
            reads.push(json!({"pc":pc,"depth":depth,"storage_address":frame.storage,"code_address":frame.code,"key":format!("0x{:064x}",stack_word(step,0)?),"value":format!("0x{:064x}",stack_word(next,0)?)}));
        }
    }
    Ok(
        json!({"storage_reads":reads,"calls":calls,"scope":"Observed execution path only; unexecuted branches and uninitialized holder state are not qualified"}),
    )
}

/// Bind attributed instruction addresses to the canonical historical runtime.
/// In particular, EIP-7702 can execute another account's code without an explicit
/// DELEGATECALL step. Reject such unresolved designations rather than mislabeling
/// their execution as the authority account's code.
pub fn validate_code(rpc: &dyn Rpc, reference: &Value, context: &Value) -> Result<Vec<Value>> {
    let mut instructions = std::collections::BTreeMap::<String, Vec<(usize, u8)>>::new();
    for (field, fixed_opcode) in [("storage_reads", Some(0x54)), ("calls", None)] {
        for row in items(context, field)? {
            let address = binary(&row["code_address"], 20)?;
            let opcode = match fixed_opcode {
                Some(op) => op,
                None => match text(&row["opcode"])? {
                    "CALL" => 0xf1,
                    "CALLCODE" => 0xf2,
                    "DELEGATECALL" => 0xf4,
                    "STATICCALL" => 0xfa,
                    _ => anyhow::bail!("unsupported attributed call opcode"),
                },
            };
            instructions.entry(address).or_default().push((usize::try_from(number(&row["pc"])?)?, opcode));
        }
    }
    let mut identities = vec![];
    for (address, instructions) in instructions {
        let runtime = rpc.call("eth_getCode", json!([address, reference]))?;
        let bytes = hex::decode(text(&runtime)?.strip_prefix("0x").context("runtime prefix")?)?;
        ensure!(
            bytes.len() != 23 || !bytes.starts_with(&[0xef, 0x01, 0x00]),
            "EIP-7702 code designation requires separate trace code resolution"
        );
        let mut boundaries = std::collections::BTreeMap::new();
        let mut pc = 0;
        while let Some(opcode) = bytes.get(pc) {
            boundaries.insert(pc, *opcode);
            pc += 1 + if (0x60..=0x7f).contains(opcode) { usize::from(*opcode - 0x5f) } else { 0 };
        }
        ensure!(
            instructions.iter().all(|(pc, op)| boundaries.get(pc) == Some(op)),
            "attributed trace instruction differs from historical runtime"
        );
        identities.push(json!({"address":address,"runtime_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&bytes))),"checked_instructions":instructions.len()}));
    }
    Ok(identities)
}

#[cfg(test)]
mod tests;
