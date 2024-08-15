use std::cmp::min;
use std::str::FromStr;

use fuels::accounts::provider::Provider;
use fuels::accounts::wallet::WalletUnlocked;
use fuels::types::ContractId;
use test_harness::data_structures::{
    MiraAMMContract, TransactionParameters, WalletAssetConfiguration,
};
use test_harness::interface::amm::create_pool;
use test_harness::interface::mock::{add_token, deploy_mock_token_contract};
use test_harness::interface::mock::{get_sub_id, mint_tokens};
use test_harness::interface::scripts::transaction_inputs_outputs;
use test_harness::interface::AddLiquidityScriptConfigurables;
use test_harness::interface::{AddLiquidityScript, MockToken, PoolMetadata};
use test_harness::math::{initial_liquidity, proportional_value};
use test_harness::paths::ADD_LIQUIDITY_SCRIPT_BINARY_PATH;
use test_harness::setup::common::{deploy_amm, setup_wallet_and_provider};
use test_harness::types::PoolId;
use test_harness::utils::common::order_sub_ids;

const MINIMUM_LIQUIDITY: u64 = 1000;

pub fn preview_add_liquidity(pool_metadata: PoolMetadata, asset_0_in: u64, asset_1_in: u64) -> u64 {
    let total_liquidity = pool_metadata.liquidity.amount;
    let added_liquidity: u64 = if total_liquidity == 0 {
        let init_liquidity = initial_liquidity(asset_0_in, asset_1_in);

        init_liquidity - MINIMUM_LIQUIDITY
    } else {
        min(
            proportional_value(asset_0_in, total_liquidity, pool_metadata.reserve_0),
            proportional_value(asset_1_in, total_liquidity, pool_metadata.reserve_1),
        )
    };
    added_liquidity
}

pub async fn setup() -> (
    AddLiquidityScript<WalletUnlocked>,
    MiraAMMContract,
    MockToken<WalletUnlocked>,
    Provider,
    PoolId,
    WalletUnlocked,
    TransactionParameters,
    u32,
) {
    let (wallet, _asset_ids, provider) =
        setup_wallet_and_provider(&WalletAssetConfiguration::default()).await;
    let amm = deploy_amm(&wallet).await;
    let (token_contract_id, token_contract) = deploy_mock_token_contract(&wallet).await;

    let token_a_id = add_token(&token_contract, "TOKEN_A".to_string(), "TKA".to_string(), 9)
        .await
        .value;
    let token_b_id = add_token(&token_contract, "TOKEN_B".to_string(), "TKB".to_string(), 9)
        .await
        .value;

    let token_a_sub_id = get_sub_id(&token_contract, token_a_id).await.value.unwrap();
    let token_b_sub_id = get_sub_id(&token_contract, token_b_id).await.value.unwrap();

    mint_tokens(&token_contract, token_a_id, 1_000_000).await;
    mint_tokens(&token_contract, token_b_id, 1_000_000).await;

    let (token_a_sub_id, token_b_sub_id) =
        order_sub_ids((token_a_id, token_b_id), (token_a_sub_id, token_b_sub_id));

    let pool_id = create_pool(
        &amm.instance,
        &token_contract,
        token_contract_id,
        token_a_sub_id,
        token_contract_id,
        token_b_sub_id,
        false,
    )
    .await
    .value;

    let transaction_parameters = transaction_inputs_outputs(
        &wallet,
        &provider,
        &[(*token_a_id).into(), (*token_b_id).into()],
        Some(&vec![1_000_000, 1_000_000]),
    )
    .await;

    let deadline = provider.latest_block_height().await.unwrap() + 10;

    let configurables = AddLiquidityScriptConfigurables::default()
        .with_AMM_CONTRACT_ID(ContractId::from_str(&amm.id.to_string()).unwrap())
        .unwrap();
    let script_instance = AddLiquidityScript::new(wallet.clone(), ADD_LIQUIDITY_SCRIPT_BINARY_PATH)
        .with_configurables(configurables);

    (
        script_instance,
        amm,
        token_contract,
        provider,
        pool_id,
        wallet,
        transaction_parameters,
        deadline,
    )
}
