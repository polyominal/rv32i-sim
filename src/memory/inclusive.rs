//! Inclusive cache implementation

use super::cache::Block;
use super::cache::CachePolicy;
use super::AccessType;
use super::Cache;
use super::StorageInterface;
use super::WriteHitPolicy;
use super::WriteMissPolicy;
use super::MMU;
use crate::error::MemoryError;
use crate::error::SimulatorResult;

/// Inclusive cache implementation.
/// We maintain n (k >= 0) caches and 1 MMU
pub struct InclusiveCache {
    pub n: usize,
    pub caches: Vec<Cache>,
    pub mmu: MMU,

    write_hit_policy: WriteHitPolicy,
    write_miss_policy: WriteMissPolicy,

    pub miss_penalty: i32,
    pub total_penalty: i32,
    pub total_worst_penalty: i32,

    pub use_victim_cache: bool,
    pub victim_cache: Cache,

    pub ref_counter: i32,
}

impl Default for InclusiveCache {
    /// Make a 3-level inclusive cache
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
            WriteHitPolicy::default(),
            WriteMissPolicy::default(),
            100,
            false,
            // true,
        )
    }
}

impl InclusiveCache {
    /// Create an inclusive cache
    /// from a vector of cache policies for each level,
    /// and write-hit and write-miss policies
    pub fn make(
        policies: Vec<CachePolicy>,
        write_hit_policy: WriteHitPolicy,
        write_miss_policy: WriteMissPolicy,
        miss_penalty: i32,
        use_victim_cache: bool,
    ) -> Self {
        let caches: Vec<_> =
            policies.iter().map(|policy| Cache::make(*policy)).collect();
        let victim_cache = if !caches.is_empty() {
            let block_size = caches[0].policy.block_size;
            Cache::make(CachePolicy::make(8 * block_size, block_size, 1, 0))
        } else {
            // Whatever it is, it's not going to be used
            Cache::make(CachePolicy::default())
        };
        Self {
            n: policies.len(),
            caches,
            mmu: MMU::make(),
            write_hit_policy,
            write_miss_policy,
            miss_penalty,
            total_penalty: 0,
            total_worst_penalty: 0,
            use_victim_cache,
            victim_cache,
            ref_counter: 0,
        }
    }

    /// Write a block to the victim cache
    fn write_block_to_victim_cache(&mut self, block: &Block) {
        let address = self.caches[0].get_address(block);
        let index_to_replace = self.victim_cache.get_index(address);
        self.victim_cache.blocks[index_to_replace] = block.clone();
        // Must fix fields
        // Note that the block must from L1
        self.victim_cache.fix_block(index_to_replace, address);
    }

