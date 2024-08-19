pub mod common {
    use fuels::{
        accounts::{wallet::WalletUnlocked, ViewOnlyAccount},
        types::{AssetId, Bits256, Bytes32, ContractId},
    };
    use sha2::{Digest, Sha256};

    use crate::data_structures::WalletBalances;

    pub const MINIMUM_LIQUIDITY: u64 = 1000;

    pub fn to_9_decimal(num: u64) -> u64 {
        num * 1_000_000_000
    }

    pub async fn wallet_balances_for_pool_asset(
        wallet: &WalletUnlocked,
        asset_pair: &(AssetId, AssetId),
        pool_asset_sub_id: &AssetId,
    ) -> WalletBalances {
        let asset_a = wallet.get_asset_balance(&asset_pair.0).await.unwrap();
        let asset_b = wallet.get_asset_balance(&asset_pair.1).await.unwrap();
        let liquidity_pool_asset = wallet.get_asset_balance(&pool_asset_sub_id).await.unwrap();
        WalletBalances {
            asset_a,
            asset_b,
            liquidity_pool_asset,
        }
    }

    pub fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
        let mut hasher = Sha256::new();
        hasher.update(*contract);
        hasher.update(*sub_id);
        AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
    }

    pub fn get_share_sub_id(asset_pair: &(AssetId, AssetId)) -> Bytes32 {
        let mut hasher = Sha256::new();
        hasher.update(*asset_pair.0);
        hasher.update(*asset_pair.1);
        Bytes32::from(<[u8; 32]>::from(hasher.finalize()))
    }

    pub fn order_token_ids(pair: (AssetId, AssetId)) -> (AssetId, AssetId) {
        if pair.0 < pair.1 {
            (pair.0, pair.1)
        } else {
            (pair.1, pair.0)
        }
    }

    pub fn order_sub_ids(
        asset_ids: (AssetId, AssetId),
        sub_ids: (Bits256, Bits256),
    ) -> (Bits256, Bits256) {
        if asset_ids.0 < asset_ids.1 {
            (sub_ids.0, sub_ids.1)
        } else {
            (sub_ids.1, sub_ids.0)
        }
    }
}

pub mod mock {
    use fuels::{accounts::wallet::WalletUnlocked, types::ContractId};

    use crate::interface::mock;

    pub async fn deploy_2_mock_tokens(
        wallet: &WalletUnlocked,
        name_0: String,
        symbol_0: String,
        decimals_0: u8,
        name_1: String,
        symbol_1: String,
        decimals_1: u8,
    ) -> ContractId {
        let (token_contract_id, token_contract) = mock::deploy_mock_token_contract(wallet).await;

        mock::add_token(&token_contract, name_0, symbol_0, decimals_0)
            .await
            .tx_id
            .unwrap();
        mock::add_token(&token_contract, name_1, symbol_1, decimals_1)
            .await
            .tx_id
            .unwrap();

        token_contract_id
    }
}
