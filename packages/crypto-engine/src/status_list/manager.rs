//! Status List Manager for credential revocation.
//!
//! Manages a status list including:
//! - Bitstring state
//! - Index allocation
//! - Credential generation
//! - Revocation operations

use std::sync::Arc;

use chrono::Duration;

use super::allocator::IndexAllocator;
use super::bitstring::Bitstring;
use super::credential::{BitstringStatusListEntry, StatusListCredential, StatusPurpose};

/// Manager for a Status List.
///
/// Handles index allocation, revocation tracking, and credential generation.
///
/// # Thread Safety
/// Uses `Arc` and `RwLock` internally for thread-safe access.
pub struct StatusListManager {
    /// Tenant ID this list belongs to.
    tenant_id: String,
    /// Credential type this list tracks (e.g., "ResidentBadge").
    credential_type: String,
    /// The underlying bitstring.
    bitstring: std::sync::RwLock<Bitstring>,
    /// Index allocator.
    allocator: IndexAllocator,
    /// Base URL for the status list credential.
    base_url: String,
    /// Issuer DID for signing.
    issuer_did: String,
    /// TTL for generated status list credentials.
    credential_ttl: Duration,
}

impl StatusListManager {
    /// Create a new status list manager.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier (e.g., "TNT_01HXK...")
    /// * `credential_type` - Type of credentials this list tracks
    /// * `base_url` - Base URL for status list publication
    /// * `issuer_did` - DID of the status list issuer
    #[must_use]
    pub fn new(
        tenant_id: impl Into<String>,
        credential_type: impl Into<String>,
        base_url: impl Into<String>,
        issuer_did: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            credential_type: credential_type.into(),
            bitstring: std::sync::RwLock::new(Bitstring::standard()),
            allocator: IndexAllocator::standard(),
            base_url: base_url.into(),
            issuer_did: issuer_did.into(),
            credential_ttl: Duration::minutes(15), // 900 seconds per spec
        }
    }

    /// Set the TTL for generated status list credentials.
    #[must_use]
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.credential_ttl = ttl;
        self
    }

    /// Get the status list URL.
    #[must_use]
    pub fn status_list_url(&self) -> String {
        format!(
            "{}/api/v1/status/{}/{}",
            self.base_url, self.tenant_id, self.credential_type
        )
    }

    /// Allocate a new index and create a status entry for a credential.
    ///
    /// # Arguments
    /// * `purpose` - Status purpose (typically `Revocation`)
    ///
    /// # Returns
    /// - `Some(entry)` - Status entry to embed in the credential
    /// - `None` - Status list is full
    #[must_use]
    pub fn allocate(&self, purpose: StatusPurpose) -> Option<BitstringStatusListEntry> {
        let index = self.allocator.allocate()?;
        Some(BitstringStatusListEntry::new(
            index,
            self.status_list_url(),
            purpose,
        ))
    }

    /// Allocate a revocation status entry.
    #[must_use]
    pub fn allocate_revocation(&self) -> Option<BitstringStatusListEntry> {
        self.allocate(StatusPurpose::Revocation)
    }

    /// Revoke a credential by its status list index.
    ///
    /// # Returns
    /// - `Ok(())` - Credential successfully revoked
    /// - `Err(msg)` - Index out of bounds or already revoked
    ///
    /// # Errors
    /// Returns an error if the index is invalid.
    pub fn revoke(&self, index: usize) -> Result<(), String> {
        let mut bitstring = self
            .bitstring
            .write()
            .map_err(|_| "Lock poisoned".to_string())?;

        if bitstring.is_revoked(index) == Some(true) {
            return Err(format!("Credential at index {index} already revoked"));
        }

        if !bitstring.revoke(index) {
            return Err(format!("Index {index} out of bounds"));
        }

        Ok(())
    }

    /// Revoke multiple credentials atomically.
    ///
    /// # Errors
    /// Returns an error if any index is invalid.
    pub fn revoke_batch(&self, indices: &[usize]) -> Result<(), String> {
        let mut bitstring = self
            .bitstring
            .write()
            .map_err(|_| "Lock poisoned".to_string())?;

        for &index in indices {
            if !bitstring.revoke(index) {
                return Err(format!("Index {index} out of bounds"));
            }
        }

        Ok(())
    }

    /// Check if a credential is revoked.
    ///
    /// # Returns
    /// - `Some(true)` - Revoked
    /// - `Some(false)` - Valid
    /// - `None` - Index out of bounds
    #[must_use]
    pub fn is_revoked(&self, index: usize) -> Option<bool> {
        let bitstring = self.bitstring.read().ok()?;
        bitstring.is_revoked(index)
    }

    /// Generate a signed Status List Credential.
    ///
    /// Note: This generates the credential structure but does NOT sign it.
    /// The caller must sign the credential using the appropriate key.
    ///
    /// # Errors
    /// Returns an error if the bitstring cannot be encoded.
    pub fn generate_credential(&self) -> Result<StatusListCredential, String> {
        let bitstring = self
            .bitstring
            .read()
            .map_err(|_| "Lock poisoned".to_string())?;

        StatusListCredential::for_revocation(
            &self.status_list_url(),
            self.issuer_did.clone(),
            &bitstring,
            self.credential_ttl,
        )
    }

    /// Get statistics about this status list.
    #[must_use]
    pub fn stats(&self) -> StatusListStats {
        let allocated = self.allocator.allocated_count();
        let revoked = self
            .bitstring
            .read()
            .map(|bs| bs.revoked_count())
            .unwrap_or(0);

        StatusListStats {
            tenant_id: self.tenant_id.clone(),
            credential_type: self.credential_type.clone(),
            total_capacity: 131_072,
            allocated,
            revoked,
            available: self.allocator.available_count(),
            utilization: self.allocator.utilization(),
        }
    }

    /// Load state from persisted data.
    ///
    /// # Arguments
    /// * `allocated_indices` - Previously allocated indices
    /// * `bitstring_data` - Encoded bitstring (base64 gzip)
    ///
    /// # Errors
    /// Returns an error if decoding fails.
    pub fn load_state(
        &self,
        allocated_indices: impl IntoIterator<Item = usize>,
        bitstring_data: &str,
    ) -> Result<(), String> {
        // Load allocated indices
        self.allocator.load_allocated(allocated_indices);

        // Load bitstring
        let decoded = Bitstring::decode(bitstring_data)?;
        let mut bitstring = self
            .bitstring
            .write()
            .map_err(|_| "Lock poisoned".to_string())?;
        *bitstring = decoded;

        Ok(())
    }

    /// Export state for persistence.
    ///
    /// # Errors
    /// Returns an error if encoding fails.
    pub fn export_state(&self) -> Result<ExportedState, String> {
        let bitstring = self
            .bitstring
            .read()
            .map_err(|_| "Lock poisoned".to_string())?;

        Ok(ExportedState {
            allocated_indices: self.allocator.get_allocated(),
            encoded_bitstring: bitstring.encode()?,
        })
    }
}

