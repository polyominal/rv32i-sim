//! Memory management unit implemented
//! with a two-level page table

use crate::error::MemoryError;
use crate::error::MemoryErrorKind;
use crate::error::SimulatorResult;

const WORD_WIDTH: usize = 32;
const FIRST_LEVEL_WIDTH: usize = 10;
const SECOND_LEVEL_WIDTH: usize = 10;
const PAGE_WIDTH: usize = 12;

const FIRST_LEVEL_SIZE: usize = 1 << FIRST_LEVEL_WIDTH;
const SECOND_LEVEL_SIZE: usize = 1 << SECOND_LEVEL_WIDTH;
const PAGE_SIZE: usize = 1 << PAGE_WIDTH;

// Defines page type
type PageType = Box<[u8; PAGE_SIZE]>;

/// Memory management unit
pub struct MMU {
    // Address are in u32
    // data[x][y][z] stores the byte (u8) at (x << 22) | (y << 12) | z
    // Allocate stuff lazily
    data: Vec<Option<Vec<Option<PageType>>>>,
}

impl MMU {
    /// Make a new MMU
    pub fn make() -> Self {
        Self { data: vec![None; FIRST_LEVEL_SIZE] }
    }

    /// The first-level index of the address
    pub fn get_first_level_index(address: u32) -> usize {
        (address >> (WORD_WIDTH - FIRST_LEVEL_WIDTH)) as usize
    }

    /// The second-level index of the address
    pub fn get_second_level_index(address: u32) -> usize {
        ((address >> (WORD_WIDTH - FIRST_LEVEL_WIDTH - SECOND_LEVEL_WIDTH))
            & ((SECOND_LEVEL_SIZE - 1) as u32)) as usize
    }

    /// The page offset (third-level?)
    pub fn get_page_offset(address: u32) -> usize {
        (address & ((PAGE_SIZE - 1) as u32)) as usize
    }

    /// Check if a page is allocated at the given address
    pub fn page_exists(&self, address: u32) -> bool {
        let (i, j) = (
            Self::get_first_level_index(address),
            Self::get_second_level_index(address),
        );

        if let Some(second_level) = &self.data[i] {
            // If the second level exists, check if the page exists
            second_level[j].is_some()
        } else {
            false
        }
    }

    /// Allocate a page of memory at the given address.
    /// Returns true if the allocation was successful, false if the page was
    /// already allocated
    pub fn allocate_page(&mut self, address: u32) -> bool {
        let (i, j) = (
            Self::get_first_level_index(address),
            Self::get_second_level_index(address),
        );

        // Allocate the second level if it doesn't exist
        if self.data[i].is_none() {
            self.data[i] = Some(vec![None; SECOND_LEVEL_SIZE]);
        }

        // Now the second level must exist
        if let Some(second_level) = &mut self.data[i] {
            // Allocate the page if it doesn't exist
            if second_level[j].is_none() {
                second_level[j] = Some(Box::new([0; PAGE_SIZE]));
                true
            } else {
                false
            }
        } else {
            // This branch should be unreachable because we just allocated the
            // second level
            false
        }
    }

    /// Set the byte starting at the given address
    pub fn set8(&mut self, address: u32, byte: u8) -> SimulatorResult<()> {
        let (i, j, k) = (
            Self::get_first_level_index(address),
            Self::get_second_level_index(address),
            Self::get_page_offset(address),
        );

        if let Some(second_level) = &mut self.data[i] {
            if let Some(page) = &mut second_level[j] {
                page[k] = byte;
                return Ok(());
            }
        }

        Err(MemoryError::AccessError {
            address,
            kind: MemoryErrorKind::WriteUnallocated,
        }
        .into())
    }

    /// Get the byte starting at the given address
    pub fn get8(&mut self, address: u32) -> SimulatorResult<u8> {
        let (i, j, k) = (
            Self::get_first_level_index(address),
            Self::get_second_level_index(address),
            Self::get_page_offset(address),
        );

        if let Some(second_level) = &self.data[i] {
            if let Some(page) = &second_level[j] {
                return Ok(page[k]);
            }
        }

        Err(MemoryError::AccessError {
            address,
            kind: MemoryErrorKind::ReadUnallocated,
        }
        .into())
    }

