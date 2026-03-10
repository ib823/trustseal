//! BLE payload encoding/decoding for credential presentation.
//!
//! Handles serialization of SD-JWT VPs for BLE transmission.

/// Encode a presentation payload for BLE transmission.
///
/// Uses a compact binary format to fit within MTU limits.
/// Target: < 512 bytes to fit in a single BLE packet with MTU 512.
pub fn encode_ble_payload(
    sd_jwt_vp: &[u8],
    key_binding_jwt: &[u8],
) -> Result<Vec<u8>, String> {
    // Format:
    // [1 byte: version]
    // [2 bytes: sd_jwt length (big-endian)]
    // [N bytes: sd_jwt]
    // [2 bytes: kb_jwt length (big-endian)]
    // [M bytes: kb_jwt]

    let version: u8 = 1;
    let sd_jwt_len = sd_jwt_vp.len();
    let kb_jwt_len = key_binding_jwt.len();

    if sd_jwt_len > 65535 || kb_jwt_len > 65535 {
        return Err("Payload too large for BLE".to_string());
    }

    let total_len = 1 + 2 + sd_jwt_len + 2 + kb_jwt_len;
    let mut payload = Vec::with_capacity(total_len);

    payload.push(version);
    payload.extend_from_slice(&(sd_jwt_len as u16).to_be_bytes());
    payload.extend_from_slice(sd_jwt_vp);
    payload.extend_from_slice(&(kb_jwt_len as u16).to_be_bytes());
    payload.extend_from_slice(key_binding_jwt);

    Ok(payload)
}

/// Decode a BLE payload back to components.
pub fn decode_ble_payload(payload: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    if payload.len() < 5 {
        return Err("Payload too short".to_string());
    }

    let version = payload[0];
    if version != 1 {
        return Err(format!("Unsupported payload version: {}", version));
    }

    let sd_jwt_len = u16::from_be_bytes([payload[1], payload[2]]) as usize;
    if payload.len() < 3 + sd_jwt_len + 2 {
        return Err("Payload truncated".to_string());
    }

    let sd_jwt = payload[3..3 + sd_jwt_len].to_vec();

    let kb_offset = 3 + sd_jwt_len;
    let kb_jwt_len = u16::from_be_bytes([payload[kb_offset], payload[kb_offset + 1]]) as usize;

    if payload.len() < kb_offset + 2 + kb_jwt_len {
        return Err("Payload truncated".to_string());
    }

    let kb_jwt = payload[kb_offset + 2..kb_offset + 2 + kb_jwt_len].to_vec();

    Ok((sd_jwt, kb_jwt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let sd_jwt = b"eyJhbGciOiJFUzI1NiJ9.test";
        let kb_jwt = b"eyJhbGciOiJFUzI1NiJ9.kb";

        let encoded = encode_ble_payload(sd_jwt, kb_jwt).unwrap();
        let (decoded_sd, decoded_kb) = decode_ble_payload(&encoded).unwrap();

        assert_eq!(decoded_sd, sd_jwt);
        assert_eq!(decoded_kb, kb_jwt);
    }

    #[test]
    fn test_version_header() {
        let sd_jwt = b"test";
        let kb_jwt = b"kb";

        let encoded = encode_ble_payload(sd_jwt, kb_jwt).unwrap();
        assert_eq!(encoded[0], 1); // version
    }
}
