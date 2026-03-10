//! NFC module for credential presentation via PC/SC.
//!
//! Supports ISO 14443 Type A/B cards and NDEF External Type records.

mod apdu;
mod reader;

pub use reader::{NfcEvent, NfcReader};
