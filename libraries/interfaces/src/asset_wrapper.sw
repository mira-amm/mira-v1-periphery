library;

use std::string::String;

abi AssetWrapper {
    #[storage(read, write)]
    fn transfer_ownership(new_owner: Identity);

    #[storage(read, write)]
    fn update_asset_config(asset: AssetId, name: String, symbol: String, decimals: u8) -> AssetId;

    #[payable]
    #[storage(read, write)]
    fn wrap() -> AssetId;

    #[payable]
    #[storage(read, write)]
    fn unwrap() -> AssetId;

    #[storage(read)]
    fn wrapped_asset_for(underlying_asset: AssetId) -> Option<(b256, AssetId)>;

    #[storage(read)]
    fn underlying_asset_for(wrapped_asset: AssetId) -> Option<AssetId>;
}
