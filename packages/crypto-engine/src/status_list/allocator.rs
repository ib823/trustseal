//! Index allocation for Status List credentials.
//!
//! Randomized index allocation prevents correlation attacks where
//! an observer could link credentials based on sequential indices.

use std::collections::HashSet;
use std::sync::RwLock;

use rand::Rng;

/// Index allocator for status list entries.
///
/// Allocates indices randomly to prevent correlation attacks.
/// Tracks allocated indices to prevent collisions.
///
/// # Thread Safety
/// Uses `RwLock` for thread-safe access.
pub struct IndexAllocator {
    /// Maximum index (exclusive).
    max_index: usize,
    /// Set of allocated indices.
    allocated: RwLock<HashSet<usize>>,
}

impl IndexAllocator {
    /// Create a new allocator with the given capacity.
    ///
    /// # Arguments
    /// * `max_index` - Maximum index (exclusive). Typically 131,072 for 16KB lists.
    #[must_use]
    pub fn new(max_index: usize) -> Self {
        Self {
            max_index,
            allocated: RwLock::new(HashSet::new()),
        }
    }

    /// Create an allocator for a standard 16KB status list.
    #[must_use]
    pub fn standard() -> Self {
        Self::new(131_072)
    }

    /// Allocate a new random index.
    ///
    /// # Returns
    /// - `Some(index)` - A randomly allocated, previously unused index
    /// - `None` - The allocator is full (all indices allocated)
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn allocate(&self) -> Option<usize> {
        let mut allocated = self.allocated.write().unwrap();

        // Check if full
        if allocated.len() >= self.max_index {
            return None;
        }

        // For efficiency, switch strategies based on fill level
        if allocated.len() < self.max_index / 2 {
            // Less than half full: random selection with collision retry
            self.allocate_random(&mut allocated)
        } else {
            // More than half full: select from remaining indices
            self.allocate_from_remaining(&mut allocated)
        }
    }

    /// Maximum random allocation attempts before fallback.
    const MAX_RANDOM_ATTEMPTS: usize = 100;

    /// Allocate using random selection (efficient when mostly empty).
    fn allocate_random(&self, allocated: &mut HashSet<usize>) -> Option<usize> {
        let mut rng = rand::thread_rng();
        let mut attempts = 0;

        loop {
            let index = rng.gen_range(0..self.max_index);
            if allocated.insert(index) {
                return Some(index);
            }
            attempts += 1;
            if attempts >= Self::MAX_RANDOM_ATTEMPTS {
                // Fall back to sequential scan
                return self.allocate_from_remaining(&mut *allocated);
            }
        }
    }

    /// Allocate by selecting from remaining indices (efficient when mostly full).
    fn allocate_from_remaining(&self, allocated: &mut HashSet<usize>) -> Option<usize> {
        let mut rng = rand::thread_rng();

        // Build list of available indices
        let available: Vec<usize> = (0..self.max_index)
            .filter(|i| !allocated.contains(i))
            .collect();

        if available.is_empty() {
            return None;
        }

        // Select random index from available
        let idx = rng.gen_range(0..available.len());
        let selected = available[idx];

        // Insert the selected index
        allocated.insert(selected);
        Some(selected)
    }

    /// Allocate a specific index.
    ///
    /// # Returns
    /// - `true` if the index was successfully allocated
    /// - `false` if the index was already allocated or out of bounds
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn allocate_specific(&self, index: usize) -> bool {
        if index >= self.max_index {
            return false;
        }
        let mut allocated = self.allocated.write().unwrap();
        allocated.insert(index)
    }

    /// Release an index back to the pool.
    ///
    /// This is typically called when a credential is deleted (not revoked).
    /// Revoked credentials keep their index allocated.
    ///
    /// # Returns
    /// - `true` if the index was released
    /// - `false` if the index was not allocated
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn release(&self, index: usize) -> bool {
        let mut allocated = self.allocated.write().unwrap();
        allocated.remove(&index)
    }

    /// Check if an index is allocated.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn is_allocated(&self, index: usize) -> bool {
        let allocated = self.allocated.read().unwrap();
        allocated.contains(&index)
    }

    /// Get the number of allocated indices.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn allocated_count(&self) -> usize {
        let allocated = self.allocated.read().unwrap();
        allocated.len()
    }

    /// Get the number of available indices.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn available_count(&self) -> usize {
        self.max_index - self.allocated_count()
    }

    /// Check if the allocator is full.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.allocated_count() >= self.max_index
    }

    /// Get the utilization ratio (0.0 to 1.0).
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn utilization(&self) -> f64 {
        self.allocated_count() as f64 / self.max_index as f64
    }

    /// Initialize from a set of pre-allocated indices.
    ///
    /// Use this when loading state from a database.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn load_allocated(&self, indices: impl IntoIterator<Item = usize>) {
        let mut allocated = self.allocated.write().unwrap();
        for index in indices {
            if index < self.max_index {
                allocated.insert(index);
            }
        }
    }

    /// Get all allocated indices.
    ///
    /// Use this when persisting state to a database.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get_allocated(&self) -> Vec<usize> {
        let allocated = self.allocated.read().unwrap();
        allocated.iter().copied().collect()
    }
}

impl Default for IndexAllocator {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_returns_unique_indices() {
        let allocator = IndexAllocator::new(100);
        let mut indices = HashSet::new();

        for _ in 0..50 {
            let index = allocator.allocate().unwrap();
            assert!(indices.insert(index), "Duplicate index allocated");
            assert!(index < 100);
        }
    }

    #[test]
    fn allocate_specific_works() {
        let allocator = IndexAllocator::new(100);

        assert!(allocator.allocate_specific(42));
        assert!(!allocator.allocate_specific(42)); // Already allocated
        assert!(!allocator.allocate_specific(100)); // Out of bounds

        assert!(allocator.is_allocated(42));
    }

    #[test]
    fn release_works() {
        let allocator = IndexAllocator::new(100);

        allocator.allocate_specific(42);
        assert!(allocator.is_allocated(42));

        assert!(allocator.release(42));
        assert!(!allocator.is_allocated(42));
        assert!(!allocator.release(42)); // Already released
    }

    #[test]
    fn full_allocator_returns_none() {
        let allocator = IndexAllocator::new(10);

        // Allocate all indices
        for _ in 0..10 {
            assert!(allocator.allocate().is_some());
        }

        // Should be full
        assert!(allocator.is_full());
        assert_eq!(allocator.allocate(), None);
    }

    #[test]
    fn utilization_calculation() {
        let allocator = IndexAllocator::new(100);

        assert!((allocator.utilization() - 0.0).abs() < 0.001);

        for _ in 0..50 {
            allocator.allocate();
        }
        assert!((allocator.utilization() - 0.5).abs() < 0.001);
    }

    #[test]
    fn load_and_get_allocated() {
        let allocator = IndexAllocator::new(100);

        allocator.load_allocated(vec![10, 20, 30, 40]);
        assert_eq!(allocator.allocated_count(), 4);

        let indices = allocator.get_allocated();
        assert!(indices.contains(&10));
        assert!(indices.contains(&20));
        assert!(indices.contains(&30));
        assert!(indices.contains(&40));
    }

    #[test]
    fn standard_size() {
        let allocator = IndexAllocator::standard();
        assert_eq!(allocator.available_count(), 131_072);
    }
}
