use elrond_wasm::types::{Address, ManagedAddress, TokenIdentifier};
use elrond_wasm_debug::{rust_biguint, testing_framework::*, DebugApi};
use router_sc::*;

const WASM_PATH: &'static str = "output/router-sc.wasm";

struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> router_sc::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_a_address: Address,
    pub user_b_address: Address,
    pub contract_wrapper: ContractObjWrapper<router_sc::ContractObj<DebugApi>, ContractObjBuilder>,
}

fn setup_contract<ContractObjBuilder>(
    cf_builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> router_sc::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let user_a_address = blockchain_wrapper.create_user_account(&rust_zero);
    let user_b_address = blockchain_wrapper.create_user_account(&rust_zero);
    let cf_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        cf_builder,
        WASM_PATH,
    );

    blockchain_wrapper
        .execute_tx(&owner_address, &cf_wrapper, &rust_zero, |sc| {
            sc.init();
        })
        .assert_ok();

    blockchain_wrapper.add_mandos_set_account(cf_wrapper.address_ref());

    ContractSetup {
        blockchain_wrapper,
        owner_address,
        user_a_address,
        user_b_address,
        contract_wrapper: cf_wrapper,
    }
}

const ECITY_TOKEN_ID: &[u8] = b"ECITY-123456";

#[test]
fn deploy_test() {
    let mut setup = setup_contract(router_sc::contract_obj);

    // simulate deploy
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.init();
            },
        )
        .assert_ok();
}

#[test]
fn add_distribution_test() {
    let mut setup = setup_contract(router_sc::contract_obj);
    let setup2 = setup_contract(router_sc::contract_obj);

    // Sets the distribution to 100% for user_a
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.add_distribution(
                    ManagedAddress::from(setup2.user_a_address),
                    10000,
                );
            },
        )
        .assert_ok();
}

#[test]
fn remove_distribution_test() {
    let mut setup = setup_contract(router_sc::contract_obj);
    let setup2 = setup_contract(router_sc::contract_obj);
    let setup3 = setup_contract(router_sc::contract_obj);

    // simulate deploy
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.init();
            },
        )
        .assert_ok();

    // Sets the distribution to 100% for user_a
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.add_distribution(
                    ManagedAddress::from(setup2.user_a_address),
                    10000,
                );
            },
        )
        .assert_ok();

    // Removes the distribution for user_a
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.remove_distribution(
                    ManagedAddress::from(setup3.user_a_address),
                );
            },
        )
        .assert_ok();
}

#[test]
fn add_token_test() {
    let mut setup = setup_contract(router_sc::contract_obj);

    // Sets the token identifier of the token to be distributed
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.set_token(
                    TokenIdentifier::from("ECITY-123456"),
                );
            },
        )
        .assert_ok();
}

#[test]
fn set_all_and_distribute_test() {
    let mut setup = setup_contract(router_sc::contract_obj);
    let setup2 = setup_contract(router_sc::contract_obj);
    let setup3 = setup_contract(router_sc::contract_obj);

    // Sets the token identifier of the token to be distributed
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.set_token(
                    TokenIdentifier::from(ECITY_TOKEN_ID),
                );
            },
        )
        .assert_ok();

    // Sets the owner's balance to 1000 tokens
    setup
        .blockchain_wrapper
        .set_esdt_balance(&setup.owner_address, ECITY_TOKEN_ID, &rust_biguint!(1000u64));

    // Sets the distribution to 50% for user_a
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.add_distribution(
                    ManagedAddress::from(setup2.user_a_address),
                    5000,
                );
            },
        )
        .assert_ok();

    // Sets the distribution to 50% for user_b
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.add_distribution(
                    ManagedAddress::from(setup3.user_b_address),
                    5000,
                );
            },
        )
        .assert_ok();

    // Distributes the tokens to the users
    setup
        .blockchain_wrapper
        .execute_esdt_transfer(
            &setup.owner_address,
            &setup.contract_wrapper,
            ECITY_TOKEN_ID,
            0u64,
            &rust_biguint!(1000u64),
            |sc| {
                sc.distribute();
            },
        )
        .assert_ok();

    // Checks the contract's balance
    setup
        .blockchain_wrapper
        .check_esdt_balance(&setup.contract_wrapper.address_ref(), ECITY_TOKEN_ID, &rust_biguint!(0u64));

    // Checks user_a's balance
    setup
        .blockchain_wrapper
        .check_esdt_balance(&setup.user_a_address, ECITY_TOKEN_ID, &rust_biguint!(500u64));

    // Checks user_b's balance
    setup
        .blockchain_wrapper
        .check_esdt_balance(&setup.user_b_address, ECITY_TOKEN_ID, &rust_biguint!(500u64));
}

#[test]
fn more_than_100_percent_test() {
    let mut setup = setup_contract(router_sc::contract_obj);
    let setup2 = setup_contract(router_sc::contract_obj);
    let setup3 = setup_contract(router_sc::contract_obj);

    // Sets the distribution to 100% for user_a
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.add_distribution(
                    ManagedAddress::from(setup2.user_a_address),
                    10000,
                );
            },
        )
        .assert_ok();

    // Sets the distribution to 100% for user_b
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.add_distribution(
                    ManagedAddress::from(setup3.user_b_address),
                    10000,
                );
            },
        )
        .assert_user_error("Total percentage must be less than or equal to 10000");
}

#[test]
fn lock_and_change() {
    let mut setup = setup_contract(router_sc::contract_obj);
    let setup2 = setup_contract(router_sc::contract_obj);
    let setup3 = setup_contract(router_sc::contract_obj);

    // Sets the distribution to 100% for user_a
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.add_distribution(
                    ManagedAddress::from(setup2.user_a_address),
                    10000,
                );
            },
        )
        .assert_ok();

    // Locks the contract
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.lock_distribution();
            },
        )
        .assert_ok();

    // Sets the distribution to 100% for user_b
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.contract_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.add_distribution(
                    ManagedAddress::from(setup3.user_b_address),
                    10000,
                );
            },
        )
        .assert_user_error("Distribution is locked");
}