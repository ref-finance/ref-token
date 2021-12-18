use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};
use ref_token::ContractContract as RefToken;
use xref_token::ContractContract as XRefToken;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    REF_WASM_BYTES => "../res/ref_token_release.wasm",
    XREF_WASM_BYTES => "../res/xref_token_local.wasm",
}

pub fn init_env(register_user: bool) -> (UserAccount, UserAccount, UserAccount, ContractAccount<RefToken>, ContractAccount<XRefToken>){
    let root = init_simulator(None);

    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let user = root.create_user("user".to_string(), to_yocto("100"));

    let ref_contract = deploy!(
        contract: RefToken,
        contract_id: "ref",
        bytes: &REF_WASM_BYTES,
        signer_account: root
    );
    call!(root, ref_contract.new(owner.valid_account_id(), to_yocto("1000000").into())).assert_success();
    call!(user, ref_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(user, ref_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    
    call!(owner, ref_contract.ft_transfer(user.valid_account_id(), to_yocto("100").into(), None), deposit = 1).assert_success();

    let xref_contract = deploy!(
        contract: XRefToken,
        contract_id: "xref",
        bytes: &XREF_WASM_BYTES,
        signer_account: root
    );
    call!(root, xref_contract.new(owner.valid_account_id(), ref_contract.valid_account_id())).assert_success();
    call!(root, ref_contract.storage_deposit(Some(xref_contract.valid_account_id()), None), deposit = to_yocto("1")).assert_success();
    if register_user {
        call!(user, xref_contract.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    }
    (root, owner, user, ref_contract, xref_contract)
}