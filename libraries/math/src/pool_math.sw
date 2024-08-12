library;

use interfaces::{data_structures::{Asset, PoolId, PoolMetadata}, mira_amm::MiraAMM};
use std::{math::*, primitive_conversions::u64::*};

const ONE_E_18: u256 = 1_000_000_000_000_000_000;
const BASIS_POINTS_DENOMINATOR: u256 = 10_000;

// TODO: find a standard library function for that
fn pow_decimals(decimals: u8) -> u256 {
    let mut res: u256 = 1;
    let mut i: u8 = 0;
    while i < decimals {
        res *= 10;
        i += 1;
    }
    res
}

fn _k(
    is_stable: bool,
    x: u64,
    y: u64,
    pow_decimals_x: u256,
    pow_decimals_y: u256,
) -> u256 {
    if (is_stable) {
        let _x: u256 = x.as_u256() * ONE_E_18 / pow_decimals_x;
        let _y: u256 = y.as_u256() * ONE_E_18 / pow_decimals_y;
        let _a: u256 = (_x * _y) / ONE_E_18;
        let _b: u256 = ((_x * _x) / ONE_E_18 + (_y * _y) / ONE_E_18);
        return _a * _b / ONE_E_18; // x3y+y3x >= k
    } else {
        return x.as_u256() * y.as_u256(); // xy >= k
    }
}

fn _f(x_0: u256, y: u256) -> u256 {
    x_0 * (y * y / ONE_E_18 * y / ONE_E_18) / ONE_E_18 + (x_0 * x_0 / ONE_E_18 * x_0 / ONE_E_18) * y / ONE_E_18
}

fn _d(x_0: u256, y: u256) -> u256 {
    0x3u256 * x_0 * (y * y / ONE_E_18) / ONE_E_18 + (x_0 * x_0 / ONE_E_18 * x_0 / ONE_E_18)
}

fn _get_y(x_0: u256, xy: u256, y: u256) -> u256 {
    let mut y: u256 = y;
    let mut i = 0;
    while i < 255 {
        let y_prev = y;
        let k = _f(x_0, y);
        if k < xy {
            let dy = (xy - k) * ONE_E_18 / _d(x_0, y);
            y = y + dy;
        } else {
            let dy = (k - xy) * ONE_E_18 / _d(x_0, y);
            y = y - dy;
        }
        if y > y_prev {
            if y - y_prev <= 0x1u256 {
                return y;
            }
        } else {
            if y_prev - y <= 0x1u256 {
                return y;
            }
        }
        i += 1;
    }
    y
}

pub fn max(a: u64, b: u64) -> u64 {
    if a > b { a } else { b }
}

pub fn calculate_fee(amount: u64, fee: u64) -> u64 {
    let fee = u64::try_from(amount.as_u256() * fee.as_u256() / BASIS_POINTS_DENOMINATOR).unwrap();
    max(1, fee)
}

fn adjust_for_fee(amount: u64, fee: u64) -> u64 {
    amount - calculate_fee(amount, fee)
}

fn is_stable(pool_id: PoolId) -> bool {
    pool_id.2
}

fn get_amount_out(pool_id: PoolId, pool: PoolMetadata, amount_in: u64, asset_in: AssetId, stable_fee: u64, volatile_fee: u64) -> u64 {
    let amount_out: u256 = if is_stable(pool_id) {
        let (pow_decimals_0, pow_decimals_1) = (pow_decimals(pool.decimals_0), pow_decimals(pool.decimals_1));
        let xy: u256 = _k(true, pool.reserve_0, pool.reserve_1, pow_decimals_0, pow_decimals_1);
        let _reserve_0: u256 = pool.reserve_0.as_u256() * ONE_E_18 / pow_decimals_0;
        let _reserve_1: u256 = pool.reserve_1.as_u256() * ONE_E_18 / pow_decimals_1;
        let (reserve_in, reserve_out, decimals_in, decimals_out) = if asset_in == pool_id.0 { (_reserve_0, _reserve_1, pow_decimals_0, pow_decimals_1) } else { (_reserve_1, _reserve_0, pow_decimals_1, pow_decimals_0) };
        let amount_in_with_fee = adjust_for_fee(amount_in, stable_fee);
        let amount_in_adjusted: u256 = amount_in_with_fee.as_u256() * ONE_E_18 / decimals_in;
        let y: u256 = reserve_out - _get_y(amount_in_adjusted + reserve_in, xy, reserve_out);
        y * decimals_out / ONE_E_18
    } else {
        let amount_in_with_fee = adjust_for_fee(amount_in, volatile_fee);
        let (reserve_in, reserve_out) = if asset_in == pool_id.0 { (pool.reserve_0, pool.reserve_1) } else { (pool.reserve_1, pool.reserve_0) };
        amount_in_with_fee.as_u256() * reserve_out.as_u256() / (reserve_in.as_u256() + amount_in_with_fee.as_u256())
    };
    u64::try_from(amount_out).unwrap()
}

pub fn get_amounts_out(amm_contract: ContractId, amount_in: u64, asset_in: AssetId, pools: Vec<PoolId>) -> Vec<(u64, AssetId)> {
    require(pools.len() >= 1, "Router: INVALID_PATH");

    let amm = abi(MiraAMM, amm_contract.into());
    let (lp_fee_volatile, lp_fee_stable, protocol_fee_volatile, protocol_fee_stable) = amm.fees();
    let (stable_fee, volatile_fee) = (lp_fee_stable + protocol_fee_stable, lp_fee_volatile + protocol_fee_volatile);
    
    let mut amounts: Vec<(u64, AssetId)> = Vec::new();
    amounts.push((amount_in, asset_in));
    let mut i = 0;
    while (i < pools.len()) {
        let pool_id = pools.get(i).unwrap();
        let pool = amm.pool_metadata(pool_id);
        require(pool.is_some(), "Pool not present");
        let (amount_in, asset_in) = amounts.get(i).unwrap();
        let amount_out = get_amount_out(pool_id, pool.unwrap(), amount_in, asset_in, stable_fee, volatile_fee);
        let asset_out = if pool_id.0 == asset_in { pool_id.1 } else { pool_id.0 };
        amounts.push((amount_out, asset_out));
        i += 1;
    }
    amounts
}
