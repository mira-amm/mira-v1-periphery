use fuels::{
    prelude::{AssetId, ContractId, WalletUnlocked},
    types::{input::Input, output::Output},
};

use super::interface::MiraAMM;

pub type PoolId = (AssetId, AssetId);

const AMOUNT_PER_COIN: u64 = 1_000_000;
const COINS_PER_ASSET: u64 = 100;
const DEADLINE: u32 = 1000;
const DEPOSIT_AMOUNTS: (u64, u64) = (10000, 40000);
const LIQUIDITY: u64 = 20000;
pub const NUMBER_OF_ASSETS: u64 = 5;

pub fn build_pool_id(asset_a: AssetId, asset_b: AssetId) -> PoolId {
    if asset_a < asset_b {
        (asset_a, asset_b)
    } else {
        (asset_b, asset_a)
    }
}

pub struct MiraAMMContract {
    pub id: ContractId,
    pub instance: MiraAMM<WalletUnlocked>,
}

pub struct LiquidityParameters {
    pub amounts: (u64, u64),
    pub deadline: u32,
    pub liquidity: u64,
}

pub struct TransactionParameters {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

pub struct WalletAssetConfiguration {
    pub number_of_assets: u64,
    pub coins_per_asset: u64,
    pub amount_per_coin: u64,
}

impl LiquidityParameters {
    pub fn new(amounts: Option<(u64, u64)>, deadline: Option<u32>, liquidity: Option<u64>) -> Self {
        Self {
            amounts: amounts.unwrap_or(DEPOSIT_AMOUNTS),
            deadline: deadline.unwrap_or(DEADLINE),
            liquidity: liquidity.unwrap_or(LIQUIDITY),
        }
    }
}

pub struct SwapParameters {
    pub amount: u64,
    pub route_length: u64,
}

pub struct SwapResult {
    pub actual: u64,
    pub expected: Option<u64>,
}

impl Default for WalletAssetConfiguration {
    fn default() -> Self {
        Self {
            number_of_assets: NUMBER_OF_ASSETS,
            coins_per_asset: COINS_PER_ASSET,
            amount_per_coin: AMOUNT_PER_COIN,
        }
    }
}
