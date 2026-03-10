//! Status list parsing for revocation checking.
//!
//! Implements W3C Bitstring Status List v1.0 parsing on the client side.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use flate2::read::GzDecoder;
use std::io::Read;

/// Check if a credential is revoked by checking the status list.
///
/// # Arguments
/// * `encoded_list` - The gzip + base64 encoded bitstring from the status list credential
/// * `index` - The status list index for the credential
///
/// # Returns
/// `true` if the credential is revoked (bit is set), `false` otherwise.
pub fn check_revocation_status(encoded_list: &str, index: usize) -> Result<bool, String> {
    // Decode base64
    let compressed = BASE64
        .decode(encoded_list)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    // Decompress gzip
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut bitstring = Vec::new();
    decoder
        .read_to_end(&mut bitstring)
        .map_err(|e| format!("Gzip decompress error: {}", e))?;

    // Check bit at index (MSB-first per W3C spec)
    let byte_index = index / 8;
    let bit_index = 7 - (index % 8); // MSB-first

    if byte_index >= bitstring.len() {
        return Err(format!(
            "Index {} out of range (list size: {} bits)",
            index,
            bitstring.len() * 8
        ));
    }

    let is_revoked = (bitstring[byte_index] >> bit_index) & 1 == 1;
    Ok(is_revoked)
}

/// Parse a status list credential to extract the encoded list.
///
/// # Arguments
/// * `credential_json` - The JSON-encoded status list credential
///
/// # Returns
/// The encoded list string from the credential subject.
pub fn extract_encoded_list(credential_json: &str) -> Result<String, String> {
    // Simple JSON parsing - in production would use serde_json
    // For now, look for "encodedList": "..."
    let marker = "\"encodedList\":";
    let start = credential_json
        .find(marker)
        .ok_or("encodedList field not found")?;

    let rest = &credential_json[start + marker.len()..];
    let quote_start = rest.find('"').ok_or("Invalid JSON format")?;
    let rest = &rest[quote_start + 1..];
    let quote_end = rest.find('"').ok_or("Invalid JSON format")?;

    Ok(rest[..quote_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn create_test_status_list(size: usize, revoked_indices: &[usize]) -> String {
        let mut bitstring = vec![0u8; size.div_ceil(8)];

        for &index in revoked_indices {
            let byte_index = index / 8;
            let bit_index = 7 - (index % 8);
            if byte_index < bitstring.len() {
                bitstring[byte_index] |= 1 << bit_index;
            }
        }

        // Gzip compress
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bitstring).unwrap();
        let compressed = encoder.finish().unwrap();

        // Base64 encode
        BASE64.encode(&compressed)
    }

    #[test]
    fn test_check_not_revoked() {
        let encoded = create_test_status_list(1000, &[5, 10, 100]);
        assert!(!check_revocation_status(&encoded, 0).unwrap());
        assert!(!check_revocation_status(&encoded, 50).unwrap());
    }

    #[test]
    fn test_check_revoked() {
        let encoded = create_test_status_list(1000, &[5, 10, 100]);
        assert!(check_revocation_status(&encoded, 5).unwrap());
        assert!(check_revocation_status(&encoded, 10).unwrap());
        assert!(check_revocation_status(&encoded, 100).unwrap());
    }

    #[test]
    fn test_index_out_of_range() {
        let encoded = create_test_status_list(100, &[]);
        let result = check_revocation_status(&encoded, 200);
        assert!(result.is_err());
    }
}
