use crate::utils::setup;
use fuels::programs::call_utils::TxDependencyExtension;
use test_harness::interface::amm::pool_metadata;
use test_harness::interface::BurnEvent;
use test_harness::utils::common::{
    to_9_decimal, wallet_balances_for_pool_asset, MINIMUM_LIQUIDITY,
};
use test_harness::wallet::get_transaction_inputs_outputs;

#[tokio::test]
#[ignore]
async fn removes_all_liquidity_passing_exact_a_and_b_values() {
    let (
        add_liquidity_script_instance,
        remove_liquidity_script_instance,
        amm,
        _token_contract,
        _provider,
        pool_id,
        wallet,
        transaction_parameters,
        deadline,
    ) = setup().await;

    let token_0_amount: u64 = to_9_decimal(1);
    let token_1_amount: u64 = to_9_decimal(1);
    let expected_liquidity: u64 = to_9_decimal(1) - MINIMUM_LIQUIDITY;

    // adds initial liquidity
    let added_liquidity = add_liquidity_script_instance
        .main(
            pool_id,
            token_0_amount,
            token_1_amount,
            0,
            0,
            wallet.address().into(),
            deadline,
        )
        .with_contracts(&[&amm.instance])
        .with_inputs(transaction_parameters.inputs)
        .with_outputs(transaction_parameters.outputs)
        .call()
        .await
        .unwrap()
        .value;

    let initial_pool_metadata = pool_metadata(&amm.instance, pool_id).await.value.unwrap();
    let initial_wallet_balances =
        wallet_balances_for_pool_asset(&wallet, &(pool_id.0, pool_id.1), &added_liquidity.id).await;

    let transaction_parameters = get_transaction_inputs_outputs(
        &wallet,
        &vec![added_liquidity.id],
        &vec![added_liquidity.amount],
    )
    .await;

    let removed_liquidity = remove_liquidity_script_instance
        .main(
            pool_id,
            added_liquidity.amount,
            0,
            0,
            wallet.address().into(),
            deadline,
        )
        .with_contracts(&[&amm.instance])
        .with_inputs(transaction_parameters.0)
        .with_outputs(transaction_parameters.1)
        .append_variable_outputs(2)
        .call()
        .await
        .unwrap();

    let log = removed_liquidity
        .decode_logs_with_type::<BurnEvent>()
        .unwrap();
    let event = log.first().unwrap();

    let final_pool_metadata = pool_metadata(&amm.instance, pool_id).await.value.unwrap();
    let final_wallet_balances =
        wallet_balances_for_pool_asset(&wallet, &(pool_id.0, pool_id.1), &added_liquidity.id).await;

    assert_eq!(
        *event,
        BurnEvent {
            pool_id,
            recipient: wallet.address().into(),
            liquidity: added_liquidity,
            asset_0_out: removed_liquidity.value.0,
            asset_1_out: removed_liquidity.value.1,
        }
    );
    assert_eq!(
        final_wallet_balances.liquidity_pool_asset,
        initial_wallet_balances.liquidity_pool_asset - expected_liquidity
    );
    assert_eq!(
        final_pool_metadata.reserve_0,
        initial_pool_metadata.reserve_0 - token_0_amount
    );
    assert_eq!(
        final_pool_metadata.reserve_1,
        initial_pool_metadata.reserve_1 - token_1_amount
    );
    assert_eq!(
        final_pool_metadata.liquidity.amount,
        initial_pool_metadata.liquidity.amount - expected_liquidity
    );
}

#[tokio::test]
#[ignore]
async fn removes_all_liquidity_passing_exact_a_but_not_exact_b_values() {
    let (
        add_liquidity_script_instance,
        remove_liquidity_script_instance,
        amm,
        _token_contract,
        _provider,
        pool_id,
        wallet,
        transaction_parameters,
        deadline,
    ) = setup().await;

    let token_0_amount: u64 = to_9_decimal(1);
    let token_1_amount: u64 = to_9_decimal(4);
    let expected_liquidity: u64 = to_9_decimal(2) - MINIMUM_LIQUIDITY;

    // adds initial liquidity
    let added_liquidity = add_liquidity_script_instance
        .main(
            pool_id,
            token_0_amount,
            token_1_amount,
            1,
            1,
            wallet.address().into(),
            deadline,
        )
        .with_contracts(&[&amm.instance])
        .with_inputs(transaction_parameters.inputs)
        .with_outputs(transaction_parameters.outputs)
        .call()
        .await
        .unwrap()
        .value;

    let lp_token_amount = added_liquidity.amount;
    assert_eq!(lp_token_amount, expected_liquidity);

    let initial_wallet_balances =
        wallet_balances_for_pool_asset(&wallet, &(pool_id.0, pool_id.1), &added_liquidity.id).await;

    let transaction_parameters = get_transaction_inputs_outputs(
        &wallet,
        &vec![added_liquidity.id],
        &vec![expected_liquidity],
    )
    .await;

    let removed_liquidity = remove_liquidity_script_instance
        .main(
            pool_id,
            added_liquidity.amount,
            0,
            0,
            wallet.address().into(),
            deadline,
        )
        .with_contracts(&[&amm.instance])
        .with_inputs(transaction_parameters.0)
        .with_outputs(transaction_parameters.1)
        .append_variable_outputs(2)
        .call()
        .await
        .unwrap();

    let log = removed_liquidity
        .decode_logs_with_type::<BurnEvent>()
        .unwrap();
    let event = log.first().unwrap();

    let final_wallet_balances =
        wallet_balances_for_pool_asset(&wallet, &(pool_id.0, pool_id.1), &added_liquidity.id).await;

    assert_eq!(
        *event,
        BurnEvent {
            pool_id,
            recipient: wallet.address().into(),
            liquidity: added_liquidity,
            asset_0_out: removed_liquidity.value.0,
            asset_1_out: removed_liquidity.value.1,
        }
    );
    assert_eq!(
        final_wallet_balances.asset_a,
        initial_wallet_balances.asset_a + token_0_amount - 500
    );
    assert_eq!(
        final_wallet_balances.asset_b,
        initial_wallet_balances.asset_b + token_1_amount - 2000
    );
}
