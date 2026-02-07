use common::{bytes_to_string, Encoding};
use proto::pb::erc20::tokens::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{log_key, logs::set_template_log, set_clock, transactions::set_template_tokens_tx};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            // WETH Deposit
            if let Some(pb::log::Log::WethDeposit(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("weth_deposit", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("dst", bytes_to_string(&event.dst, encoding));
                row.set("wad", &event.wad);
            }

            // WETH Withdrawal
            if let Some(pb::log::Log::WethWithdrawal(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("weth_withdrawal", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("src", bytes_to_string(&event.src, encoding));
                row.set("wad", &event.wad);
            }

            // USDC Mint
            if let Some(pb::log::Log::UsdcMint(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdc_mint", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("minter", bytes_to_string(&event.minter, encoding));
                row.set("to", bytes_to_string(&event.to, encoding));
                row.set("amount", &event.amount);
            }

            // USDC Burn
            if let Some(pb::log::Log::UsdcBurn(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdc_burn", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("burner", bytes_to_string(&event.burner, encoding));
                row.set("amount", &event.amount);
            }

            // USDT Issue
            if let Some(pb::log::Log::UsdtIssue(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdt_issue", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("owner", bytes_to_string(&event.owner, encoding));
                row.set("amount", &event.amount);
            }

            // USDT Redeem
            if let Some(pb::log::Log::UsdtRedeem(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdt_redeem", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("owner", bytes_to_string(&event.owner, encoding));
                row.set("amount", &event.amount);
            }

            // USDT DestroyedBlackFunds
            if let Some(pb::log::Log::UsdtDestroyedBlackFunds(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdt_destroyed_black_funds", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("black_listed_user", bytes_to_string(&event.black_listed_user, encoding));
                row.set("balance", &event.balance);
            }

            // WBTC Mint
            if let Some(pb::log::Log::WbtcMint(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("wbtc_mint", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("to", bytes_to_string(&event.to, encoding));
                row.set("amount", &event.amount);
            }

            // WBTC Burn
            if let Some(pb::log::Log::WbtcBurn(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("wbtc_burn", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("burner", bytes_to_string(&event.burner, encoding));
                row.set("value", &event.value);
            }

            // SAI Mint
            if let Some(pb::log::Log::SaiMint(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("sai_mint", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("guy", bytes_to_string(&event.guy, encoding));
                row.set("wad", &event.wad);
            }

            // SAI Burn
            if let Some(pb::log::Log::SaiBurn(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("sai_burn", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("guy", bytes_to_string(&event.guy, encoding));
                row.set("wad", &event.wad);
            }

            // stETH Submitted
            if let Some(pb::log::Log::StethSubmitted(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_submitted", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("sender", bytes_to_string(&event.sender, encoding));
                row.set("amount", &event.amount);
                row.set("referral", bytes_to_string(&event.referral, encoding));
            }

            // stETH TokenRebased
            if let Some(pb::log::Log::StethTokenRebased(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_token_rebased", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("report_timestamp", &event.report_timestamp);
                row.set("time_elapsed", &event.time_elapsed);
                row.set("pre_total_shares", &event.pre_total_shares);
                row.set("pre_total_ether", &event.pre_total_ether);
                row.set("post_total_shares", &event.post_total_shares);
                row.set("post_total_ether", &event.post_total_ether);
                row.set("shares_minted_as_fees", &event.shares_minted_as_fees);
            }

            // stETH TransferShares
            if let Some(pb::log::Log::StethTransferShares(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_transfer_shares", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("from", bytes_to_string(&event.from, encoding));
                row.set("to", bytes_to_string(&event.to, encoding));
                row.set("shares_value", &event.shares_value);
            }

            // stETH SharesBurnt
            if let Some(pb::log::Log::StethSharesBurnt(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_shares_burnt", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("account", bytes_to_string(&event.account, encoding));
                row.set("pre_rebase_token_amount", &event.pre_rebase_token_amount);
                row.set("post_rebase_token_amount", &event.post_rebase_token_amount);
                row.set("shares_amount", &event.shares_amount);
            }

            // stETH ExternalSharesMinted
            if let Some(pb::log::Log::StethExternalSharesMinted(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_external_shares_minted", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("recipient", bytes_to_string(&event.recipient, encoding));
                row.set("amount_of_shares", &event.amount_of_shares);
            }

            // stETH ExternalSharesBurnt
            if let Some(pb::log::Log::StethExternalSharesBurnt(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_external_shares_burnt", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tokens_tx(encoding, tx, tx_index, row);

                row.set("owner", bytes_to_string(&event.owner, encoding));
                row.set("amount_of_shares", &event.amount_of_shares);
            }
        }
    }
}