    /// Fetche a block from the next level
    /// and return the target cache index,
    /// since this is usually called
    /// when a block is not found in the current cache
    fn fetch_from_next_level(
        &mut self,
        k: usize,
        address: u32,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<usize> {
        assert!(k < self.n());

        // If we enable victim cache and we're at k = 0,
        // we shall first check the victim cache
        if self.use_victim_cache && k == 0 {
            if let Some(hit_index) = self.victim_cache.lookup(address) {
                // A hit at the victim cache
                if let Some(stall_count) = stall_count {
                    *stall_count = self.victim_cache.policy.hit_latency;
                    // Record the hit
                    self.victim_cache.record_hit();
                }

                // Swap the hit block with a block in L1
                let victim_address = self
                    .victim_cache
                    .get_address(&self.victim_cache.blocks[hit_index]);
                let index = self.caches[0].get_index(victim_address);
                let index_to_replace =
                    self.caches[k].get_index_to_replace(index);
                let replaced_address = self.caches[k]
                    .get_address(&self.caches[k].blocks[index_to_replace]);
                assert_eq!(index, index_to_replace);
                assert_eq!(index, self.caches[0].get_index(address));
                std::mem::swap(
                    &mut self.caches[0].blocks[index],
                    &mut self.victim_cache.blocks[hit_index],
                );
                // Must fix fields
                self.caches[0].fix_block(index, victim_address);
                self.victim_cache.fix_block(hit_index, replaced_address);

                return Ok(index_to_replace);
            } else {
                // Record the miss
                self.victim_cache.record_miss();
            }
        }

        // Make a new block and replace some
        // evicted one
        let block = self.caches[k].make_block(address);

        // Access the next level
        self.access_inner(k + 1, address, AccessType::Write, stall_count)?;

        // Replace the block with the least recent reference
        let index_to_replace = self.caches[k].get_index_to_replace(block.index);

        // Make sure this replacement is needed
        // assert!(block.tag != self.caches[k].blocks[index_to_replace].tag);
        assert!(self.lookup(k, address).is_none());

        // Replace the block
        let replaced_block = std::mem::replace(
            &mut self.caches[k].blocks[index_to_replace],
            block,
        );

        // If we enable victim cache and we're at k = 0,
        // we write the replaced block to the victim cache
        if self.use_victim_cache && k == 0 && replaced_block.valid {
            self.write_block_to_victim_cache(&replaced_block);
        }

        // If this block is using write-back policy,
        // and the replaced block is dirty,
        // write it to the next_level
        if self.write_hit_policy == WriteHitPolicy::WriteBack
            && replaced_block.dirty
        {
            self.write_to_next_level(k, &replaced_block)?;
        }

        Ok(index_to_replace)
    }

    pub fn verify_inclusiveness(&mut self) -> SimulatorResult<()> {
        if !self.use_victim_cache {
            for k in 0..self.n() {
                for i in 0..self.caches[k].policy.block_num {
                    if !self.caches[k].blocks[i].valid {
                        continue;
                    }
                    let address =
                        self.caches[k].get_address(&self.caches[k].blocks[i]);

                    for k2 in k + 1..self.n() {
                        if self.caches[k2].lookup(address).is_none() {
                            return Err(MemoryError::CacheInconsistency(
                            k2,
                            format!("Cache level {} does not contain address {:#010x} found in level {}", 
                                    k2, address, k)
                        ).into());
                        }
                    }
                }
            }
        } else {
            for k in 1..self.n() {
                for i in 0..self.caches[k].policy.block_num {
                    if !self.caches[k].blocks[i].valid {
                        continue;
                    }
                    let address =
                        self.caches[k].get_address(&self.caches[k].blocks[i]);

                    for k2 in k + 1..self.n() {
                        if self.caches[k2].lookup(address).is_none() {
                            return Err(MemoryError::CacheInconsistency(
                            k2,
                            format!("Cache level {} does not contain address {:#010x} found in level {}", 
                                    k2, address, k)
                        ).into());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl StorageInterface for InclusiveCache {
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

    /// Must take victim cache into account
    fn get_amat(&mut self) -> f64 {
        let mut result = self.miss_penalty() as f64;
        for k in (0..self.n()).rev() {
            // If we use a victim cache
            if k == 0 && self.use_victim_cache {
                // Need to access lower level caches
                // only if vc misses
                let vc = &self.victim_cache;
                eprintln!("vc: {:?}", vc.history);
                result =
                    vc.policy.hit_latency as f64 + vc.get_miss_rate() * result;
            }

            let cache = &self.caches(k);
            eprintln!("k = {}: {:?}", k, cache.history);
            result = cache.policy.hit_latency as f64
                + cache.get_miss_rate() * result;
        }
        result
    }

    fn handle_hit(
        &mut self,
        k: usize,
        address: u32,
        access_type: AccessType,
        _: &mut Option<i32>,
    ) -> SimulatorResult<()> {
        // If it's a write and we use write-through,
        // we must write to the next level immediately
        if access_type == AccessType::Write
            && self.write_hit_policy == WriteHitPolicy::WriteThrough
        {
            // This is an inclusive cache,
            // so the address must be present in the next level.
            // In addition, we've already done the write
            // instruction and there is no need to penalize further;
            // that is, we won't pass the stall counter to this write
            self.access_inner(k + 1, address, AccessType::Write, &mut None)?;
        };
        Ok(())
    }

    fn handle_miss(
        &mut self,
        k: usize,
        address: u32,
        access_type: AccessType,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<Option<usize>> {
        // Fetch from some lower-level cache iff
        // 1. It's a read, or
        // 2. It's a write and we use write-allocate
        // target_index = Some(
        //     self.fetch_from_next_level(k, address, stall_count)
        // );
        Ok(
            if access_type == AccessType::Read
                || self.write_miss_policy == WriteMissPolicy::WriteAllocate
            {
                Some(self.fetch_from_next_level(k, address, stall_count)?)
            } else {
                self.access_inner(
                    k + 1,
                    address,
                    AccessType::Write,
                    stall_count,
                )?;
                None
            },
        )
    }
}
