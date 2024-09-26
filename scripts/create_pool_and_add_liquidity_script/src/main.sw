script;

use interfaces::{data_structures::{Asset, PoolId}, mira_amm::MiraAMM};
use std::asset::transfer;
use utils::blockchain_utils::{check_deadline, wrap_if_needed};

configurable {
    AMM_CONTRACT_ID: ContractId = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
    ASSET_WRAPPER_CONTRACT_ID: ContractId = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
}

fn main(
    token_0_contract_id: ContractId,
    token_0_sub_id: b256,
    token_1_contract_id: ContractId,
    token_1_sub_id: b256,
    is_stable: bool,
    amount_0_desired: u64,
    amount_1_desired: u64,
    recipient: Identity,
    deadline: u32,
) -> Asset {
    check_deadline(deadline);
    let amm = abi(MiraAMM, AMM_CONTRACT_ID.into());

    let pool_id = amm.create_pool(
        token_0_contract_id,
        token_0_sub_id,
        token_1_contract_id,
        token_1_sub_id,
        is_stable,
    );

    wrap_if_needed(ASSET_WRAPPER_CONTRACT_ID, pool_id.0, amount_0_desired);
    wrap_if_needed(ASSET_WRAPPER_CONTRACT_ID, pool_id.1, amount_1_desired);
    transfer(
        Identity::ContractId(AMM_CONTRACT_ID),
        pool_id.0,
        amount_0_desired,
    );
    transfer(
        Identity::ContractId(AMM_CONTRACT_ID),
        pool_id.1,
        amount_1_desired,
    );

    amm.mint(pool_id, recipient)
}