/// Statistics about a status list.
#[derive(Debug, Clone)]
pub struct StatusListStats {
    /// Tenant ID.
    pub tenant_id: String,
    /// Credential type.
    pub credential_type: String,
    /// Total capacity (typically 131,072).
    pub total_capacity: usize,
    /// Number of allocated indices.
    pub allocated: usize,
    /// Number of revoked credentials.
    pub revoked: usize,
    /// Number of available indices.
    pub available: usize,
    /// Utilization ratio (0.0 to 1.0).
    pub utilization: f64,
}

/// Exported state for persistence.
#[derive(Debug, Clone)]
pub struct ExportedState {
    /// All allocated indices.
    pub allocated_indices: Vec<usize>,
    /// Encoded bitstring (base64 gzip).
    pub encoded_bitstring: String,
}

/// Registry of status list managers per tenant/credential type.
pub struct StatusListRegistry {
    managers: std::sync::RwLock<std::collections::HashMap<String, Arc<StatusListManager>>>,
    base_url: String,
    issuer_did: String,
}

impl StatusListRegistry {
    /// Create a new registry.
    #[must_use]
    pub fn new(base_url: impl Into<String>, issuer_did: impl Into<String>) -> Self {
        Self {
            managers: std::sync::RwLock::new(std::collections::HashMap::new()),
            base_url: base_url.into(),
            issuer_did: issuer_did.into(),
        }
    }

    /// Get or create a manager for the given tenant and credential type.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn get_or_create(&self, tenant_id: &str, credential_type: &str) -> Arc<StatusListManager> {
        let key = format!("{tenant_id}:{credential_type}");

