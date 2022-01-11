# Release Notes

### Version 1.0.1
1. let owner choose whether or not to sync up distribution before update to the new `reward_per_sec` when `modify_reward_per_sec`;
2. add two fields to show current undistributed reward and staked token amount (by calculation at call time) in `ContractMetadata`;
3. change field `prev_distribution_time: u64` to `prev_distribution_time_in_sec: u32` in `ContractMetadata`;
4. add reward_genesis_time;