    /// Set a 16-bit value at the given address
    pub fn set16(&mut self, address: u32, value: u16) -> SimulatorResult<()> {
        if address % 2 != 0 {
            return Err(MemoryError::AlignmentError(address, 2).into());
        }

        self.set8(address, value as u8)?;
        self.set8(address + 1, (value >> 8) as u8)?;

        Ok(())
    }

    /// Get a 16-bit value from the given address
    pub fn get16(&mut self, address: u32) -> SimulatorResult<u16> {
        if address % 2 != 0 {
            return Err(MemoryError::AlignmentError(address, 2).into());
        }

        let low = self.get8(address)? as u16;
        let high = self.get8(address + 1)? as u16;

        Ok(low | (high << 8))
    }

    /// Set a 32-bit value at the given address
    pub fn set32(&mut self, address: u32, value: u32) -> SimulatorResult<()> {
        if address % 4 != 0 {
            return Err(MemoryError::AlignmentError(address, 4).into());
        }

        self.set16(address, value as u16)?;
        self.set16(address + 2, (value >> 16) as u16)?;

        Ok(())
    }

    /// Get a 32-bit value from the given address
    pub fn get32(&mut self, address: u32) -> SimulatorResult<u32> {
        if address % 4 != 0 {
            return Err(MemoryError::AlignmentError(address, 4).into());
        }

        let low = self.get16(address)? as u32;
        let high = self.get16(address + 2)? as u32;

        Ok(low | (high << 16))
    }

    pub fn dump(&self) -> Vec<(u32, Vec<u8>)> {
        let mut result = Vec::new();

        for (i, first_level) in self.data.iter().enumerate() {
            if let Some(second_level) = first_level {
                for (j, page) in second_level.iter().enumerate() {
                    if let Some(data) = page {
                        let base_address = ((i as u32)
                            << (WORD_WIDTH - FIRST_LEVEL_WIDTH))
                            | ((j as u32) << PAGE_WIDTH);
                        result.push((base_address, data.to_vec()));
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_exists() {
        let mut memory = MMU::make();
        let address = 0x12345678;

        assert!(!memory.page_exists(address));

        memory.allocate_page(address);

        assert!(memory.page_exists(address));
    }

    #[test]
    fn test_allocate_page() {
        let mut memory = MMU::make();
        let address = 0x12345678;

        assert!(memory.allocate_page(address));
        assert!(!memory.allocate_page(address));
    }

    #[test]
    fn test_set8() {
        let mut memory = MMU::make();
        let address = 0x12345678;
        let byte = 0xAB;

        memory.allocate_page(address);
        memory.set8(address, byte);

        let res = memory.get8(address);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), byte);
    }

    #[test]
    fn test_get8() {
        let mut memory = MMU::make();
        let address = 0x12345678;
        let byte = 0xAB;

        memory.allocate_page(address);
        memory.set8(address, byte);

        let res = memory.get8(address);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), byte);
    }

    #[test]
    fn test_by_hand() {
        let mut memory = MMU::make();

        assert!(!memory.page_exists(0x1000));
        assert!(memory.allocate_page(0x1000));
        assert!(!memory.page_exists(0x2000));

        {
            // set_byte and get_byte

            // Make a string "Birds aren't real"
            let s: &[u8] = b"Birds aren't real";

            // Insert all bytes into memory,
            // starting with 0x1000
            for i in 0..s.len() {
                // Get the current address
                let current_address = 0x1000_u32 + (i as u32);
                let res = memory.set8(current_address, s[i]);
                assert!(res.is_ok());
            }

            // Ensure content
            for i in 0..s.len() {
                // Get the current address
                let current_address = 0x1000_u32 + (i as u32);
                let res = memory.get8(current_address);
                assert!(res.is_ok());
            }
        }
    }
}
