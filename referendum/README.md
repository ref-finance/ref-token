# Referendum

Detailed introduction to Referendum, see this [post](https://gov.ref.finance/).

## Instructions

### Initialization
After deploy, we can initiate the contract as following:
```bash
near call $REFERENDUM new '{"owner_id": "'$OWNER'", "token_id": "'$TOKEN'"}' --account_id $REFERENDUM
```
Then owner can determine a launch date and set it into contract in 30 days after deployment:
```bash
# set 2022-02-01 00:00:00 UTC to be genesis time
near call $REFERENDUM modify_genesis_timestamp '{"genesis_timestamp_in_sec": 1643673600}' --account_id $OWNER
```

### Owner Methods
Ownership can be transfered:
```bash
near view $REFERENDUM get_owner
near call $REFERENDUM set_owner '{"owner_id": "'$NEW_OWNER'"}' --account_id $OWNER
```
Owner can set launch date, see Initialization for details.   

Owner can modify endorsement NEAR amount per proposal:
```bash
# set endorsement NEAR amount to 15 NEAR.
near call $REFERENDUM modify_endorsement_amount '{"amount": "15'$ZERO24'"}' --account_id $OWNER
```

Owner can set nonsense threshold:
```bash
# set threshold to 40%
near call $REFERENDUM modify_nonsense_threshold '{"threshold": {"numerator": 40, "denominator": 100}}' --account_id $OWNER
```

Owner can set vote policy:
```bash
# set relative policy to 30% voting ballot and 50%+ supported opinion wins
near call $REFERENDUM modify_vote_policy '{"vote_policy": {"Relative": [{"numerator": 30, "denominator": 100}, {"numerator": 1, "denominator": 2}]}}' --account_id $OWNER

# set absolute policy to pass with 55%+ ballot power and fail with 45%+ ballot power
near call $REFERENDUM modify_vote_policy '{"vote_policy": {"Absolute": [{"numerator": 55, "denominator": 100}, {"numerator": 45, "denominator": 100}]}}' --account_id $OWNER
```

### Proposer and Proposals
Anyone can initiate a referendum proposal with fixed amount of NEAR as endorsement:
```bash
# alice.near deposit 15 NEAR as endorsement to create a referendum,
# referendum will start at 7 days (604800) after beginning of session 0, and lasts 14 days (1209600),
# The vote policy is Absolute (the other is Relative),
# Currently there is only one kind of referendum, Vote.
near call $REFERENDUM add_proposal '{"description": "example referendum, see detail at https://xxxxxxx", "kind": "Vote", "policy_type": "Absolute", "session_id": 0, "start_offset_sec": 604800, "lasts_sec": 1209600}' --account_id=alice.near --amount 15
```
The deposit NEAR would lock until the referendum goes to a final state, that is one of Approved, Rejected, Nonsense or Expired.

On approved and Rejected state, the locked NEAR would auto transfer back to proposer;
On nonsense state, the locked NEAR would be slashed;
On expired state, proposer need to explicit call to redeem the locked NEAR:
```bash
# it returns true when succeed
near call $REFERENDUM redeem_near_in_expired_proposal '{"id": 0}' --account_id=alice.near
```

The proposer can also remove his proposal before it starts and gets locked NEAR back:
```bash
# it returns true when succeed
near call $REFERENDUM remove_proposal '{"id": 0}'
```

### User Register
This contract obeys NEP-145 to manage storage, but choose a fixed storage fee policy in this contract. Each user only needs deposit to lock a fixed 0.01 NEAR as storage cost.

Detailed interface description could be found at [NEP-145](https://nomicon.io/Standards/StorageManagement.html).

Here we only list some common-use interfaces:

* `storage_deposit`, to register a user,
* `storage_unregister`, to unregister caller self and get 0.01 NEAR back,
* `storage_balance_of`, to get given user storage balance.


### User Methods

### View Methods

## Development
1. Install `rustup` via [https://rustup.rs/](https://rustup.rs/)
2. Run the following:
```bash
rustup default stable
rustup target add wasm32-unknown-unknown
```
### Compiling
You can build release version by running script:
```
./build_release.sh
```
### Testing
Contract has unit tests as well as simulation tests. All together can be run:
```bash
cargo test -- --nocapture
``` 
### Deploying to TestNet
To deploy to TestNet, you can use:
```bash
near dev-deploy ../res/referendum.wasm
```
This will output on the contract ID it deployed.
