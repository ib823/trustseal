//! Bitstring implementation for Status List.
//!
//! Reference: W3C Bitstring Status List v1.0
//! <https://www.w3.org/TR/vc-bitstring-status-list/>

use std::io::{Read, Write};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

/// A compressed bitstring for credential status tracking.
///
/// Each bit represents the status of one credential:
/// - 0 = valid (not revoked)
/// - 1 = revoked
///
/// The bitstring is stored compressed (gzip) and encoded as base64
/// for transmission in the Status List Credential.
#[derive(Debug, Clone)]
pub struct Bitstring {
    /// The raw bits (uncompressed).
    bits: Vec<u8>,
    /// Number of status entries (bits) in this list.
    size: usize,
}

impl Bitstring {
    /// Create a new bitstring with the given capacity.
    ///
    /// # Arguments
    /// * `size` - Number of status entries (bits). Rounded up to byte boundary.
    ///
    /// # Example
    /// ```ignore
    /// // Create a status list for 131,072 credentials (16KB)
    /// let bitstring = Bitstring::new(131_072);
    /// ```
    #[must_use]
    pub fn new(size: usize) -> Self {
        let byte_size = size.div_ceil(8);
        Self {
            bits: vec![0u8; byte_size],
            size,
        }
    }

    /// Create a 16KB bitstring (131,072 entries) as per MASTER_PLAN spec.
    #[must_use]
    pub fn standard() -> Self {
        Self::new(131_072)
    }

    /// Get the status of a credential at the given index.
    ///
    /// # Returns
    /// - `true` if revoked (bit is 1)
    /// - `false` if valid (bit is 0)
    /// - `None` if index is out of bounds
    #[must_use]
    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.size {
            return None;
        }
        let byte_index = index / 8;
        let bit_index = 7 - (index % 8); // MSB first
        Some((self.bits[byte_index] >> bit_index) & 1 == 1)
    }

    /// Set the status of a credential at the given index.
    ///
    /// # Arguments
    /// * `index` - The credential's status list index
    /// * `revoked` - `true` to mark as revoked, `false` to mark as valid
    ///
    /// # Returns
    /// `true` if the index was valid, `false` if out of bounds.
    pub fn set(&mut self, index: usize, revoked: bool) -> bool {
        if index >= self.size {
            return false;
        }
        let byte_index = index / 8;
        let bit_index = 7 - (index % 8); // MSB first

        if revoked {
            self.bits[byte_index] |= 1 << bit_index;
        } else {
            self.bits[byte_index] &= !(1 << bit_index);
        }
        true
    }

    /// Revoke a credential (set bit to 1).
    ///
    /// # Returns
    /// `true` if successful, `false` if index out of bounds.
    pub fn revoke(&mut self, index: usize) -> bool {
        self.set(index, true)
    }

    /// Unrevoke a credential (set bit to 0).
    ///
    /// Note: This should be used with caution. Once revoked, credentials
    /// typically should not be unrevoked.
    ///
    /// # Returns
    /// `true` if successful, `false` if index out of bounds.
    pub fn unrevoke(&mut self, index: usize) -> bool {
        self.set(index, false)
    }

    /// Check if a credential is revoked.
    ///
    /// # Returns
    /// - `Some(true)` if revoked
    /// - `Some(false)` if valid
    /// - `None` if index out of bounds
    #[must_use]
    pub fn is_revoked(&self, index: usize) -> Option<bool> {
        self.get(index)
    }

    /// Returns the number of status entries (bits) in this list.
    #[must_use]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the size in bytes (uncompressed).
    #[must_use]
    pub fn byte_size(&self) -> usize {
        self.bits.len()
    }

    /// Count the number of revoked credentials.
    #[must_use]
    pub fn revoked_count(&self) -> usize {
        self.bits.iter().map(|b| b.count_ones() as usize).sum()
    }

    /// Encode the bitstring as compressed base64.
    ///
    /// This is the format used in the `encodedList` field of
    /// a Bitstring Status List Credential.
    ///
    /// # Errors
    /// Returns an error if compression fails.
    pub fn encode(&self) -> Result<String, String> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        // Compress with gzip
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&self.bits)
            .map_err(|e| format!("Compression write failed: {e}"))?;
        let compressed = encoder
            .finish()
            .map_err(|e| format!("Compression finish failed: {e}"))?;

        // Base64 encode
        Ok(STANDARD.encode(compressed))
    }

    /// Decode a bitstring from compressed base64.
    ///
    /// # Arguments
    /// * `encoded` - Base64-encoded gzip-compressed bitstring
    ///
    /// # Errors
    /// Returns an error if decoding or decompression fails.
    pub fn decode(encoded: &str) -> Result<Self, String> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        // Base64 decode
        let compressed = STANDARD
            .decode(encoded)
            .map_err(|e| format!("Base64 decode failed: {e}"))?;

        // Decompress
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut bits = Vec::new();
        decoder
            .read_to_end(&mut bits)
            .map_err(|e| format!("Decompression failed: {e}"))?;

        let size = bits.len() * 8;
        Ok(Self { bits, size })
    }

    /// Create from raw bytes (uncompressed).
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let size = bytes.len() * 8;
        Self { bits: bytes, size }
    }

    /// Get the raw bytes (uncompressed).
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }
}

