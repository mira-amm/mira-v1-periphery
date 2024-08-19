use fuels::{
    accounts::provider::Provider,
    accounts::provider::ResourceFilter,
    prelude::{Bech32Address, WalletUnlocked},
    types::{input::Input, output::Output, AssetId},
};

pub async fn get_transaction_inputs_outputs(
    wallet: &WalletUnlocked,
    assets: &Vec<AssetId>,
    amounts: &Vec<u64>,
) -> (Vec<Input>, Vec<Output>) {
    let provider = wallet.provider().unwrap();
    let mut input_coins: Vec<Input> = vec![]; // capacity depends on wallet resources
    let mut output_variables: Vec<Output> = Vec::with_capacity(assets.len());

    for (asset_index, asset) in assets.iter().enumerate() {
        let asset_coins = transaction_input_coin(
            provider,
            wallet.address(),
            *asset,
            *amounts.get(asset_index).unwrap(),
        )
        .await;

        if asset_coins.len() == 0 {
            output_variables.push(Output::Variable {
                asset_id: *asset,
                amount: 0,
                to: wallet.address().into(),
            });
        } else {
            output_variables.push(Output::Change {
                asset_id: *asset,
                amount: 0,
                to: wallet.address().into(),
            });
        }

        input_coins.extend(asset_coins);
    }

    (input_coins, output_variables)
}

async fn transaction_input_coin(
    provider: &Provider,
    from: &Bech32Address,
    asset_id: AssetId,
    amount: u64,
) -> Vec<Input> {
    let coins = &provider
        .get_spendable_resources(ResourceFilter {
            from: from.clone(),
            asset_id: Some(asset_id),
            amount,
            excluded_utxos: vec![],
            excluded_message_nonces: vec![],
        })
        .await
        .unwrap();

    let input_coins: Vec<Input> = coins
        .iter()
        .map(|coin| Input::ResourceSigned {
            resource: coin.clone(),
        })
        .collect();

    input_coins
}
