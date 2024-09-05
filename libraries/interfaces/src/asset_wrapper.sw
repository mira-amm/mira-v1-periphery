library;

use std::string::String;

abi AssetWrapper {
    #[storage(read, write)]
    fn transfer_ownership(new_owner: Identity);

    #[storage(read, write)]
    fn init_wrapped_asset(underlying_asset: AssetId, name: String, symbol: String, decimals: u8) -> AssetId;

    #[payable]
    #[storage(read, write)]
    fn wrap() -> AssetId;

    #[payable]
    #[storage(read, write)]
    fn unwrap(underlying_asset: AssetId);
}
