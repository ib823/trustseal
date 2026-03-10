//! APDU commands and NDEF parsing for NFC.

use thiserror::Error;

/// APDU errors.
#[derive(Debug, Error)]
pub enum ApduError {
    #[error("Invalid APDU format")]
    InvalidFormat,

    #[error("NDEF parse error: {0}")]
    NdefParseError(String),

    #[error("Unsupported record type: {0}")]
    UnsupportedRecordType(String),

    #[error("Response error: SW1={sw1:02X} SW2={sw2:02X}")]
    ResponseError { sw1: u8, sw2: u8 },
}

/// APDU command structure.
#[derive(Debug, Clone)]
pub struct ApduCommand {
    /// Class byte.
    pub cla: u8,
    /// Instruction byte.
    pub ins: u8,
    /// Parameter 1.
    pub p1: u8,
    /// Parameter 2.
    pub p2: u8,
    /// Command data.
    pub data: Vec<u8>,
    /// Expected response length (Le).
    pub le: Option<u8>,
}

impl ApduCommand {
    /// Create SELECT command for NDEF application.
    pub fn select_ndef_application() -> Self {
        Self {
            cla: 0x00,
            ins: 0xA4,
            p1: 0x04,
            p2: 0x00,
            data: vec![0xD2, 0x76, 0x00, 0x00, 0x85, 0x01, 0x01], // NDEF AID
            le: Some(0x00),
        }
    }

    /// Create SELECT command for capability container.
    pub fn select_cc() -> Self {
        Self {
            cla: 0x00,
            ins: 0xA4,
            p1: 0x00,
            p2: 0x0C,
            data: vec![0xE1, 0x03],
            le: None,
        }
    }

    /// Create SELECT command for NDEF file.
    pub fn select_ndef_file() -> Self {
        Self {
            cla: 0x00,
            ins: 0xA4,
            p1: 0x00,
            p2: 0x0C,
            data: vec![0xE1, 0x04],
            le: None,
        }
    }

    /// Create READ BINARY command.
    pub fn read_binary(offset: u16, length: u8) -> Self {
        Self {
            cla: 0x00,
            ins: 0xB0,
            p1: (offset >> 8) as u8,
            p2: (offset & 0xFF) as u8,
            data: Vec::new(),
            le: Some(length),
        }
    }

    /// Encode to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.cla, self.ins, self.p1, self.p2];

        if !self.data.is_empty() {
            bytes.push(self.data.len() as u8);
            bytes.extend_from_slice(&self.data);
        }

        if let Some(le) = self.le {
            bytes.push(le);
        }

        bytes
    }
}

/// APDU response structure.
#[derive(Debug, Clone)]
pub struct ApduResponse {
    /// Response data.
    pub data: Vec<u8>,
    /// Status word 1.
    pub sw1: u8,
    /// Status word 2.
    pub sw2: u8,
}

impl ApduResponse {
    /// Parse from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ApduError> {
        if bytes.len() < 2 {
            return Err(ApduError::InvalidFormat);
        }

        let len = bytes.len();
        Ok(Self {
            data: bytes[..len - 2].to_vec(),
            sw1: bytes[len - 2],
            sw2: bytes[len - 1],
        })
    }

    /// Check if response indicates success.
    pub fn is_success(&self) -> bool {
        self.sw1 == 0x90 && self.sw2 == 0x00
    }

    /// Get status word.
    pub fn status_word(&self) -> u16 {
        ((self.sw1 as u16) << 8) | (self.sw2 as u16)
    }
}

/// NDEF record parser.
pub struct NdefParser;

/// NDEF record.
#[derive(Debug, Clone)]
pub struct NdefRecord {
    /// Type Name Format.
    pub tnf: u8,
    /// Record type.
    pub record_type: Vec<u8>,
    /// Record ID.
    pub id: Vec<u8>,
    /// Payload.
    pub payload: Vec<u8>,
}

/// VaultPass NDEF External Type.
pub const VAULTPASS_NDEF_TYPE: &[u8] = b"sahi.my:vaultpass";

impl NdefParser {
    /// Parse NDEF message.
    pub fn parse(data: &[u8]) -> Result<Vec<NdefRecord>, ApduError> {
        if data.len() < 2 {
            return Err(ApduError::NdefParseError("Data too short".to_string()));
        }

        // Skip NDEF message length (first 2 bytes)
        let ndef_data = if data.len() > 2 && data[0] == 0x00 {
            &data[2..]
        } else {
            data
        };

        let mut records = Vec::new();
        let mut offset = 0;

        while offset < ndef_data.len() {
            let record = Self::parse_record(&ndef_data[offset..])?;
            let record_len = Self::record_length(&record, &ndef_data[offset..]);
            records.push(record);
            offset += record_len;

            // Check for Message End flag
            if ndef_data.get(offset.saturating_sub(record_len)).map_or(false, |b| b & 0x40 != 0) {
                break;
            }
        }

        Ok(records)
    }

