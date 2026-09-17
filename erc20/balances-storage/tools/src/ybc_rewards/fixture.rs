use super::{Hour, State};
use crate::data::uint;
use anyhow::{Context, Result};
use serde_json::{json, Value};

pub fn encode(s: &State) -> Value {
    json!({"raw":s.raw.to_string(),"static_rate":s.static_rate.to_string(),"wbnb_value":s.wbnb_value.to_string(),"lp_reward":s.lp_reward.to_string(),"now":s.now.to_string(),"launch_time":s.launch_time.to_string(),"last_cycle":s.last_cycle.to_string(),"initial_total_rate":s.initial_total_rate.to_string(),"initial_user_rate":s.initial_user_rate.to_string(),"pool_balance":s.pool_balance.to_string(),"burn_per_day":s.burn_per_day.to_string(),"token_reserve":s.token_reserve.to_string(),"quote_reserve":s.quote_reserve.to_string(),"hours":s.hours.iter().map(|h|json!({"no_burn":h.no_burn,"total_rate":h.total_rate.to_string(),"user_rate":h.user_rate.to_string(),"reward":h.reward.to_string()})).collect::<Vec<_>>()})
}

pub fn decode(v: &Value) -> Result<State> {
    Ok(State {
        raw: uint(&v["raw"])?,
        static_rate: uint(&v["static_rate"])?,
        wbnb_value: uint(&v["wbnb_value"])?,
        lp_reward: uint(&v["lp_reward"])?,
        now: uint(&v["now"])?,
        launch_time: uint(&v["launch_time"])?,
        last_cycle: uint(&v["last_cycle"])?,
        initial_total_rate: uint(&v["initial_total_rate"])?,
        initial_user_rate: uint(&v["initial_user_rate"])?,
        pool_balance: uint(&v["pool_balance"])?,
        burn_per_day: uint(&v["burn_per_day"])?,
        token_reserve: uint(&v["token_reserve"])?,
        quote_reserve: uint(&v["quote_reserve"])?,
        hours: v["hours"]
            .as_array()
            .context("missing initialized hourly state")?
            .iter()
            .map(|h| {
                Ok(Hour {
                    no_burn: h["no_burn"].as_bool().context("missing initialized burn flag")?,
                    total_rate: uint(&h["total_rate"])?,
                    user_rate: uint(&h["user_rate"])?,
                    reward: uint(&h["reward"])?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
}
