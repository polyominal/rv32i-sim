//! Utility functions for preparing the CPU and memory for execution

use crate::cpu::CPUState;
use crate::elf_helper::*;
use crate::memory::mmu::MMU;
use object::read::elf::*;
use std::error::Error;

/// Initializes the stack for the CPU
pub fn set_stack(
    cpu: &mut CPUState,
    mem: &mut MMU,
    stack_base: u32,
    stack_size: u32,
) {
    cpu.stack_base = stack_base;
    cpu.stack_size = stack_size;

    // Initialize SP register
    cpu.gpr[2].write(stack_base);

    // Allocate the stack memory for (stack_base - stack_size, stack_base]
    for address in stack_base - stack_size + 1..stack_base + 1 {
        mem.allocate_page(address);
        mem.set8(address, 0);
    }
}

/// Loads an ELF file for the CPU
pub fn load_elf(
    cpu: &mut CPUState,
    mem: &mut MMU,
    elf_reader: &ELFReaderType,
    elf_data: &[u8],
) -> Result<(), Box<dyn Error>> {
    let endian = get_elf_endian(elf_reader)?;

    // Set program entry
    match get_elf_entry(elf_reader) {
        Ok(entry) => {
            cpu.pc.write(entry);
        }
        Err(e) => {
            return Err(e);
        }
    }

    if cpu.policy.verbose {
        // Print the initial PC
        eprintln!("[VERBOSE] Initial PC: {:#010x}", cpu.pc.read());
    }

    // Get all segments (program headers)
    let segments = get_elf_segments(elf_reader, elf_data)?;
    for segment in segments {
        // Load the segment into memory

        // Get memory size
        let memory_size = segment.p_memsz(endian);
        // Get virtual address
        let virtual_address = segment.p_vaddr(endian);
        // Get file size
        let file_size = segment.p_filesz(endian);

        // Can't handle with 32b memory
        if virtual_address.checked_add(memory_size).is_none() {
            return Err("Memory address out of bounds".into());
        }

        if cpu.policy.verbose {
            eprintln!("[VERBOSE] Loading segment:");
            eprintln!("[VERBOSE] Virtual address: {:#010x}", virtual_address);
            eprintln!("[VERBOSE] Memory size: {:#010x}", memory_size);
            eprintln!("[VERBOSE] File size: {:#010x}", file_size);
            eprintln!();
        }

        for address in virtual_address..virtual_address + memory_size {
            // Allocate the page if it doesn't exist
            if !mem.page_exists(address) {
                mem.allocate_page(address);
            }

            assert!(mem.page_exists(address));

            // If this is in the file
            let file_offset = address - virtual_address;
            if file_offset < file_size {
                // Get the byte from the file
                let byte = elf_data
                    [segment.p_offset(endian) as usize + file_offset as usize];
                // Set the byte in memory
                mem.set8(address, byte);
            } else {
                // Otherwise, set the byte to 0
                mem.set8(address, 0);
            }
        }
    }

    return Ok(());
}
