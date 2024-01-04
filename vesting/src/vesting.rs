use core::ops::{ Add, Sub, Mul, Div };

use alloc::{ string::{ String, ToString }, vec::Vec, vec, boxed::Box };

use crate::{
    error::Error,
    interfaces::cep18::CEP18,
    utils::{ self },
    events::VestingEvent,
    events::emit,
};

use casper_types::{
    account::AccountHash,
    U256,
    EntryPoint,
    Key,
    ContractHash,
    EntryPointAccess,
    CLType,
    Parameter,
    EntryPointType,
    EntryPoints,
    contracts::NamedKeys,
    RuntimeArgs,
    runtime_args,
};

use casper_contract::contract_api::{ runtime, storage };

const CONTRACT_NAME: &str = "contract_name";
const VESTING_AMOUNT: &str = "vesting_amount";
const START_DATE: &str = "start_date";
const DURATION: &str = "duration";
const CEP18_CONTRACT_HASH: &str = "cep18_contract_hash";
const OWNER: &str = "owner";

const RECIPIENTS: &str = "recipients";
const ALLOCATIONS: &str = "allocations";
const CLIFF_TIMESTAMP: &str = "cliff_timestamp";
const RELEASE_DATE: &str = "release_date";
const END_DATE: &str = "end_date";
const INDEX: &str = "index";
const RECIPIENT_COUNT: &str = "recipient_count";
const PERIOD: &str = "period";
const RELEASED: &str = "released";
// entry points
const ENTRY_POINT_CLAIM: &str = "claim";
const ENTRY_POINT_INIT: &str = "init";
const ENTRY_POINT_REALASE: &str = "release";

// dicts
const RECIPIENTS_DICT: &str = "recipients_dict";
const ALLOCATIONS_DICT: &str = "allocations_dict";
const CLAIMED_DICT: &str = "claimed_dict";

#[no_mangle]
pub extern "C" fn claim() {
    // let release :bool = utils::read_from(release());

    // if !release {
    //     runtime::revert(Error::ReleaseError);
    // }

    // arguements
    let cep18_contract_hash = runtime
        ::get_named_arg::<Key>(CEP18_CONTRACT_HASH)
        .into_hash()
        .map(ContractHash::new)
        .unwrap();
    let index: i32 = runtime::get_named_arg(INDEX);

    let recipients_dict = *runtime::get_key(RECIPIENTS_DICT).unwrap().as_uref().unwrap();
    let allocations_dict = *runtime::get_key(ALLOCATIONS_DICT).unwrap().as_uref().unwrap();
    let claimed_dict = *runtime::get_key(CLAIMED_DICT).unwrap().as_uref().unwrap();

    let recipient = storage
        ::dictionary_get::<Key>(recipients_dict, &index.to_string())
        .unwrap()
        .unwrap();
    let allocation = storage
        ::dictionary_get::<U256>(allocations_dict, &index.to_string())
        .unwrap()
        .unwrap();
    let claimed = storage
        ::dictionary_get::<U256>(claimed_dict, &index.to_string())
        .unwrap()
        .unwrap();

    // dates
    // let start_date: u64 = utils::read_from(START_DATE);
    let release_date: u64 = utils::read_from(RELEASE_DATE);
    // let end_date: u64 = utils::read_from(END_DATE);
    let duration: u64 = utils::read_from(DURATION);
    let period: u64 = utils::read_from(PERIOD);

    // utils
    let caller = runtime::get_caller();
    // milisecond
    let now: u64 = runtime::get_blocktime().into();

    if recipient != Key::Account(caller) {
        runtime::revert(Error::UserError);
    }

    if release_date.gt(&now) {
        runtime::revert(Error::VestingStartError);
    }

    // dicts
    // let start_date: u64 = utils::read_from(START_DATE);
    let release_date: u64 = utils::read_from(RELEASE_DATE);

    let mut completed_time = now.sub(release_date);

    if completed_time.gt(&duration) {
        completed_time = duration;
    }

    let period_count = duration.div(period);

    let current_period = completed_time.div(period);

    let current_amount = allocation.div(period_count).mul(current_period);

    let claimable_amount = current_amount.sub(claimed);

    if claimable_amount.eq(&U256::zero()) {
        runtime::revert(Error::UnsufficentBalance);
    }

    let cep18 = CEP18::new(cep18_contract_hash);
    cep18.transfer(recipient, claimable_amount);

    storage::dictionary_put(claimed_dict, &index.to_string(), current_amount);

    emit(&(VestingEvent::Claim { cep18_contract_hash, recipient, claim_amount: claimable_amount }));
}

