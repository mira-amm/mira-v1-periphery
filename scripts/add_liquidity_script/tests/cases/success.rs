use test_harness::interface::{
    amm::pool_metadata, mock::mint_tokens, scripts::transaction_inputs_outputs,
};

use crate::utils::{preview_add_liquidity, setup};

#[tokio::test]
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

    let amount_0_desired = 10000;
    let amount_1_desired = 10000;
    let expected_liquidity = 9000;

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

    assert_eq!(added_liquidity.amount, expected_liquidity);
}

#[tokio::test]
async fn adds_liquidity_to_make_a_more_valuable() {
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
    let pool_metadata = pool_metadata(&amm.instance, pool_id).await.value.unwrap();

    let amount_0_desired = 40000;
    let amount_1_desired = 10000;

    let expected_liquidity =
        preview_add_liquidity(pool_metadata, amount_0_desired, amount_1_desired);

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

    assert_eq!(added_liquidity.amount, expected_liquidity);
}

#[tokio::test]
async fn adds_liquidity_to_make_b_more_valuable() {
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

    let amount_0_desired = 10000;
    let amount_1_desired = 40000;

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

    assert_eq!(added_liquidity.amount, 19000); // 20000 - 1000(minimal liquidity)
}

#[tokio::test]
async fn adds_further_liquidity_without_extra_deposit_when_a_is_more_valuable() {
    let (
        script_instance,
        amm,
        token_contract,
        provider,
        pool_id,
        wallet,
        transaction_parameters,
        deadline,
    ) = setup().await;

    // adds initial liquidity
    script_instance
        .main(
            pool_id,
            10_000,
            10_000,
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
        .unwrap();

    let amount_0_desired = 10000;
    let amount_1_desired = 40000;

    mint_tokens(&token_contract, pool_id.0, amount_0_desired).await;
    mint_tokens(&token_contract, pool_id.1, amount_1_desired).await;

    let transaction_parameters = transaction_inputs_outputs(
        &wallet,
        &provider,
        &[(*pool_id.0).into(), (*pool_id.1).into()],
        Some(&vec![amount_0_desired, amount_1_desired]),
    )
    .await;

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

    assert_eq!(added_liquidity.amount, 10000);
}

#[tokio::test]
async fn adds_further_liquidity_without_extra_deposit_when_b_is_more_valuable() {
    let (
        script_instance,
        amm,
        token_contract,
        provider,
        pool_id,
        wallet,
        transaction_parameters,
        deadline,
    ) = setup().await;

    // adds initial liquidity
    script_instance
        .main(
            pool_id,
            10_000,
            10_000,
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
        .unwrap();

    let amount_0_desired = 10000;
    let amount_1_desired = 40000;

    mint_tokens(&token_contract, pool_id.0, amount_0_desired).await;
    mint_tokens(&token_contract, pool_id.1, amount_1_desired).await;

    let transaction_parameters = transaction_inputs_outputs(
        &wallet,
        &provider,
        &[(*pool_id.0).into(), (*pool_id.1).into()],
        Some(&vec![amount_0_desired, amount_1_desired]),
    )
    .await;

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

    assert_eq!(added_liquidity.amount, 10000);
}
