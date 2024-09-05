contract;

use sway_libs::{ownership::{_owner, initialize_ownership, only_owner, transfer_ownership,},};
use standards::{src20::SRC20, src5::{SRC5, State}};
use sway_libs::asset::base::{_set_name, _set_symbol, _total_assets};
use std::{
    asset::{
        burn,
        mint_to,
        transfer,
    },
    call_frames::msg_asset_id,
    context::msg_amount,
    hash::Hash,
    primitive_conversions::{
        b256::*,
        u256::*,
    },
    storage::storage_string::*,
    string::String,
};
use interfaces::{asset_wrapper::AssetWrapper, data_structures::WrappedAssetMetadata};

storage {
    total_assets: u64 = 0,
    asset_total_supply: StorageMap<AssetId, u64> = StorageMap {},
    asset_meta: StorageMap<AssetId, WrappedAssetMetadata> = StorageMap {},
    asset_to_wrapped_sub_id: StorageMap<AssetId, b256> = StorageMap {},
    // No way to add those to WrappedAssetMetadata
    asset_name: StorageMap<AssetId, StorageString> = StorageMap {},
    asset_symbol: StorageMap<AssetId, StorageString> = StorageMap {},
}

#[storage(read)]
fn get_asset_meta(asset_id: AssetId) -> Option<WrappedAssetMetadata> {
    storage.asset_meta.get(asset_id).try_read()
}

#[storage(read)]
fn get_total_supply(asset_id: AssetId) -> Option<u64> {
    storage.asset_total_supply.get(asset_id).try_read()
}

#[storage(read, write)]
fn store_asset_meta(
    underlying_asset: AssetId,
    sub_id: b256,
    name: String,
    symbol: String,
    decimals: u8,
) -> AssetId {
    let wrapped_asset = AssetId::new(ContractId::this(), sub_id);
    _set_name(storage.asset_name, wrapped_asset, name);
    _set_symbol(storage.asset_symbol, wrapped_asset, symbol);

    let asset_meta = WrappedAssetMetadata {
        underlying_asset: underlying_asset,
        sub_id: sub_id,
        decimals: decimals,
    };

    storage.asset_meta.insert(wrapped_asset, asset_meta);
    storage
        .asset_to_wrapped_sub_id
        .insert(underlying_asset, sub_id);
    wrapped_asset
}

#[storage(read, write)]
fn create_wrapped_asset(
    underlying_asset: AssetId,
    name: String,
    symbol: String,
    decimals: u8,
) -> AssetId {
    let total_assets = storage.total_assets.read();
    let total_assets_u256: u256 = total_assets.into();
    let sub_id: b256 = total_assets_u256.into();

    let wrapped_asset = store_asset_meta(underlying_asset, sub_id, name, symbol, decimals);
    storage.asset_total_supply.insert(wrapped_asset, 0);
    storage.total_assets.write(total_assets + 1);

    wrapped_asset
}

#[storage(read, write)]
fn update_wrapped_asset(
    underlying_asset: AssetId,
    name: String,
    symbol: String,
    decimals: u8,
) -> Option<AssetId> {
    match storage.asset_to_wrapped_sub_id.get(underlying_asset).try_read() {
        None => None,
        Some(sub_id) => {
            let wrapped_asset = store_asset_meta(underlying_asset, sub_id, name, symbol, decimals);
            Some(wrapped_asset)
        }
    }
}

impl SRC5 for Contract {
    #[storage(read)]
    fn owner() -> State {
        _owner()
    }
}

impl SRC20 for Contract {
    #[storage(read)]
    fn total_assets() -> u64 {
        _total_assets(storage.total_assets)
    }

    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        get_total_supply(asset)
    }

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        storage.asset_name.get(asset).read_slice()
    }

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        storage.asset_symbol.get(asset).read_slice()
    }

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        match get_asset_meta(asset) {
            Some(meta) => Some(meta.decimals),
            None => None,
        }
    }
}

impl AssetWrapper for Contract {
    #[storage(read, write)]
    fn transfer_ownership(new_owner: Identity) {
        if _owner() == State::Uninitialized {
            initialize_ownership(new_owner);
        } else {
            transfer_ownership(new_owner);
        }
    }

    #[storage(read, write)]
    fn update_asset_config(asset: AssetId, name: String, symbol: String, decimals: u8) -> AssetId {
        only_owner();
        match update_wrapped_asset(asset, name, symbol, decimals) {
            Some(wrapped) => {
                wrapped
            },
            None => {
                create_wrapped_asset(asset, name, symbol, decimals)
            }
        }
    }

    #[payable]
    #[storage(read, write)]
    fn wrap() -> AssetId {
        let sender = msg_sender().unwrap();
        let underlying_asset = msg_asset_id();
        let amount = msg_amount();

        let wrapped_sub_id = storage.asset_to_wrapped_sub_id.get(underlying_asset).read();
        let wrapped_asset = AssetId::new(ContractId::this(), wrapped_sub_id);
        mint_to(sender, wrapped_sub_id, amount);

        let old_total_supply = get_total_supply(wrapped_asset).unwrap();
        storage
            .asset_total_supply
            .insert(wrapped_asset, old_total_supply + amount);
        wrapped_asset
    }

    #[payable]
    #[storage(read, write)]
    fn unwrap() -> AssetId {
        let sender = msg_sender().unwrap();
        let wrapped_asset = msg_asset_id();
        let amount = msg_amount();

        let wrapped_asset_meta = get_asset_meta(wrapped_asset).unwrap();
        transfer(sender, wrapped_asset_meta.underlying_asset, amount);
        burn(wrapped_asset_meta.sub_id, amount);

        let old_total_supply = get_total_supply(wrapped_asset).unwrap();
        storage
            .asset_total_supply
            .insert(wrapped_asset, old_total_supply - amount);
        wrapped_asset_meta.underlying_asset
    }

    #[storage(read)]
    fn wrapped_asset_for(underlying_asset: AssetId) -> Option<(b256, AssetId)> {
        match storage.asset_to_wrapped_sub_id.get(underlying_asset).try_read() {
            Some(wrapped_sub_id) => {
                Some((wrapped_sub_id, AssetId::new(ContractId::this(), wrapped_sub_id)))
            },
            None => {
                None
            },
        }
    }

    #[storage(read)]
    fn underlying_asset_for(wrapped_asset: AssetId) -> Option<AssetId> {
        match get_asset_meta(wrapped_asset) {
            Some(wrapped_asset_meta) => {
                Some(wrapped_asset_meta.underlying_asset)
            },
            None => None,
        }
    }
}
