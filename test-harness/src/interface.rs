use fuels::{prelude::abigen, programs::call_utils::TxDependencyExtension};

abigen!(
    Script(
        name = "AddLiquidityScript",
        abi = "scripts/add_liquidity_script/out/debug/add_liquidity_script-abi.json"
    ),
    Contract(
        name = "MiraAMM",
        abi = "fixtures/mira-amm/mira_amm_contract-abi.json"
    ),
    Contract(
        name = "MockToken",
        abi = "fixtures/mock-token/mock_token-abi.json"
    )
);

pub mod amm {
    use fuels::{
        accounts::wallet::WalletUnlocked,
        programs::call_response::FuelCallResponse,
        types::{Bits256, ContractId},
    };

    use crate::types::PoolId;

    use super::*;

    pub async fn create_pool(
        contract: &MiraAMM<WalletUnlocked>,
        token_contract: &MockToken<WalletUnlocked>,
        token_0_contract_id: ContractId,
        token_0_sub_id: Bits256,
        token_1_contract_id: ContractId,
        token_1_sub_id: Bits256,
        is_stable: bool,
    ) -> FuelCallResponse<PoolId> {
        contract
            .methods()
            .create_pool(
                token_0_contract_id,
                token_0_sub_id,
                token_1_contract_id,
                token_1_sub_id,
                is_stable,
            )
            .with_contracts(&[token_contract])
            .call()
            .await
            .unwrap()
    }

    pub async fn pool_metadata(
        contract: &MiraAMM<WalletUnlocked>,
        pool_id: PoolId,
    ) -> FuelCallResponse<Option<PoolMetadata>> {
        contract
            .methods()
            .pool_metadata(pool_id)
            .call()
            .await
            .unwrap()
    }
}

pub mod mock {
    use fuels::{
        accounts::wallet::WalletUnlocked,
        programs::{
            call_response::FuelCallResponse,
            contract::{Contract, LoadConfiguration},
        },
        types::{transaction::TxPolicies, AssetId, Bits256, ContractId},
    };

    use crate::paths::MOCK_TOKEN_CONTRACT_BINARY_PATH;

    use super::*;

    pub async fn deploy_mock_token_contract(
        wallet: &WalletUnlocked,
    ) -> (ContractId, MockToken<WalletUnlocked>) {
        let contract_id = Contract::load_from(
            MOCK_TOKEN_CONTRACT_BINARY_PATH,
            LoadConfiguration::default(),
        )
        .unwrap()
        .deploy(wallet, TxPolicies::default())
        .await
        .unwrap();

        let id = ContractId::from(contract_id.clone());
        let instance = MockToken::new(contract_id, wallet.clone());

        (id, instance)
    }

    pub async fn add_token(
        contract: &MockToken<WalletUnlocked>,
        name: String,
        symbol: String,
        decimals: u8,
    ) -> FuelCallResponse<AssetId> {
        contract
            .methods()
            .add_token(name, symbol, decimals)
            .call()
            .await
            .unwrap()
    }

    pub async fn mint_tokens(
        contract: &MockToken<WalletUnlocked>,
        asset_id: AssetId,
        amount: u64,
    ) -> FuelCallResponse<()> {
        contract
            .methods()
            .mint_tokens(asset_id, amount)
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap()
    }

    pub async fn get_sub_id(
        contract: &MockToken<WalletUnlocked>,
        asset_id: AssetId,
    ) -> FuelCallResponse<Option<Bits256>> {
        contract
            .methods()
            .get_sub_id(asset_id)
            .call()
            .await
            .unwrap()
    }
}

pub mod scripts {
    use crate::data_structures::TransactionParameters;

    use fuels::{
        accounts::{provider::Provider, wallet::WalletUnlocked},
        prelude::ResourceFilter,
        types::{
            bech32::Bech32Address, coin_type::CoinType, input::Input, output::Output, Address,
            AssetId,
        },
    };

    pub const MAXIMUM_INPUT_AMOUNT: u64 = 100_000;

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
                ..Default::default()
            })
            .await
            .unwrap();

        let input_coins: Vec<Input> = coins
            .iter()
            .map(|coin| match coin {
                CoinType::Coin(_) => Input::resource_signed(coin.clone()),
                _ => panic!("Coin type does not match"),
            })
            .collect();

        input_coins
    }

    fn transaction_output_variable() -> Output {
        Output::Variable {
            amount: 0,
            to: Address::zeroed(),
            asset_id: AssetId::default(),
        }
    }

    pub async fn transaction_inputs_outputs(
        wallet: &WalletUnlocked,
        provider: &Provider,
        assets: &[AssetId],
        amounts: Option<&Vec<u64>>,
    ) -> TransactionParameters {
        let mut input_coins: Vec<Input> = vec![]; // capacity depends on wallet resources
        let mut output_variables: Vec<Output> = Vec::with_capacity(assets.len());

        for (asset_index, asset) in assets.iter().enumerate() {
            input_coins.extend(
                transaction_input_coin(
                    provider,
                    wallet.address(),
                    *asset,
                    if let Some(amounts_) = amounts {
                        *amounts_.get(asset_index).unwrap()
                    } else {
                        MAXIMUM_INPUT_AMOUNT
                    },
                )
                .await,
            );
            output_variables.push(transaction_output_variable());
        }

        TransactionParameters {
            inputs: input_coins,
            outputs: output_variables,
        }
    }
}