    fn parse_record(data: &[u8]) -> Result<NdefRecord, ApduError> {
        if data.is_empty() {
            return Err(ApduError::NdefParseError("Empty record".to_string()));
        }

        let flags = data[0];
        let tnf = flags & 0x07;
        let sr = (flags & 0x10) != 0; // Short Record
        let il = (flags & 0x08) != 0; // ID Length present

        if data.len() < 2 {
            return Err(ApduError::NdefParseError("Record too short".to_string()));
        }

        let type_length = data[1] as usize;

        let payload_length = if sr {
            if data.len() < 3 {
                return Err(ApduError::NdefParseError("Missing payload length".to_string()));
            }
            data[2] as usize
        } else {
            if data.len() < 6 {
                return Err(ApduError::NdefParseError("Missing payload length".to_string()));
            }
            u32::from_be_bytes([data[2], data[3], data[4], data[5]]) as usize
        };

        let header_offset = if sr { 3 } else { 6 };

        let id_length = if il {
            if data.len() <= header_offset {
                return Err(ApduError::NdefParseError("Missing ID length".to_string()));
            }
            data[header_offset] as usize
        } else {
            0
        };

        let type_offset = header_offset + if il { 1 } else { 0 };
        let id_offset = type_offset + type_length;
        let payload_offset = id_offset + id_length;

        if data.len() < payload_offset + payload_length {
            return Err(ApduError::NdefParseError("Payload truncated".to_string()));
        }

        Ok(NdefRecord {
            tnf,
            record_type: data[type_offset..type_offset + type_length].to_vec(),
            id: if il {
                data[id_offset..id_offset + id_length].to_vec()
            } else {
                Vec::new()
            },
            payload: data[payload_offset..payload_offset + payload_length].to_vec(),
        })
    }

    fn record_length(record: &NdefRecord, data: &[u8]) -> usize {
        let flags = data[0];
        let sr = (flags & 0x10) != 0;
        let il = (flags & 0x08) != 0;
        let id_len_byte = if il { 1 } else { 0 };

        1 + 1 + (if sr { 1 } else { 4 })
            + id_len_byte
            + record.record_type.len()
            + record.id.len()
            + record.payload.len()
    }

    /// Check if record is a VaultPass credential.
    pub fn is_vaultpass_record(record: &NdefRecord) -> bool {
        // TNF 0x04 = External Type
        record.tnf == 0x04 && record.record_type == VAULTPASS_NDEF_TYPE
    }

    /// Extract SD-JWT from VaultPass NDEF record.
    pub fn extract_sdjwt(record: &NdefRecord) -> Result<String, ApduError> {
        if !Self::is_vaultpass_record(record) {
            return Err(ApduError::UnsupportedRecordType(
                String::from_utf8_lossy(&record.record_type).to_string(),
            ));
        }

        String::from_utf8(record.payload.clone())
            .map_err(|e| ApduError::NdefParseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apdu_select_ndef() {
        let cmd = ApduCommand::select_ndef_application();
        let bytes = cmd.to_bytes();

        assert_eq!(bytes[0], 0x00); // CLA
        assert_eq!(bytes[1], 0xA4); // INS (SELECT)
        assert_eq!(bytes[2], 0x04); // P1
        assert_eq!(bytes[3], 0x00); // P2
    }

    #[test]
    fn test_apdu_read_binary() {
        let cmd = ApduCommand::read_binary(0x0000, 0xFF);
        let bytes = cmd.to_bytes();

        assert_eq!(bytes[0], 0x00); // CLA
        assert_eq!(bytes[1], 0xB0); // INS (READ BINARY)
        assert_eq!(bytes[2], 0x00); // P1 (offset high)
        assert_eq!(bytes[3], 0x00); // P2 (offset low)
        assert_eq!(bytes[4], 0xFF); // Le
    }

    #[test]
    fn test_apdu_response_success() {
        let response = ApduResponse::from_bytes(&[0x90, 0x00]).unwrap();
        assert!(response.is_success());
        assert!(response.data.is_empty());
    }

    #[test]
    fn test_apdu_response_with_data() {
        let response = ApduResponse::from_bytes(&[0x01, 0x02, 0x03, 0x90, 0x00]).unwrap();
        assert!(response.is_success());
        assert_eq!(response.data, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_apdu_response_error() {
        let response = ApduResponse::from_bytes(&[0x6A, 0x82]).unwrap();
        assert!(!response.is_success());
        assert_eq!(response.status_word(), 0x6A82);
    }

    #[test]
    fn test_vaultpass_record_check() {
        let record = NdefRecord {
            tnf: 0x04,
            record_type: VAULTPASS_NDEF_TYPE.to_vec(),
            id: Vec::new(),
            payload: b"test.sdjwt".to_vec(),
        };

        assert!(NdefParser::is_vaultpass_record(&record));

        let non_vaultpass = NdefRecord {
            tnf: 0x01,
            record_type: b"U".to_vec(),
            id: Vec::new(),
            payload: Vec::new(),
        };

        assert!(!NdefParser::is_vaultpass_record(&non_vaultpass));
    }

    #[test]
    fn test_extract_sdjwt() {
        let record = NdefRecord {
            tnf: 0x04,
            record_type: VAULTPASS_NDEF_TYPE.to_vec(),
            id: Vec::new(),
            payload: b"eyJ0eXAiOiJzZCtqd3QifQ.test~".to_vec(),
        };

        let sdjwt = NdefParser::extract_sdjwt(&record).unwrap();
        assert!(sdjwt.starts_with("eyJ"));
    }
}
