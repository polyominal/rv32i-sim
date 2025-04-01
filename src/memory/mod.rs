//! Memory interface trait

pub mod cache;
pub mod exclusive;
pub mod inclusive;
pub mod mmu;

use crate::error::MemoryError;
use crate::error::MemoryErrorKind;
use crate::error::SimulatorResult;
use crate::memory::cache::Block;
use crate::memory::cache::Cache;
use crate::memory::cache::CacheHistory;
use crate::memory::mmu::MMU;

/// Memory interface implementation
pub trait StorageInterface {
    /// Hit handler that returns nothing
    fn handle_hit(
        &mut self,
        _: usize,
        _: u32,
        _: AccessType,
        _: &mut Option<i32>,
    ) -> SimulatorResult<()> {
        Ok(())
    }

    /// Miss handler that returns the index of the block
    /// with the specified address
    fn handle_miss(
        &mut self,
        k: usize,
        address: u32,
        access_type: AccessType,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<Option<usize>>;

    fn get8(
        &mut self,
        address: u32,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<u8> {
        self.penalize_worst();
        self.access(address, AccessType::Read, stall_count)?;
        self.mmu().get8(address)
    }

    fn set8(
        &mut self,
        address: u32,
        value: u8,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<()> {
        self.penalize_worst();
        self.access(address, AccessType::Write, stall_count)?;
        self.mmu().set8(address, value)
    }

    fn access(
        &mut self,
        address: u32,
        access_type: AccessType,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<()> {
        *self.ref_counter() += 1;
        self.access_inner(0, address, access_type, stall_count)?;
        Ok(())
    }

    /// Access the cache and return
    /// the block index in the k-th cache
    fn access_inner(
        &mut self,
        k: usize,
        address: u32,
        access_type: AccessType,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<Option<usize>> {
        if k == self.n() {
            // Access MMU, which is the worst case
            if let Some(stall_count) = stall_count {
                *stall_count = self.miss_penalty();
            }
            Ok(None)
        } else {
            // Attempt to access the k-th level cache
            let target_index: Option<usize>;
            if let Some(hit_index) = self.lookup(k, address) {
                // A hit at this level
                target_index = Some(hit_index);
                if let Some(stall_count) = stall_count {
                    *stall_count = self.penalty(k);
                    // Record the hit
                    self.caches(k).record_hit();
                }

                self.handle_hit(k, address, access_type, stall_count)?;
            } else {
                // A miss at this level
                if let Some(stall_count) = stall_count {
                    *stall_count = self.penalty(k);
                    // Record the miss
                    self.caches(k).record_miss();
                }

                target_index =
                    self.handle_miss(k, address, access_type, stall_count)?;
            }

            // Access the cache
            if let Some(target_index) = target_index {
                let ref_counter = *self.ref_counter();
                self.caches(k).access_index(
                    target_index,
                    access_type,
                    ref_counter,
                );
            }

            Ok(target_index)
        }
    }

    /// Lookup the cache at level k
    fn lookup(&mut self, k: usize, address: u32) -> Option<usize> {
        if k >= self.n() {
            return None;
        }
        self.caches(k).lookup(address)
    }

    /// Write a block to the next level
    fn write_to_next_level(
        &mut self,
        k: usize,
        block: &Block,
    ) -> SimulatorResult<()> {
        if k >= self.n() {
            return Ok(());
        }

        let block_size: usize;
        let address: u32;
        {
            // Borrow the cache at this level
            let cache = &self.caches(k);
            block_size = cache.policy.block_size;
            address = cache.get_address(block);
        }
        for i in 0..block_size {
            self.access_inner(
                k + 1,
                address + i as u32,
                AccessType::Write,
                &mut None,
            )?;
        }

        Ok(())
    }

    fn get16(
        &mut self,
        address: u32,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<u16> {
        let low = self.get8(address, stall_count)? as u16;
        let high = self.get8(address + 1, &mut None)? as u16;

        Ok(low | (high << 8))
    }

    fn get32(
        &mut self,
        address: u32,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<u32> {
        let low = self.get16(address, stall_count)? as u32;
        let high = self.get16(address + 2, &mut None)? as u32;

        Ok(low | (high << 16))
    }

    fn set16(
        &mut self,
        address: u32,
        value: u16,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<()> {
        self.set8(address, value as u8, stall_count)?;
        self.set8(address + 1, (value >> 8) as u8, &mut None)?;

        Ok(())
    }

    fn set32(
        &mut self,
        address: u32,
        value: u32,
        stall_count: &mut Option<i32>,
    ) -> SimulatorResult<()> {
        self.set16(address, value as u16, stall_count)?;
        self.set16(address + 2, (value >> 16) as u16, &mut None)?;

        Ok(())
    }

    fn get(
        &mut self,
        address: u32,
        step: u32,
        stall_count: &mut Option<i32>,
        stall_count_worst: &mut Option<i32>,
    ) -> SimulatorResult<u32> {
        if let Some(stall_count_worst) = stall_count_worst {
            *stall_count_worst = self.miss_penalty();
        }

        match step {
            1 => Ok(self.get8(address, stall_count)? as u32),
            2 => Ok(self.get16(address, stall_count)? as u32),
            4 => self.get32(address, stall_count),
            _ => Err(MemoryError::AccessError {
                address,
                kind: MemoryErrorKind::InvalidSize(step),
            }
            .into()),
        }
    }

    fn set(
        &mut self,
        address: u32,
        step: u32,
        value: u32,
        stall_count: &mut Option<i32>,
        stall_count_worst: &mut Option<i32>,
    ) -> SimulatorResult<()> {
        if let Some(stall_count_worst) = stall_count_worst {
            *stall_count_worst += self.miss_penalty();
        }

        match step {
            1 => self.set8(address, value as u8, stall_count),
            2 => self.set16(address, value as u16, stall_count),
            4 => self.set32(address, value, stall_count),
            _ => Err(MemoryError::AccessError {
                address,
                kind: MemoryErrorKind::InvalidSize(step),
            }
            .into()),
        }
    }

    fn caches(&mut self, k: usize) -> &mut Cache;
    fn n(&self) -> usize;
    fn mmu(&mut self) -> &mut MMU;
    fn ref_counter(&mut self) -> &mut i32;

    fn total_penalty(&mut self) -> &mut i32;
    fn total_worst_penalty(&mut self) -> &mut i32;
    fn miss_penalty(&self) -> i32;

    fn penalty(&mut self, k: usize) -> i32 {
        if k < self.n() {
            self.caches(k).policy.hit_latency
        } else {
            self.miss_penalty()
        }
    }

    /// Penalize for hitting something lower than level k
    fn penalize(&mut self, k: usize) {
        *self.total_penalty() += self.penalty(k);
    }

    /// Penalize assuming the worst case (main memory access)
    fn penalize_worst(&mut self) {
        *self.total_worst_penalty() += self.miss_penalty();
    }

    /// Return the list of cache histories
    fn get_history(&mut self) -> Vec<CacheHistory> {
        let mut histories = Vec::new();
        for k in 0..self.n() {
            histories.push(self.caches(k).history);
        }
        histories
    }

    fn get_amat(&mut self) -> f64 {
        let mut result = self.miss_penalty() as f64;
        for k in (0..self.n()).rev() {
            let cache = &self.caches(k);
            let miss_rate = cache.get_miss_rate();
            result = cache.policy.hit_latency as f64 + miss_rate * result;
        }

        result
    }
}

/// Reference: <https://inst.eecs.berkeley.edu/~cs61c/su20/pdfs/lectures/lec15.pdf>
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum WriteHitPolicy {
    #[default]
    WriteBack,
    WriteThrough,
}

/// Reference: <https://inst.eecs.berkeley.edu/~cs61c/su20/pdfs/lectures/lec15.pdf>
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum WriteMissPolicy {
    #[default]
    WriteAllocate,
    WriteNoAllocate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessType {
    Read,
    Write,
}
