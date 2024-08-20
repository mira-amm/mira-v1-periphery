use crate::utils::setup;
use fuels::accounts::ViewOnlyAccount;
use fuels::prelude::{Address, AssetId};
use test_harness::interface::amm::pool_metadata;
use test_harness::interface::scripts::get_transaction_inputs_outputs;

#[tokio::test]
pub async fn adds_liquidity_with_equal_deposit_amounts() {
    let (script_instance, amm, pool_id, wallet, deadline) = setup().await;

    let amount_0_desired = 10000;
    let amount_1_desired = 10000;
    let expected_liquidity = 9000; // 10000 - 1000(minimal liquidity)

    let (inputs, outputs) = get_transaction_inputs_outputs(
        &wallet,
        &vec![(pool_id.0, amount_0_desired), (pool_id.1, amount_1_desired)],
        &vec![
            (AssetId::default(), Address::zeroed()),
            (AssetId::default(), Address::zeroed()),
        ],
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
        .with_inputs(inputs)
        .with_outputs(outputs)
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(added_liquidity.amount, expected_liquidity);
}

#[tokio::test]
async fn adds_liquidity_to_make_a_more_valuable() {
    let (script_instance, amm, pool_id, wallet, deadline) = setup().await;

    let amount_0_desired = 40000;
    let amount_1_desired = 10000;
    let expected_liquidity = 19000; // 20000 - 1000(minimal liquidity)

    let (inputs, outputs) = get_transaction_inputs_outputs(
        &wallet,
        &vec![(pool_id.0, amount_0_desired), (pool_id.1, amount_1_desired)],
        &vec![
            (AssetId::default(), Address::zeroed()),
            (AssetId::default(), Address::zeroed()),
        ],
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
        .with_inputs(inputs)
        .with_outputs(outputs)
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(added_liquidity.amount, expected_liquidity);
}

#[tokio::test]
async fn adds_liquidity_to_make_b_more_valuable() {
    let (script_instance, amm, pool_id, wallet, deadline) = setup().await;

    let amount_0_desired = 10000;
    let amount_1_desired = 40000;
    let expected_liquidity = 19000; // 20000 - 1000(minimal liquidity)

    let (inputs, outputs) = get_transaction_inputs_outputs(
        &wallet,
        &vec![(pool_id.0, amount_0_desired), (pool_id.1, amount_1_desired)],
        &vec![
            (AssetId::default(), Address::zeroed()),
            (AssetId::default(), Address::zeroed()),
        ],
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
        .with_inputs(inputs)
        .with_outputs(outputs)
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(added_liquidity.amount, expected_liquidity);
}

#[tokio::test]
async fn adds_further_liquidity_without_extra_deposit_when_a_is_more_valuable() {
    let (script_instance, amm, pool_id, wallet, deadline) = setup().await;
    let amount_0_desired = 10000;
    let amount_1_desired = 10000;

    let (inputs, outputs) = get_transaction_inputs_outputs(
        &wallet,
        &vec![(pool_id.0, amount_0_desired), (pool_id.1, amount_1_desired)],
        &vec![
            (AssetId::default(), Address::zeroed()),
            (AssetId::default(), Address::zeroed()),
        ],
    )
    .await;
    // adds initial liquidity
    script_instance
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
        .with_inputs(inputs)
        .with_outputs(outputs)
        .call()
        .await
        .unwrap();

    let amount_0_desired = 10000;
    let amount_1_desired = 40000;

    println!("{:?}", wallet.get_balances().await.unwrap());

    let (inputs, outputs) = get_transaction_inputs_outputs(
        &wallet,
        &vec![(pool_id.0, amount_0_desired), (pool_id.1, amount_1_desired)],
        &vec![(AssetId::default(), Address::zeroed())],
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
        .with_inputs(inputs)
        .with_outputs(outputs)
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(added_liquidity.amount, 10000);
    let pool_meta = pool_metadata(&amm.instance, pool_id).await.value.unwrap();
    assert_eq!(pool_meta.reserve_0, 20000);
    assert_eq!(pool_meta.reserve_1, 20000);
}