impl Default for Bitstring {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bitstring_all_zeros() {
        let bs = Bitstring::new(100);
        for i in 0..100 {
            assert_eq!(bs.get(i), Some(false), "Bit {i} should be 0");
        }
    }

    #[test]
    fn set_and_get_bits() {
        let mut bs = Bitstring::new(100);

        bs.set(0, true);
        bs.set(7, true);
        bs.set(8, true);
        bs.set(99, true);

        assert_eq!(bs.get(0), Some(true));
        assert_eq!(bs.get(7), Some(true));
        assert_eq!(bs.get(8), Some(true));
        assert_eq!(bs.get(99), Some(true));
        assert_eq!(bs.get(1), Some(false));
        assert_eq!(bs.get(50), Some(false));
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let bs = Bitstring::new(100);
        assert_eq!(bs.get(100), None);
        assert_eq!(bs.get(1000), None);
    }

    #[test]
    fn out_of_bounds_set_returns_false() {
        let mut bs = Bitstring::new(100);
        assert!(!bs.set(100, true));
        assert!(!bs.set(1000, true));
    }

    #[test]
    fn revoke_and_unrevoke() {
        let mut bs = Bitstring::new(100);

        assert!(bs.revoke(42));
        assert_eq!(bs.is_revoked(42), Some(true));

        assert!(bs.unrevoke(42));
        assert_eq!(bs.is_revoked(42), Some(false));
    }

    #[test]
    fn revoked_count() {
        let mut bs = Bitstring::new(100);
        assert_eq!(bs.revoked_count(), 0);

        bs.revoke(10);
        bs.revoke(20);
        bs.revoke(30);
        assert_eq!(bs.revoked_count(), 3);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let mut bs = Bitstring::new(1000);
        bs.revoke(0);
        bs.revoke(100);
        bs.revoke(500);
        bs.revoke(999);

        let encoded = bs.encode().unwrap();
        let decoded = Bitstring::decode(&encoded).unwrap();

        assert_eq!(decoded.is_revoked(0), Some(true));
        assert_eq!(decoded.is_revoked(100), Some(true));
        assert_eq!(decoded.is_revoked(500), Some(true));
        assert_eq!(decoded.is_revoked(999), Some(true));
        assert_eq!(decoded.is_revoked(1), Some(false));
        assert_eq!(decoded.is_revoked(50), Some(false));
    }

    #[test]
    fn standard_size() {
        let bs = Bitstring::standard();
        assert_eq!(bs.size(), 131_072);
        assert_eq!(bs.byte_size(), 16_384); // 16KB
    }

    #[test]
    fn msb_ordering() {
        // Verify MSB-first bit ordering per spec
        let mut bs = Bitstring::new(16);

        // Set bit 0 (MSB of first byte)
        bs.set(0, true);
        assert_eq!(bs.as_bytes()[0], 0b1000_0000);

        // Set bit 7 (LSB of first byte)
        bs.set(7, true);
        assert_eq!(bs.as_bytes()[0], 0b1000_0001);

        // Set bit 8 (MSB of second byte)
        bs.set(8, true);
        assert_eq!(bs.as_bytes()[1], 0b1000_0000);
    }
}
