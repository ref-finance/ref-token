# Release Notes

### Version 1.0.3
```bash=
# codehash: 5g2CDRuEV1s3UpVfuaNgGiiX4vwHaaLBwZ7ygnePDPPE 
```
1. add ft_mint and ft_burn event;

### Version 1.0.2
```bash=
# codehash: ErUzfYKkbsiZtKWf9JKV1UvztNZvXjoFpdzB6wvB8vjX 
```
1. add account counter;
2. change token icon;

### Version 1.0.1
1. let owner choose whether or not to sync up distribution before update to the new `reward_per_sec` when `modify_reward_per_sec`;
2. add two fields to show current undistributed reward and staked token amount (by calculation at call time) in `ContractMetadata`;
3. change field `prev_distribution_time: u64` to `prev_distribution_time_in_sec: u32` in `ContractMetadata`;
4. add reward_genesis_time;