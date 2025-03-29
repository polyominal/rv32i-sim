//! Cache implementation

use super::AccessType;

pub fn get_log_2(value: u32) -> usize {
    assert!(value > 0);
    31 - value.leading_zeros() as usize
}

pub fn is_pow_2(value: u32) -> bool {
    value != 0 && value & (value - 1) == 0
}

pub fn get_mask(bits: usize) -> u32 {
    (1 << bits) - 1
}

/// Cache implementation
pub struct Cache {
    pub policy: CachePolicy,

    pub history: CacheHistory,

    // Constants
    offset_bits: usize,
    index_bits: usize,

    pub offset_mask: u32,
    pub index_mask: u32,
    pub tag_mask: u32,

    pub blocks: Vec<Block>,
}

// Assume that address is 32-bit
// and looks like this:
// | tag | index | offset |
impl Cache {
    pub fn make(policy: CachePolicy) -> Self {
        assert!(policy.is_valid());

        let offset_bits = get_log_2(policy.block_size as u32);
        let index_bits =
            get_log_2((policy.block_num / policy.associativity) as u32);
        let offset_mask = get_mask(offset_bits);
        let index_mask = get_mask(index_bits);
        let tag_mask = get_mask(32 - offset_bits - index_bits);

        // Initialize blocks
        let mut blocks = vec![Block::default(); policy.block_num];
        for i in 0..blocks.len() {
            blocks[i].index = i / policy.associativity;
        }

        Self {
            policy: policy,
            history: CacheHistory::default(),
            offset_bits,
            index_bits,
            offset_mask,
            index_mask,
            tag_mask,
            blocks,
        }
    }

    /// Make a new block with the given address,
    /// usually used when loading a block with specified data
    pub fn make_block(&self, address: u32) -> Block {
        Block {
            valid: true,
            dirty: false,
            tag: self.get_tag(address),
            index: self.get_index(address),
            prv_ref: 0,
        }
    }

    pub fn reset_block(&mut self, i: usize) {
        let block = &mut self.blocks[i];
        block.valid = false;
        block.dirty = false;
        block.tag = 0;
        block.index = i / self.policy.associativity;
        block.prv_ref = 0;
    }

    /// Computes the current miss rate of the cache
    pub fn get_miss_rate(&self) -> f64 {
        (self.history.num_miss as f64)
            / ((self.history.num_hit + self.history.num_miss) as f64)
    }

    /// Given a block that is not necessarily
    /// from this cache, transform it so that it follows the
    /// format of this cache
    pub fn transformed_block(&self, block: &Block, address: u32) -> Block {
        let mut transformed_block = block.clone();
        // Must update tag and index
        transformed_block.tag = self.get_tag(address);
        transformed_block.index = self.get_index(address);
        transformed_block
    }

    pub fn fix_block(&mut self, i: usize, address: u32) {
        let tag = self.get_tag(address);
        let index = self.get_index(address);
        let block = &mut self.blocks[i];
        block.tag = tag;
        block.index = index;
        assert!(index == i / self.policy.associativity);
    }

    pub fn validate_block(&self, i: usize) -> bool {
        self.blocks[i].index == i / self.policy.associativity
    }

    pub fn get_index(&self, address: u32) -> usize {
        ((address >> self.offset_bits) & self.index_mask) as usize
    }

    pub fn get_tag(&self, address: u32) -> u32 {
        (address >> (self.offset_bits + self.index_bits)) & self.tag_mask
    }

    pub fn get_address(&self, block: &Block) -> u32 {
        (block.tag << (self.offset_bits + self.index_bits))
            | ((block.index as u32) << self.offset_bits)
    }

    pub fn is_in_cache(&self, address: u32) -> bool {
        self.lookup(address).is_some()
    }

    pub fn lookup(&self, address: u32) -> Option<usize> {
        let tag = self.get_tag(address);
        let index = self.get_index(address);
        let begin = index * self.policy.associativity;
        let end = (index + 1) * self.policy.associativity;
        for i in begin..end {
            let block = self.blocks.get(i).unwrap();
            // Ensure block index consistency
            assert!(index == block.index);
            // Return the block index if it's valid and has the same tag
            if block.valid && block.tag == tag {
                return Some(i);
            }
        }

        None
    }

    pub fn record_hit(&mut self) {
        self.history.num_hit += 1;
    }

    pub fn record_miss(&mut self) {
        self.history.num_miss += 1;
    }

    pub fn get_index_to_replace(&self, index: usize) -> usize {
        let begin = index * self.policy.associativity;
        let end = (index + 1) * self.policy.associativity;
        assert!(begin < end);
        let mut result = begin;
        let mut min_ref = self.blocks[begin].prv_ref;
        for i in begin..end {
            let block = &self.blocks.get(i).unwrap();
            // If it's not valid, replace it immediately
            if !block.valid {
                result = i;
                break;
            }
            // Otherwise, check if it's the least recent reference
            if block.prv_ref < min_ref {
                min_ref = block.prv_ref;
                result = i;
            }
        }
        result
    }

    /// Return the common block size of this cache
    pub fn get_block_size(&self) -> usize {
        self.policy.block_size
    }

    /// Return the common associativity of this cache
    pub fn get_associativity(&self) -> usize {
        self.policy.associativity
    }

    /// Access the given cache block
    pub fn access_index(
        &mut self,
        target_index: usize,
        access_type: AccessType,
        ref_counter: i32,
    ) {
        let target_block = &mut self.blocks[target_index];

        // Update reference counter
        target_block.prv_ref = ref_counter;

        // If it's a write, mark the block as dirty
        if access_type == AccessType::Write {
            target_block.dirty = true;
        }
    }
}

#[derive(Clone, Default)]
pub struct Block {
    pub valid: bool,
    pub dirty: bool,

    pub tag: u32,
    pub index: usize,

    pub prv_ref: i32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct CacheHistory {
    pub num_hit: i32,
    pub num_miss: i32,
}

#[derive(Clone, Copy)]
pub struct CachePolicy {
    pub cache_size: usize,
    pub block_size: usize,
    pub block_num: usize,
    pub associativity: usize,

    pub hit_latency: i32,
}

impl Default for CachePolicy {
    /// Default cache policy adhering
    /// to the assignment specification
    fn default() -> Self {
        Self::make(16 * 1024, 64, 1, 1)
    }
}

impl CachePolicy {
    pub fn make(
        cache_size: usize,
        block_size: usize,
        associativity: usize,
        hit_latency: i32,
    ) -> Self {
        assert!(cache_size % block_size == 0);
        Self {
            cache_size,
            block_size,
            block_num: cache_size / block_size,
            associativity,
            hit_latency,
        }
    }

    pub fn is_valid(&self) -> bool {
        // Cache size must be a power of 2
        if !is_pow_2(self.cache_size as u32) {
            return false;
        }
        // Block size must be a power of 2
        if !is_pow_2(self.block_size as u32) {
            return false;
        }
        // Cache size must be a multiple of block size
        if self.cache_size % self.block_size != 0 {
            return false;
        }
        // cache_size = block_size * block_num
        if self.cache_size != self.block_size * self.block_num {
            return false;
        }
        // Block number must be a multiple of associativity
        if self.block_num % self.associativity != 0 {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_log_2() {
        for n in 1..1234567 {
            let expected = {
                let mut count = 0;
                let mut t = n;
                while t > 1 {
                    count += 1;
                    t >>= 1;
                }
                count
            };
            assert_eq!(expected, get_log_2(n));
        }
    }

    #[test]
    fn test_is_valid() {
        let policy = CachePolicy::default();
        assert_eq!(policy.is_valid(), true);
    }
}
