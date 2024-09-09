//! Inclusive cache implementation

use super::cache::{Cache, CachePolicy};
use super::mmu::MMU;
use super::{AccessType, StorageInterface};

/// Exclusive cache implementation.
/// We maintain n (k >= 0) caches and 1 MMU
pub struct ExclusiveCache {
    pub n: usize,
    pub caches: Vec<Cache>,
    pub mmu: MMU,

    pub miss_penalty: i32,
    pub total_penalty: i32,
    pub total_worst_penalty: i32,

    pub ref_counter: i32,
}

impl Default for ExclusiveCache {
    /// Make a 3-level exclusive cache
    /// with policies adhering to the assignment specification
    fn default() -> Self {
        Self::make(
            vec![
                CachePolicy::make(16 * 1024, 64, 1, 1),
                CachePolicy::make(128 * 1024, 64, 8, 8),
                CachePolicy::make(2 * 1024 * 1024, 64, 16, 20),
                // CachePolicy::make(32 * 1024, 64, 8, 1),
                // CachePolicy::make(256 * 1024, 64, 8, 8),
                // CachePolicy::make(8 * 1024 * 1024, 64, 8, 20),
            ],
            100,
        )
    }
}

impl ExclusiveCache {
    /// Create an exclusive cache
    /// from a vector of cache policies for each level,
    pub fn make(policies: Vec<CachePolicy>, miss_penalty: i32) -> Self {
        let caches: Vec<_> =
            policies.iter().map(|policy| Cache::make(*policy)).collect();
        Self {
            n: policies.len(),
            caches,
            mmu: MMU::make(),
            miss_penalty,
            total_penalty: 0,
            total_worst_penalty: 0,
            ref_counter: 0,
        }
    }

    pub fn verify_exclusiveness(&mut self) {
        for k in 0..self.n() {
            for i in 0..self.caches[k].policy.block_num {
                if !self.caches[k].blocks[i].valid {
                    continue;
                }
                let address = self.caches[k].get_address(&self.caches[k].blocks[i]);

                for k2 in k + 1..self.n() {
                    assert_eq!(self.lookup(k2, address), None);
                }
            }
        }
    }
}

impl StorageInterface for ExclusiveCache {
    fn n(&self) -> usize {
        self.n
    }
    fn caches(&mut self, k: usize) -> &mut Cache {
        self.caches.get_mut(k).unwrap()
    }
    fn mmu(&mut self) -> &mut MMU {
        &mut self.mmu
    }
    fn ref_counter(&mut self) -> &mut i32 {
        &mut self.ref_counter
    }

    fn total_penalty(&mut self) -> &mut i32 {
        &mut self.total_penalty
    }
    fn total_worst_penalty(&mut self) -> &mut i32 {
        &mut self.total_worst_penalty
    }
    fn miss_penalty(&self) -> i32 {
        self.miss_penalty
    }

    /// Essentially we're swapping
    /// the block all the way up to the top
    fn handle_miss(
        &mut self,
        k: usize,
        address: u32,
        access_type: AccessType,
        stall_count: &mut Option<i32>,
    ) -> Option<usize> {
        assert!(k < self.n());

        // Make a new block and replace some
        // evicted one
        let block = self.caches[k].make_block(address);

        // Replace the block with the least recent reference
        let index_to_replace = self.caches[k].get_index_to_replace(block.index);

        // Make sure this replacement is needed
        // assert!(block.tag != self.caches[k].blocks[index_to_replace].tag);
        assert!(self.lookup(k, address).is_none());

        // Replace the block
        let replaced_block =
            std::mem::replace(&mut self.caches[k].blocks[index_to_replace], block);

        // If there is a next level cache,
        // we clear up the block in the next level,
        // and write the replaced block (if valid) to the next level
        if let Some(next_index) =
            self.access_inner(k + 1, address, access_type, stall_count)
        {
            // Clear up the block in the next level
            self.caches[k + 1].reset_block(next_index);

            // Write a valid block to the next level
            if replaced_block.valid {
                self.write_to_next_level(k, &replaced_block);
            }
        }

        Some(index_to_replace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclusive_cache() {
        let address = 0x12345679;
        let value = 1234;

        let mut cache = ExclusiveCache::default();
        cache.mmu().allocate_page(address);

        let mut stall_count = Some(0);
        cache.set(address, 4, value, &mut stall_count, &mut None);
        // Cold miss
        assert!(stall_count == Some(100));
        cache.set(address, 4, value, &mut stall_count, &mut None);
        // Hit at L1
        assert!(stall_count == Some(1));
    }
}
