# Ref Token Contract

### Compiling

You can build release version by running next scripts inside each contract folder:

```
./build.sh
```

### Deploying to TestNet

To deploy to TestNet, you can use next command:
```
near dev-deploy
```

This will output on the contract ID it deployed.

### Metadata
```rust
FungibleTokenMetadata {
    spec: FT_METADATA_SPEC.to_string(),
    name: String::from("Ref Finance Token"),
    symbol: String::from("REF"),
    // see code for the detailed icon content
    icon: Some(String::from("data:image/svg+xml;base64,PHN2ZyBpZD0......Zz4=")),
    reference: None,
    reference_hash: None,
    decimals: 18,
}
```

### initialize
```shell
near call $TOKEN_ID new '{"owner": "u1.testnet", "total_supply": "100000000000000000000000000"}' --account_id=$TOKEN_ID
```