#[no_mangle]
pub extern "C" fn init() {
    check_admin_account();
    let contract_name: String = runtime::get_named_arg(CONTRACT_NAME);
    let vesting_amount: U256 = runtime::get_named_arg(VESTING_AMOUNT);
    let cep18_contract_hash = runtime
        ::get_named_arg::<Key>(CEP18_CONTRACT_HASH)
        .into_hash()
        .map(ContractHash::new)
        .unwrap();

    let start_date: u64 = runtime::get_named_arg(START_DATE);
    let duration: u64 = runtime::get_named_arg(DURATION);
    let period: u64 = runtime::get_named_arg(PERIOD);

    let recipients: Vec<Key> = runtime::get_named_arg(RECIPIENTS);
    let allocations: Vec<U256> = runtime::get_named_arg(ALLOCATIONS);

    let cliff_timestamp: u64 = runtime::get_named_arg(CLIFF_TIMESTAMP);

    let owner: AccountHash = runtime::get_caller().into();

    let release_date: u64 = start_date.add(cliff_timestamp);
    let end_date: u64 = release_date.add(duration);

    let recipients_count = recipients.len().to_string();
    let released = false;

    runtime::put_key(CONTRACT_NAME, storage::new_uref(contract_name).into());
    runtime::put_key(CEP18_CONTRACT_HASH, storage::new_uref(cep18_contract_hash).into());
    runtime::put_key(START_DATE, storage::new_uref(start_date).into());
    runtime::put_key(DURATION, storage::new_uref(duration).into());
    runtime::put_key(PERIOD, storage::new_uref(period).into());
    runtime::put_key(OWNER, storage::new_uref(owner).into());
    runtime::put_key(VESTING_AMOUNT, storage::new_uref(vesting_amount).into());
    runtime::put_key(CLIFF_TIMESTAMP, storage::new_uref(cliff_timestamp).into());
    runtime::put_key(RELEASE_DATE, storage::new_uref(release_date).into());
    runtime::put_key(END_DATE, storage::new_uref(end_date).into());
    runtime::put_key(RECIPIENT_COUNT, storage::new_uref(recipients_count).into());
    runtime::put_key(RELEASED, storage::new_uref(released).into());

    storage::new_dictionary(RECIPIENTS_DICT).unwrap_or_default();
    storage::new_dictionary(ALLOCATIONS_DICT).unwrap_or_default();
    storage::new_dictionary(CLAIMED_DICT).unwrap_or_default();

    let recipients_dict = *runtime::get_key(RECIPIENTS_DICT).unwrap().as_uref().unwrap();
    let allocations_dict = *runtime::get_key(ALLOCATIONS_DICT).unwrap().as_uref().unwrap();
    let claimed_dict = *runtime::get_key(CLAIMED_DICT).unwrap().as_uref().unwrap();

    for (index, recipient) in recipients.into_iter().enumerate() {
        storage::dictionary_put(recipients_dict, &index.to_string(), recipient);
    }

    for (index, allocation) in allocations.into_iter().enumerate() {
        storage::dictionary_put(allocations_dict, &index.to_string(), allocation);
        storage::dictionary_put(claimed_dict, &index.to_string(), U256::zero());
    }
}

#[no_mangle]
pub extern "C" fn release() {
    // check erc20 balance
    let caller: AccountHash = runtime::get_caller().into();
    let owner: AccountHash = utils::read_from(OWNER);

    if owner.to_string() != caller.to_string() {
        runtime::revert(Error::UnsufficentBalance);
    }

    let relased_result = true;

    runtime::put_key(RELEASED, storage::new_uref(relased_result).into());
}

pub fn check_admin_account() {
    let admin: AccountHash = utils::get_key(OWNER);
    let caller = runtime::get_caller();
    if admin != caller {
        runtime::revert(Error::AdminError);
    }
}

#[no_mangle]
pub extern "C" fn call() {
    let contract_name: String = runtime::get_named_arg(CONTRACT_NAME);
    let vesting_amount: U256 = runtime::get_named_arg(VESTING_AMOUNT);
    let cep18_contract_hash = runtime::get_named_arg::<Key>(CEP18_CONTRACT_HASH);

    let start_date: u64 = runtime::get_named_arg(START_DATE);
    let duration: u64 = runtime::get_named_arg(DURATION);
    let period: u64 = runtime::get_named_arg(PERIOD);

    let recipients: Vec<Key> = runtime::get_named_arg(RECIPIENTS);
    let allocations: Vec<U256> = runtime::get_named_arg(ALLOCATIONS);

    let cliff_timestamp: u64 = runtime::get_named_arg(CLIFF_TIMESTAMP);

    let init_entry_point = EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![
            Parameter::new(CONTRACT_NAME, CLType::String),
            Parameter::new(VESTING_AMOUNT, CLType::U256),
            Parameter::new(CEP18_CONTRACT_HASH, CLType::Key),
            Parameter::new(START_DATE, CLType::U64),
            Parameter::new(DURATION, CLType::U64),
            Parameter::new(PERIOD, CLType::U64),
            Parameter::new(RECIPIENTS, CLType::List(Box::new(CLType::Key))),
            Parameter::new(ALLOCATIONS, CLType::List(Box::new(CLType::U256))),
            Parameter::new(CLIFF_TIMESTAMP, CLType::U64)
        ],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let claim_entry_point = EntryPoint::new(
        ENTRY_POINT_CLAIM,
        vec![Parameter::new(CEP18_CONTRACT_HASH, CLType::Key), Parameter::new(INDEX, CLType::I32)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let release_entry_point = EntryPoint::new(
        ENTRY_POINT_REALASE,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(init_entry_point);
    entry_points.add_entry_point(claim_entry_point);
    entry_points.add_entry_point(release_entry_point);

    let named_keys = NamedKeys::new();

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
        Some(uref_name.to_string())
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
        PERIOD => period,
        RECIPIENTS => recipients,
        ALLOCATIONS => allocations,
        CLIFF_TIMESTAMP => cliff_timestamp
    }
    )
}
