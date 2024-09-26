library;

use std::block::height;
use interfaces::{asset_wrapper::AssetWrapper, data_structures::PoolId};
use std::hash::*;

/// Validates that the provided deadline hasn't passed yet
pub fn check_deadline(deadline: u32) {
    require(deadline >= height(), "Deadline passed");
}

/// Builds and returns an LP sub id and asset id for the provided pool id
pub fn get_lp_asset(contract_id: ContractId, pool_id: PoolId) -> (b256, AssetId) {
    let lp_sub_id = sha256(pool_id);
    (lp_sub_id, AssetId::new(contract_id, lp_sub_id))
}

pub fn is_stable(pool_id: PoolId) -> bool {
    pool_id.2
}

/// Checks if the provided asset is a wrapped asset and wraps the provided amount of it in that case
pub fn wrap_if_needed(
    asset_wrapper_contract: ContractId,
    asset_id: AssetId,
    amount: u64,
) {
    let asset_wrapper = abi(AssetWrapper, asset_wrapper_contract.into());
    match asset_wrapper.get_underlying_asset(asset_id) {
        Some(underlying_asset) => {
            asset_wrapper
                .wrap {
                    asset_id: underlying_asset.into(),
                    coins: amount,
                }();
        },
        None => {}
    };
}

/// Checks if the provided asset is a wrapped asset and unwraps the provided amount of it in that case
pub fn unwrap_if_needed(
    asset_wrapper_contract: ContractId,
    asset_id: AssetId,
    amount: u64,
) {
    let asset_wrapper = abi(AssetWrapper, asset_wrapper_contract.into());
    match asset_wrapper.get_underlying_asset(asset_id) {
        Some(_) => {
            asset_wrapper
                .unwrap {
                    asset_id: asset_id.into(),
                    coins: amount,
                }();
        },
        None => {}
    };
}
