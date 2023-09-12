
use core::ops::Add;

use alloc::{
    string::{String, ToString},
    vec::Vec,
    vec,
    boxed::Box
};

use crate::{
    error::Error,
    interfaces::cep18::CEP18,
    utils::{get_current_address, get_key, self}, events::VestingEvent,
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
const CLIFF_AMOUNT: &str = "cliff_amount";
const RELEASE_DATE : &str = "release_date";
const END_DATE : &str = "end_date";
const INDEX : &str = "index";

// entry points 
const ENTRY_POINT_CLAIM: &str = "claim";
const ENTRY_POINT_INIT: &str = "init";

// dicts
const ALLOCATION: &str = "allocation";
const CLAIMED: &str = "claimed";

// #[derive(Clone, Copy, Debug, CLTyped, ToBytes, FromBytes)]
// pub struct Vest {
//     pub recipient: Key,
//     pub amount: U256,
//     pub claimed: U256,
// }


#[no_mangle]
pub extern "C" fn claim() {
    // utils
    let now : u64 = runtime::get_blocktime().into();
    let caller = runtime::get_caller();

    // contract
    let cep18_contract_hash = runtime::get_named_arg::<Key>(CEP18_CONTRACT_HASH)
    .into_hash()
    .map(ContractHash::new)
    .unwrap();

    // params temp
    let recipient : Key = runtime::get_named_arg(RECIPIENT);
    let claim_amount : U256 = runtime::get_named_arg(CLAIM_AMOUNT);

    // date's
    let start_date : u64 =  utils::read_from(START_DATE);
    let duration : u64 =  utils::read_from(DURATION);
    let release_date : u64 = utils::read_from(RELEASE_DATE);
    let end_date: u64 = utils::read_from(END_DATE);

    // dicts
    let recipients: Vec<Key> = utils::read_from(RECIPIENTS);
    let allocations: Vec<U256> = utils::read_from(ALLOCATIONS);

    // amounts
    let vesting_amount : U256 = runtime::get_named_arg(VESTING_AMOUNT);
    let cliff_amount : U256 = runtime::get_named_arg(CLIFF_AMOUNT);

    // let index : i32 = runtime::get_named_arg(INDEX);
    let index : i32 = 1;

    let d = allocations[index];
    let f = Some(d);
    // let current_recipient = recipients.get(index).into();
    let allocation  = allocations.get(index);


    if isExist < 0 {
        runtime::revert(Error::FatalError)
    }

    let cep18 = CEP18::new(cep18_contract_hash);
    cep18.transfer(recipient, claim_amount);

    let claim_dict = *runtime::get_key(CLAIMED).unwrap().as_uref().unwrap();

    storage::dictionary_put(claim_dict, &isExist.to_string(), claim_amount);

    emit(&VestingEvent::Claim { cep18_contract_hash,recipient, claim_amount });
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
    let cliff_amount : U256 = runtime::get_named_arg(CLIFF_AMOUNT);

    let owner : AccountHash = runtime::get_caller().into();

    let release_date : u64 = start_date.add(cliff_timestamp);
    let end_date: u64 = start_date.add(release_date);

    runtime::put_key(CONTRACT_NAME, storage::new_uref(contract_name).into());
    runtime::put_key(CEP18_CONTRACT_HASH, storage::new_uref(cep18_contract_hash).into());
    runtime::put_key(START_DATE, storage::new_uref(start_date).into());
    runtime::put_key(DURATION, storage::new_uref(duration).into());
    runtime::put_key(OWNER, storage::new_uref(owner).into());
    runtime::put_key(VESTING_AMOUNT, storage::new_uref(vesting_amount).into());
    runtime::put_key(RECIPIENTS, storage::new_uref(recipients).into());
    runtime::put_key(ALLOCATIONS, storage::new_uref(allocations).into());
    runtime::put_key(CLIFF_TIMESTAMP, storage::new_uref(cliff_timestamp).into());
    runtime::put_key(CLIFF_AMOUNT, storage::new_uref(cliff_amount).into());
    runtime::put_key(RELEASE_DATE, storage::new_uref(release_date).into());
    runtime::put_key(END_DATE, storage::new_uref(end_date).into());

    storage::new_dictionary(CLAIMED).unwrap_or_default();

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
let cliff_amount : U256 = runtime::get_named_arg(CLIFF_AMOUNT);

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
        Parameter::new(CLIFF_AMOUNT, CLType::U256),
    ],
    CLType::URef,
    EntryPointAccess::Public,
    EntryPointType::Contract,
 );  


let claim_entry_point = EntryPoint::new(
    ENTRY_POINT_CLAIM,
    vec![
        Parameter::new(CEP18_CONTRACT_HASH, CLType::Key),
        Parameter::new(RECIPIENT, CLType::Key),
        Parameter::new(CLAIM_AMOUNT, CLType::U256),
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
        CLIFF_TIMESTAMP => cliff_timestamp,
        CLIFF_AMOUNT => cliff_amount
    },
    )
}

// add init function