        // Check if exists
        {
            let managers = self.managers.read().unwrap();
            if let Some(manager) = managers.get(&key) {
                return Arc::clone(manager);
            }
        }

        // Create new
        let mut managers = self.managers.write().unwrap();
        let manager = Arc::new(StatusListManager::new(
            tenant_id,
            credential_type,
            &self.base_url,
            &self.issuer_did,
        ));
        managers.insert(key, Arc::clone(&manager));
        manager
    }

    /// Get an existing manager.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get(&self, tenant_id: &str, credential_type: &str) -> Option<Arc<StatusListManager>> {
        let key = format!("{tenant_id}:{credential_type}");
        let managers = self.managers.read().unwrap();
        managers.get(&key).cloned()
    }

    /// List all registered tenant/credential type pairs.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn list_all(&self) -> Vec<(String, String)> {
        let managers = self.managers.read().unwrap();
        managers
            .keys()
            .filter_map(|key| {
                let parts: Vec<&str> = key.split(':').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_allocate_and_revoke() {
        let manager = StatusListManager::new(
            "TNT_01HXK",
            "ResidentBadge",
            "https://sahi.my",
            "did:web:status.sahi.my",
        );

        // Allocate
        let entry = manager.allocate_revocation().unwrap();
        let index = entry.parse_index().unwrap();

        // Should not be revoked yet
        assert_eq!(manager.is_revoked(index), Some(false));

        // Revoke
        manager.revoke(index).unwrap();
        assert_eq!(manager.is_revoked(index), Some(true));
    }

    #[test]
    fn manager_generate_credential() {
        let manager = StatusListManager::new(
            "TNT_01HXK",
            "ResidentBadge",
            "https://sahi.my",
            "did:web:status.sahi.my",
        );

        let credential = manager.generate_credential().unwrap();
        assert_eq!(credential.issuer, "did:web:status.sahi.my");
        assert!(credential.id.contains("TNT_01HXK"));
        assert!(credential.id.contains("ResidentBadge"));
    }

    #[test]
    fn manager_stats() {
        let manager = StatusListManager::new(
            "TNT_01HXK",
            "ResidentBadge",
            "https://sahi.my",
            "did:web:status.sahi.my",
        );

        // Allocate some
        for _ in 0..10 {
            let _ = manager.allocate_revocation();
        }

        // Revoke some
        manager.revoke(0).ok();
        manager.revoke(1).ok();

        let stats = manager.stats();
        assert_eq!(stats.allocated, 10);
        assert_eq!(stats.revoked, 2);
        assert_eq!(stats.available, 131_072 - 10);
    }

    #[test]
    fn manager_export_import() {
        let manager1 = StatusListManager::new(
            "TNT_01HXK",
            "ResidentBadge",
            "https://sahi.my",
            "did:web:status.sahi.my",
        );

        // Allocate and revoke
        let entry = manager1.allocate_revocation().unwrap();
        let index = entry.parse_index().unwrap();
        manager1.revoke(index).unwrap();

        // Export
        let exported = manager1.export_state().unwrap();

        // Import into new manager
        let manager2 = StatusListManager::new(
            "TNT_01HXK",
            "ResidentBadge",
            "https://sahi.my",
            "did:web:status.sahi.my",
        );
        manager2
            .load_state(exported.allocated_indices, &exported.encoded_bitstring)
            .unwrap();

        // Verify state
        assert_eq!(manager2.is_revoked(index), Some(true));
    }

    #[test]
    fn registry_get_or_create() {
        let registry = StatusListRegistry::new("https://sahi.my", "did:web:status.sahi.my");

        let manager1 = registry.get_or_create("TNT_01", "ResidentBadge");
        let manager2 = registry.get_or_create("TNT_01", "ResidentBadge");

        // Should return same instance
        assert!(Arc::ptr_eq(&manager1, &manager2));

        // Different types should be different
        let manager3 = registry.get_or_create("TNT_01", "VisitorPass");
        assert!(!Arc::ptr_eq(&manager1, &manager3));
    }

    #[test]
    fn status_list_url_format() {
        let manager = StatusListManager::new(
            "TNT_01HXK",
            "ResidentBadge",
            "https://sahi.my",
            "did:web:status.sahi.my",
        );

        assert_eq!(
            manager.status_list_url(),
            "https://sahi.my/api/v1/status/TNT_01HXK/ResidentBadge"
        );
    }
}
