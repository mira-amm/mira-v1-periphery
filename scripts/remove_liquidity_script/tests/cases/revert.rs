use crate::utils::setup;
use fuels::programs::call_utils::TxDependencyExtension;
use test_harness::utils::common::to_9_decimal;
use test_harness::wallet::get_transaction_inputs_outputs;

#[tokio::test]
#[ignore]
#[should_panic(expected = "ZeroInputAmount")]
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

    let transaction_parameters = get_transaction_inputs_outputs(
        &wallet,
        &vec![added_liquidity.id],
        &vec![added_liquidity.amount],
    )
    .await;

    remove_liquidity_script_instance
        .main(pool_id, 0, 0, 0, wallet.address().into(), deadline)
        .with_contracts(&[&amm.instance])
        .with_inputs(transaction_parameters.0)
        .with_outputs(transaction_parameters.1)
        .append_variable_outputs(2)
        .call()
        .await
        .unwrap();
}
