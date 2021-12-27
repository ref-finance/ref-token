use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};
use test_token::ContractContract as TestToken;
use referendum::ContractContract as Referendum;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TEST_WASM_BYTES => "../res/test_token.wasm",
    REFERENDUM_WASM_BYTES => "../res/referendum_local.wasm",
}



pub fn init_env(register_user: bool) -> (UserAccount, UserAccount, UserAccount, ContractAccount<TestToken>, ContractAccount<Referendum>) {
    let root = init_simulator(None);

    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let user = root.create_user("user".to_string(), to_yocto("100"));

    let xref_contract = deploy!(
        contract: TestToken,
        contract_id: "xref",
        bytes: &TEST_WASM_BYTES,
        signer_account: root
    );
    call!(root, xref_contract.new("xref".to_string(), "xref".to_string(), 18)).assert_success();
    call!(owner, xref_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(user, xref_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();

    call!(root, xref_contract.mint(owner.valid_account_id(), to_yocto("10000").into())).assert_success();
    call!(root, xref_contract.mint(user.valid_account_id(), to_yocto("100").into())).assert_success();

    let referendum_contract = deploy!(
        contract: Referendum,
        contract_id: "referendum",
        bytes: &REFERENDUM_WASM_BYTES,
        signer_account: root
    );
    call!(root, referendum_contract.new(owner.valid_account_id(), xref_contract.valid_account_id())).assert_success();
    call!(root, xref_contract.storage_deposit(Some(referendum_contract.valid_account_id()), None), deposit = to_yocto("1")).assert_success();
    if register_user {
        let current_timestamp = root.borrow_runtime().current_block().block_timestamp;
        call!(
            owner,
            referendum_contract.modify_genesis_timestamp(current_timestamp)
        )
        .assert_success();
        
        call!(user, referendum_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    }
    (root, owner, user, xref_contract, referendum_contract)
}