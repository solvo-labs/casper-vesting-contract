use crate::{ alloc::string::ToString, utils::get_current_address };
use alloc::{ collections::BTreeMap, vec::Vec };
use casper_contract::contract_api::storage;
use casper_types::{ URef, U256, Key, ContractHash };

pub enum VestingEvent {
    Claim {
        cep18_contract_hash: ContractHash,
        recipient: Key,
        claim_amount: U256,
    },
}

pub fn emit(event: &VestingEvent) {
    let mut events = Vec::new();
    let mut param = BTreeMap::new();
    param.insert(
        "contract_package_hash",
        get_current_address().as_contract_package_hash().unwrap().to_string()
    );
    match event {
        VestingEvent::Claim { cep18_contract_hash, recipient, claim_amount } => {
            param.insert("event_type", "claim".to_string());
            param.insert("round_name", cep18_contract_hash.to_string());
            param.insert("recipient", recipient.to_string());
            param.insert("deposit_amount", claim_amount.to_string());
        }
    }
    events.push(param);
    for param in events {
        let _: URef = storage::new_uref(param);
    }
}
