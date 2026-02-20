use common::clickhouse::{bytes_to_hex, common_key, set_log};
use proto::pb::evm::erc721;
use substreams::pb::substreams::Clock;

use crate::enums::{TokenStandard, TransferType};

pub fn process_erc721(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: erc721::v1::Events) {
    let mut index = 0;

    for event in events.transfers {
        let key = common_key(clock, index);
        let row = tables
            .create_row("erc721_transfers", key)
            .set("token_id", &event.token_id)
            .set("from", bytes_to_hex(&event.from))
            .set("to", bytes_to_hex(&event.to))
            .set("operator", "".to_string())
            .set("amount", 1)
            .set("transfer_type", TransferType::Single.to_string())
            .set("token_standard", TokenStandard::ERC721.to_string());

        set_log(clock, index, event.tx_hash, event.contract, event.ordinal, event.caller, row);
        index += 1;
    }

    for event in events.approvals {
        let key = common_key(clock, index);
        let row = tables
            .create_row("erc721_approvals", key)
            .set("owner", bytes_to_hex(&event.owner))
            .set("approved", bytes_to_hex(&event.approved))
            .set("token_id", &event.token_id);

        set_log(clock, index, event.tx_hash, event.contract, event.ordinal, event.caller, row);
        index += 1;
    }

    for event in events.approvals_for_all {
        let key = common_key(clock, index);
        let row = tables
            .create_row("erc721_approvals_for_all", key)
            .set("owner", bytes_to_hex(&event.owner))
            .set("operator", bytes_to_hex(&event.operator))
            .set("approved", &event.approved.to_string())
            .set("token_standard", TokenStandard::ERC721.to_string());

        set_log(clock, index, event.tx_hash, event.contract, event.ordinal, event.caller, row);
        index += 1;
    }
}
