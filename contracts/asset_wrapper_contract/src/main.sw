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
use interfaces::asset_wrapper::AssetWrapper;

storage {
    total_assets: u64 = 0,
    asset_total_supply: StorageMap<AssetId, u64> = StorageMap {},
    asset_decimals: StorageMap<AssetId, u8> = StorageMap {},
    asset_name: StorageMap<AssetId, StorageString> = StorageMap {},
    asset_symbol: StorageMap<AssetId, StorageString> = StorageMap {},
}

fn get_wrapped_asset(underlying_asset: AssetId) -> (b256, AssetId) {
    let sub_id: b256 = underlying_asset.into();
    (sub_id, AssetId::new(ContractId::this(), sub_id))
}

#[storage(read)]
fn asset_exists(asset: AssetId) -> bool {
    get_total_supply(asset).is_some()
}

#[storage(read)]
fn wrapped_asset_exists(underlying_asset: AssetId) -> bool {
    let (_, wrapped_asset) = get_wrapped_asset(underlying_asset);
    asset_exists(wrapped_asset)
}

#[storage(read)]
fn get_total_supply(asset_id: AssetId) -> Option<u64> {
    storage.asset_total_supply.get(asset_id).try_read()
}

#[storage(read, write)]
fn store_asset_meta(
    underlying_asset: AssetId,
    name: String,
    symbol: String,
    decimals: u8,
) -> AssetId {
    let (_, wrapped_asset) = get_wrapped_asset(underlying_asset);
    _set_name(storage.asset_name, wrapped_asset, name);
    _set_symbol(storage.asset_symbol, wrapped_asset, symbol);
    storage.asset_decimals.insert(wrapped_asset, decimals);
    wrapped_asset
}

#[storage(read, write)]
fn init_wrapped_asset(wrapped_asset: AssetId) {
    storage.asset_total_supply.insert(wrapped_asset, 0);
    storage.total_assets.write(storage.total_assets.read() + 1);
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
        storage.asset_decimals.get(asset).try_read()
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
    fn update_wrapped_asset_config(underlying_asset: AssetId, name: String, symbol: String, decimals: u8) -> AssetId {
        only_owner();
        let wrapped_asset = store_asset_meta(underlying_asset, name, symbol, decimals);
        if !asset_exists(wrapped_asset) {
            init_wrapped_asset(wrapped_asset);
        }
        wrapped_asset
    }

    #[payable]
    #[storage(read, write)]
    fn wrap() -> AssetId {
        let sender = msg_sender().unwrap();
        let underlying_asset = msg_asset_id();
        let amount = msg_amount();

        require(wrapped_asset_exists(underlying_asset), "Wrapped asset doesn't exist");
        let (wrapped_sub_id, wrapped_asset) = get_wrapped_asset(underlying_asset);

        mint_to(sender, wrapped_sub_id, amount);

        let old_total_supply = get_total_supply(wrapped_asset).unwrap();
        storage
            .asset_total_supply
            .insert(wrapped_asset, old_total_supply + amount);
        wrapped_asset
    }

    #[payable]
    #[storage(read, write)]
    fn unwrap(underlying_asset: AssetId) {
        let sender = msg_sender().unwrap();
        let deposited_asset = msg_asset_id();
        let amount = msg_amount();

        let (wrapped_sub_id, wrapped_asset) = get_wrapped_asset(underlying_asset);
        require(deposited_asset == wrapped_asset, "Deposited asset doesn't match the wrapped one");
        require(wrapped_asset_exists(underlying_asset), "Wrapped asset doesn't exist");

        burn(wrapped_sub_id, amount);
        transfer(sender, underlying_asset, amount);

        let old_total_supply = get_total_supply(wrapped_asset).unwrap();
        storage
            .asset_total_supply
            .insert(wrapped_asset, old_total_supply - amount);
    }
}
