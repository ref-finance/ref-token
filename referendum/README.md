# Referendum

Detailed discussion about Referendum, see this [post](https://gov.ref.finance/t/product-x-referendum/384).

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
To lock token (XREF) and get ballot power, user need start from token contract:
```bash
# alice lock 100 TOKEN for 9 sessions
near call $TOKEN ft_tranfser_call '{"receiver_id": "'$REFERENDUM'", "amount": "100'$ZERO18'", "msg": "9"}' --account_id=alice.near --amount=$YN
```
*Note: user can only start a new lock when there is no existing locking at his account.*   
  
When there is an existing locking, user can append token to it:
```bash
# alice append 100 TOKEN to his existing locking
near call $TOKEN ft_tranfser_call '{"receiver_id": "'$REFERENDUM'", "amount": "100'$ZERO18'", "msg": ""}' --account_id=alice.near --amount=$YN
```
*Note: user can only append lock when there is a existing locking at his account.*   

To withdraw token when they are unlocked, user call:
```bash
near call $REFERENDUM withdraw --account_id=alice.near --amount=$YN
```
*Note: If user wanna those token to be part of a new locking, he can directly start `ft_transfer_call` without withdrawing them first. those un-withdraw amount would auto caculate into the total locking amount.* 

User can vote any InProgress referendum:
```bash
# action could be one of VoteApprove, VoteReject, VoteNonsense
near call $REFERENDUM act_proposal '{"id": 0, "action": "VoteApprove"}' --account_id=alice.near
```
*Note: user can only vote once per proposal and the ballot power would auto renew if user append locking more token and get more ballot.*  

### View Methods
#### **To view contract info:**
```bash
near view $REFERENDUM contract_metadata
```
The return value structure is:
```rust
pub struct ContractMetadata {
    /// the owner account id of contract
    pub owner_id: AccountId,
    /// accept lock token account id
    pub locked_token: AccountId,
    /// the launch timestamp in seconds
    pub genesis_timestamp_sec: u32,
    /// current session id (start from 0)
    pub cur_session_id: u32,
    /// current total ballot amount (calculate at call time)
    pub cur_total_ballot: U128,
    /// current locking token amount (include those expired but hasn't unlock by user)
    pub cur_lock_amount: U128,
    /// the availabe proposal id for new proposal
    pub last_proposal_id: u32,
    /// lock near amount for endorsement a proposal
    pub lock_amount_per_proposal: U128,
    /// current account number in contract
    pub account_number: u64,
    /// a list of [Relative, Absolute] in which each item is formated as 
    /// [{"numerator": n, "denominator": m}, {"numerator": n, "denominator": m}]
    pub vote_policy: Vec<VotePolicy>,
    /// in format as {"numerator": n, "denominator": m}
    pub nonsense_threshold: Rational,
}
```

#### **To view proposal info:**

```bash
# returns `ProposalInfo` structure or null
near view $REFERENDUM get_proposal_info '{"proposal_id": 0}'

# returns array of `ProposalInfo`
near view $REFERENDUM get_proposals_in_session '{"session_id": 0}'

# returns array of proposal id
near view $REFERENDUM get_proposal_ids_in_session '{"session_id": 0}'
```
The `ProposalInfo` structure is:
```rust
pub struct ProposalInfo{
    pub id: u32,
    pub proposer: AccountId,
    /// near amount for endorsement
    pub lock_amount: U128,
    pub description: String,
    /// one of the following:
    /// "VotePolicy": {"Relative": [{"numerator": n, "denominator": m}, {"numerator": n, "denominator": m}]}
    /// "VotePolicy": {"Absolute": [{"numerator": n, "denominator": m}, {"numerator": n, "denominator": m}]}
    pub vote_policy: proposals::VotePolicy,
    /// currently would only be "Vote"
    pub kind: proposals::ProposalKind,
    /// one of the following:
    /// "WarmUp", "InProgress", "Approved", "Rejected", "Nonsense", "Expired"
    pub status: proposals::ProposalStatus,
    /// [Approve_count, Reject_count, Nonsense_count, Total_ballots]
    pub vote_counts: [U128; 4],
    /// The session this proposal is valid in
    pub session_id: u32,
    /// the start time = session_begin_time + start_offset
    pub start_offset_sec: u32,
    /// the proposal max valid period in seconds
    pub lasts_sec: u32,
}
```
#### **To get account info**
For basic account info:
```bash
# return `AccountInfo` or null
near view $REFERENDUM get_account_info '{"account_id": "alice.near"}'
```
The `AccountInfo` is:
```rust
pub struct AccountInfo {
    /// locked token (XREF) amount
    pub locking_amount: U128,
    /// ballot amount (calculate at call time)
    pub ballot_amount: U128,
    /// unlock at the begin of this session, meanwhile ballots reset to zero
    pub unlocking_session_id: u32,
}
```
For account votes:
```bash
# return array of `HumanReadableAccountVote`
near view $REFERENDUM get_account_proposals_in_session '{"account_id": "alice.near", "session_id": 0}'
```
The `HumanReadableAccountVote` is:
```rust
pub struct HumanReadableAccountVote {
    pub proposal_id: u32,
    pub vote: Vote,
    pub amount: U128,
}
```

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
