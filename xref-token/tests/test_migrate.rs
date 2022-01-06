
use near_sdk_sim::{deploy, view, init_simulator, to_yocto};

use xref_token::{ContractContract as Xref, ContractMetadata};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PREV_XREF_WASM_BYTES => "../res/xref_token_release.wasm",
    XREF_WASM_BYTES => "../res/xref_token_release.wasm",
}

#[test]
fn test_upgrade() {
    let root = init_simulator(None);
    let test_user = root.create_user("test".to_string(), to_yocto("100"));
    let xref = deploy!(
        contract: Xref,
        contract_id: "xref".to_string(),
        bytes: &PREV_XREF_WASM_BYTES,
        signer_account: root,
        init_method: new(root.valid_account_id(), root.valid_account_id())
    );
    // Failed upgrade with no permissions.
    let result = test_user
        .call(
            xref.user_account.account_id.clone(),
            "upgrade",
            &XREF_WASM_BYTES,
            near_sdk_sim::DEFAULT_GAS,
            0,
        )
        .status();
    assert!(format!("{:?}", result).contains("ERR_NOT_ALLOWED"));

    root.call(
        xref.user_account.account_id.clone(),
        "upgrade",
        &XREF_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
    .assert_success();
    let metadata = view!(xref.contract_metadata()).unwrap_json::<ContractMetadata>();
    // println!("{:#?}", metadata);
    assert_eq!(metadata.version, "1.0.1".to_string());

    // Upgrade to the same code migration is skipped.
    root.call(
        xref.user_account.account_id.clone(),
        "upgrade",
        &XREF_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
    .assert_success();
}