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
    name: String::from("Ref Finance"),
    symbol: String::from("REF"),
    icon: Some(String::from("https://media.discordapp.net/attachments/857712764562309121/861781753596870676/reffi-stack.png")),
    reference: None,
    reference_hash: None,
    decimals: 18,
}
```

### initialize
```shell
near call $TOKEN_ID new '{"owner": "aaa", "total_supply": "100000000000000000000000000"}' --account_id=$TOKEN_ID
```
