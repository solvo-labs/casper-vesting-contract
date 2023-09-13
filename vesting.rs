
use core::ops::{Add, Sub, Mul, Div};

use alloc::{
    string::{String, ToString},
    vec::Vec,
    vec,
    boxed::Box
};

use crate::{
    error::Error,
    interfaces::cep18::CEP18,
    utils::{get_current_address, get_key, self , read_from}, events::VestingEvent,
    events::emit, enums::Address
};

use casper_types::{
     account::AccountHash, U256,EntryPoint, Key, 
    ContractHash,EntryPointAccess,CLType,Parameter,EntryPointType,EntryPoints,
    contracts::NamedKeys,U512,RuntimeArgs,runtime_args
};
use casper_types_derive::{CLTyped, FromBytes, ToBytes};

use casper_contract::contract_api::{runtime, storage,system,account};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;

const CONTRACT_NAME: &str = "contract_name";
const VESTING_AMOUNT: &str = "vesting_amount";
const DEPOSIT_AMOUNT: &str = "deposit_amount";
const START_DATE : &str = "start_date";
const DURATION: &str = "duration";
const CEP18_CONTRACT_HASH: &str = "cep18_contract_hash";
const OWNER: &str = "owner";
const ROUND_NAME: &str = "round_name";
const VESTING_ADDRESS: &str = "vesting_address";
const CONTRACT_HASH: &str = "contract_hash";
const RECIPIENTS: &str = "recipients";
const ALLOCATIONS: &str = "allocations";
const RECIPIENT: &str = "recipient";
const CLAIM_AMOUNT: &str = "claim_amount";
const CLIFF_TIMESTAMP: &str = "cliff_timestamp";
const RELEASE_DATE : &str = "release_date";
const END_DATE : &str = "end_date";
const INDEX : &str = "index";

// entry points 
const ENTRY_POINT_CLAIM: &str = "claim";
const ENTRY_POINT_INIT: &str = "init";

// dicts
const RECIPIENTS_DICT: &str = "recipients_dict";
const ALLOCATIONS_DICT: &str = "allocations_dict";
const CLAIMED_DICT: &str = "claimed_dict";

// #[derive(Clone, Copy, Debug, CLTyped, ToBytes, FromBytes)]
// pub struct Vest {
//     pub recipient: Key,
//     pub amount: U256,
//     pub claimed: U256,
// }


#[no_mangle]
pub extern "C" fn claim() {
    // arguements
    let cep18_contract_hash = runtime::get_named_arg::<Key>(CEP18_CONTRACT_HASH)
    .into_hash()
    .map(ContractHash::new)
    .unwrap();
    let index: i32 = runtime::get_named_arg(INDEX);

    let recipients_dict = *runtime::get_key(RECIPIENTS_DICT).unwrap().as_uref().unwrap();
    let allocations_dict = *runtime::get_key(ALLOCATIONS_DICT).unwrap().as_uref().unwrap();
    let claimed_dict = *runtime::get_key(ALLOCATIONS_DICT).unwrap().as_uref().unwrap();

    let recipient =  storage::dictionary_get::<Key>(recipients_dict, &index.to_string()).unwrap().unwrap();
    let allocation =  storage::dictionary_get::<U256>(allocations_dict, &index.to_string()).unwrap().unwrap();
    let claimed =  storage::dictionary_get::<U256>(claimed_dict, &index.to_string()).unwrap().unwrap();

    // dates
    let start_date :u64 = utils::read_from(START_DATE);
    let release_date :u64 = utils::read_from(RELEASE_DATE);
    let end_date :u64 = utils::read_from(END_DATE);
    let duration :u64 = utils::read_from(DURATION);

    // utils
    let caller = runtime::get_caller();
    let now : u64 = runtime::get_blocktime().into();

    // dicts
    let start_date :u64 = utils::read_from(START_DATE);
    let release_date :u64 = utils::read_from(RELEASE_DATE);

    let cep18 = CEP18::new(cep18_contract_hash);
   
    let calculated_amount = allocation.mul(now.sub(start_date)).div(duration);
    let remaining_amount = calculated_amount.sub(claimed);

    cep18.transfer(recipient, remaining_amount);

    storage::dictionary_put(claimed_dict, &index.to_string(), calculated_amount);  


    emit(&VestingEvent::Claim { cep18_contract_hash,recipient, claim_amount : remaining_amount });
}

