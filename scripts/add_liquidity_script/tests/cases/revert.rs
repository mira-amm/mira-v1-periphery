use crate::utils::setup;

// TODO fix
#[tokio::test]
#[ignore]
// #[should_panic(expected = "CannotAddLessThanMinimumLiquidity")]
pub async fn adds_liquidity_with_equal_deposit_amounts() {
    let (
        script_instance,
        amm,
        _token_contract,
        _provider,
        pool_id,
        wallet,
        transaction_parameters,
        deadline,
    ) = setup().await;

    let amount_0_desired = 1000;
    let amount_1_desired = 1000;

    let added_liquidity = script_instance
        .main(
            pool_id,
            amount_0_desired,
            amount_1_desired,
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

    println!("{}", added_liquidity.amount);
}
