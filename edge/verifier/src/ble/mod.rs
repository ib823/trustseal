//! BLE module for credential presentation via GATT.
//!
//! Implements the VaultPass BLE protocol with custom UUIDs.

mod advertiser;
mod gatt;
mod protocol;

pub use advertiser::BleAdvertiser;
pub use gatt::GattServer;
pub use protocol::PresentationResponse;