#[no_mangle]
pub extern "C" fn init() {
    let contract_name: String = runtime::get_named_arg(CONTRACT_NAME);
    let vesting_amount : U256 = runtime::get_named_arg(VESTING_AMOUNT);
    let cep18_contract_hash = runtime::get_named_arg::<Key>(CEP18_CONTRACT_HASH)
    .into_hash()
    .map(ContractHash::new)
    .unwrap();

    let start_date : u64 = runtime::get_named_arg(START_DATE);
    let duration : u64 = runtime::get_named_arg(DURATION);

    let recipients : Vec<Key> = runtime::get_named_arg(RECIPIENTS);
    let allocations : Vec<U256> = runtime::get_named_arg(ALLOCATIONS);

    let cliff_timestamp : u64 = runtime::get_named_arg(CLIFF_TIMESTAMP);

    let owner : AccountHash = runtime::get_caller().into();

    let release_date : u64 = start_date.add(cliff_timestamp);
    let end_date: u64 = start_date.add(release_date);

    let now : u64 = runtime::get_blocktime().into();

    runtime::put_key(CONTRACT_NAME, storage::new_uref(contract_name).into());
    runtime::put_key(CEP18_CONTRACT_HASH, storage::new_uref(cep18_contract_hash).into());
    runtime::put_key(START_DATE, storage::new_uref(start_date).into());
    runtime::put_key(DURATION, storage::new_uref(duration).into());
    runtime::put_key(OWNER, storage::new_uref(owner).into());
    runtime::put_key(VESTING_AMOUNT, storage::new_uref(vesting_amount).into());
    runtime::put_key(CLIFF_TIMESTAMP, storage::new_uref(cliff_timestamp).into());
    runtime::put_key(RELEASE_DATE, storage::new_uref(release_date).into());
    runtime::put_key(END_DATE, storage::new_uref(end_date).into());

    storage::new_dictionary(RECIPIENTS_DICT).unwrap_or_default();
    storage::new_dictionary(ALLOCATIONS_DICT ).unwrap_or_default();
    storage::new_dictionary(CLAIMED_DICT).unwrap_or_default();

    let recipients_dict = *runtime::get_key(RECIPIENTS_DICT).unwrap().as_uref().unwrap();
    let allocations_dict = *runtime::get_key(ALLOCATIONS_DICT).unwrap().as_uref().unwrap();
    let claimed_dict = *runtime::get_key(CLAIMED_DICT).unwrap().as_uref().unwrap();

    for (index, recipient) in recipients.into_iter().enumerate() {
        storage::dictionary_put(recipients_dict, &index.to_string(), recipient);  
    }

    for (index, allocation) in allocations.into_iter().enumerate() {
        storage::dictionary_put(allocations_dict, &index.to_string(), allocation);  
        storage::dictionary_put(claimed_dict, &index.to_string(), 0);  
    }

}


#[no_mangle]
pub extern "C" fn call() {
let contract_name: String = runtime::get_named_arg(CONTRACT_NAME);
let vesting_amount : U256 = runtime::get_named_arg(VESTING_AMOUNT);
let cep18_contract_hash = runtime::get_named_arg::<Key>(CEP18_CONTRACT_HASH);

let start_date : u64 = runtime::get_named_arg(START_DATE);
let duration : u64 = runtime::get_named_arg(DURATION);

let recipients : Vec<Key> = runtime::get_named_arg(RECIPIENTS);
let allocations : Vec<U256> = runtime::get_named_arg(ALLOCATIONS);

let cliff_timestamp : u64 = runtime::get_named_arg(CLIFF_TIMESTAMP);

 let init_entry_point = EntryPoint::new(
    ENTRY_POINT_INIT,
    vec![
        Parameter::new(CONTRACT_NAME, CLType::String),
        Parameter::new(VESTING_AMOUNT, CLType::U256),
        Parameter::new(CEP18_CONTRACT_HASH, CLType::Key),
        Parameter::new(START_DATE, CLType::U64),
        Parameter::new(DURATION, CLType::U64),
        Parameter::new(RECIPIENTS, CLType::List(Box::new(CLType::Key))),
        Parameter::new(ALLOCATIONS, CLType::List(Box::new(CLType::U256))),
        Parameter::new(CLIFF_TIMESTAMP, CLType::U64),
    ],
    CLType::URef,
    EntryPointAccess::Public,
    EntryPointType::Contract,
 );  


let claim_entry_point = EntryPoint::new(
    ENTRY_POINT_CLAIM,
    vec![
        Parameter::new(CEP18_CONTRACT_HASH, CLType::Key),
        Parameter::new(INDEX, CLType::I32),
    ],
    CLType::Unit,
    EntryPointAccess::Public,
    EntryPointType::Contract,
 );  

let mut entry_points = EntryPoints::new();
      entry_points.add_entry_point(init_entry_point);
      entry_points.add_entry_point(claim_entry_point);
    
 
let mut named_keys = NamedKeys::new();
 
 let str1 = contract_name.to_string();
 let str2 = String::from("vesting_package_hash_");
 let str3 = String::from("vesting_access_uref_");
 let str4 = String::from("vesting_contract_hash_");
 let hash_name = str2 + &str1;
 let uref_name = str3 + &str1;
 let contract_hash_text = str4 + &str1;

  let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_name.to_string()),
        Some(uref_name.to_string()),
   );
   runtime::put_key(&contract_hash_text.to_string(), contract_hash.into());

   runtime::call_contract::<()>(
    contract_hash,
    ENTRY_POINT_INIT,
    runtime_args! {
        CONTRACT_NAME => contract_name,
        VESTING_AMOUNT => vesting_amount,
        CEP18_CONTRACT_HASH => cep18_contract_hash,
        START_DATE => start_date,
        DURATION => duration,
        RECIPIENTS => recipients,
        ALLOCATIONS => allocations,
        CLIFF_TIMESTAMP => cliff_timestamp
    },
    )
}

// add init function

