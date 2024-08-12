script;

use interfaces::{data_structures::{Asset, PoolId}, mira_amm::MiraAMM};
use math::pool_math::get_deposit_amounts;
use utils::blockchain_utils::{check_deadline, get_lp_asset};
use std::asset::transfer;

configurable {
    AMM_CONTRACT_ID: ContractId = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
}

fn main(
    pool_id: PoolId,
    liquidity: u64,
    amount_0_min: u64,
    amount_1_min: u64,
    recipient: Identity,
    deadline: u32,
) -> (u64, u64) {
    check_deadline(deadline);
    let amm = abi(MiraAMM, AMM_CONTRACT_ID.into());

    let (_, lp_asset_id) = get_lp_asset(pool_id);
    transfer(
        Identity::ContractId(AMM_CONTRACT_ID),
        lp_asset_id,
        liquidity,
    );
    let (amount_0, amount_1) = amm.burn(pool_id, recipient);

    require(amount_0 >= amount_0_min, "Insufficient amount");
    require(amount_1 >= amount_1_min, "Insufficient amount");
    (amount_0, amount_1)
}
