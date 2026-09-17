//! Shared decoding for the captured regression and its Rust fixture builder.
use super::{Globals, Holder};
use crate::{
    data::{items, text, uint},
    survey::mapping_key,
};
use anyhow::{ensure, Context, Result};
use primitive_types::U256;
use serde_json::Value;
use std::collections::BTreeMap;

pub const TOKEN: &str = "0x88886f0fd371dff856291badced45922bc888888";
pub const DEP: &str = "0x5e3cbc82d020be91a989eb747934104e9ab585fe";
pub const ZERO: &str = "0x0000000000000000000000000000000000000000";
pub const DEAD: &str = "0x000000000000000000000000000000000000dead";
pub type State = BTreeMap<(String, String), U256>;

pub fn slot(n: u64) -> String {
    format!("0x{n:064x}")
}

pub fn checkpoint(rows: &Value) -> Result<State> {
    let mut state = State::new();
    for row in rows.as_array().context("checkpoint array")? {
        ensure!(
            state
                .insert((text(&row["contract"])?.into(), text(&row["key"])?.into()), uint(&row["word"])?)
                .is_none(),
            "duplicate checkpoint key"
        );
    }
    Ok(state)
}

pub fn apply(state: &mut State, updates: &Value) -> Result<()> {
    let mut ordinal = None;
    for row in updates.as_array().context("update array")? {
        let next = row["ordinal"].as_u64().context("update ordinal")?;
        ensure!(ordinal.is_none_or(|prior| next > prior), "ambiguous update ordinal");
        ordinal = Some(next);
        let key = (text(&row["contract"])?.into(), text(&row["key"])?.into());
        let stored = state.get_mut(&key).context("missing tracked checkpoint")?;
        ensure!(*stored == uint(&row["old"])?, "tracked storage discontinuity");
        *stored = uint(&row["new"])?;
    }
    Ok(())
}

pub fn decode(state: &State, address: &str, exemptions: &[String]) -> Result<(Globals, Holder)> {
    let get = |contract: &str, key: String| -> Result<U256> { state.get(&(contract.into(), key)).copied().context("uninitialized reward field") };
    let mut globals = Globals {
        supply: get(DEP, slot(2))?,
        static_acc: get(DEP, slot(7))?,
        node_acc: get(DEP, slot(8))?,
        nodes: get(DEP, slot(12))?,
        accounted: get(DEP, slot(17))?,
        ..Default::default()
    };
    globals.set_packed(get(DEP, slot(6))?);
    let mapped = |contract, n| get(contract, mapping_key(address, &slot(n))?);
    let holder = Holder {
        raw: mapped(TOKEN, 0)?,
        shares: mapped(DEP, 0)?,
        user_index: mapped(DEP, 9)?,
        user_node_index: mapped(DEP, 10)?,
        is_node: !(mapped(DEP, 11)? & U256::from(255)).is_zero(),
        zero_or_dead: address == ZERO || address == DEAD,
        exempt: exemptions.iter().any(|v| v == address),
    };
    Ok((globals, holder))
}

pub fn exemptions(manifest: &Value) -> Result<Vec<String>> {
    items(manifest, "exemptions")?.iter().map(|v| Ok(text(v)?.to_owned())).collect()
}
