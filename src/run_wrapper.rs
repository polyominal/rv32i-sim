//! A simulator wrapper

use crate::cpu::CPUPolicy;
use crate::cpu::CPUState;
use crate::cpu::Implementation;
use crate::elf_helper;
use crate::error::ElfError;
use crate::error::SimulatorResult;
use crate::loader;
use crate::memory::inclusive::InclusiveCache;
use crate::memory::StorageInterface;
use crate::pipelined;
use crate::single_cycle;

const STACK_BASE: u32 = 0x80000000;
const STACK_SIZE: u32 = 0x400000;

// (Ideal CPI, CPI, CPI (no caching), (CPI(no caching) / CPI))
type RunStats = (f64, f64, f64, f64);

/// Run simulation on the given ELF file
/// and return the exit PC
pub fn run(elf_file: &str, policy: CPUPolicy) -> SimulatorResult<RunStats> {
    // Load the ELF file
    let (elf_reader, elf_data_origin) = elf_helper::parse_elf_file(elf_file)?;
    let elf_data = &elf_data_origin;

    let mut cpu = CPUState::make(policy);

    let mut mem = InclusiveCache::default();
    // let mut mem = ExclusiveCache::default();
    {
        // Borrow the MMU for initialization
        let mmu = &mut mem.mmu;
        // Set stack
        loader::set_stack(&mut cpu, mmu, STACK_BASE, STACK_SIZE)?;
        // Load ELF data into memory
        loader::load_elf(&mut cpu, mmu, &elf_reader, elf_data)?;
    }

    // Run the CPU
    let _ = match policy.implementation {
        Implementation::SingleCycle => single_cycle::run(&mut cpu, &mut mem)?,
        Implementation::Pipelined => pipelined::run(&mut cpu, &mut mem)?,
    };

    // mem.verify_exclusiveness();
    mem.verify_inclusiveness()?;

    let cycle_count_base = cpu.history.cycle_count;
    let cycle_count = cycle_count_base + cpu.history.mem_stall_count;
    let cycle_count_worst =
        cycle_count_base + cpu.history.mem_stall_worst_count;
    let instruction_count = cpu.history.inst_count;
    let cpi = cycle_count as f64 / instruction_count as f64;
    let cpi_worst = cycle_count_worst as f64 / instruction_count as f64;
    let cpi_ideal = cycle_count_base as f64 / instruction_count as f64;

    if policy.history {
        eprintln!("[HISTORY] # instructions = {}", instruction_count);
        eprintln!(
            "[HISTORY] CPI = {:.2}, CPI (no caching) = {:.2}, CPI (ideal) = {:.2}",
            cpi, cpi_worst, cpi_ideal
        );
        eprintln!("[HISTORY] {:?}", mem.get_history());
        eprintln!("[HISTORY] AMAT = {:.2}", mem.get_amat());
    }

    Ok((cpi_ideal, cpi, cpi_worst, cpi_worst / cpi))
}

/// Fetch operations from the trace file
pub fn fetch_operations(trace_path: &str) -> SimulatorResult<Vec<(char, u32)>> {
    let content = std::fs::read_to_string(trace_path)?;
    let mut operations: Vec<(char, u32)> = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse the line into op and address
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(ElfError::ParseError(
                trace_path.into(),
                format!(
                    "Invalid format at line {}: expected 'op address'",
                    line_num + 1
                ),
            )
            .into());
        }

        let op = parts[0].chars().next().ok_or_else(|| {
            ElfError::ParseError(
                trace_path.into(),
                format!("Invalid operation at line {}", line_num + 1),
            )
        })?;

        if op != 'r' && op != 'w' {
            return Err(ElfError::ParseError(
                trace_path.into(),
                format!(
                    "Invalid operation '{}' at line {}: expected 'r' or 'w'",
                    op,
                    line_num + 1
                ),
            )
            .into());
        }

        let address_str = parts[1];
        if !address_str.starts_with("0x") {
            return Err(ElfError::ParseError(
                trace_path.into(),
                format!("Invalid address format at line {}: expected hexadecimal starting with '0x'", line_num + 1)
            ).into());
        }

        let address =
            u32::from_str_radix(&address_str[2..], 16).map_err(|_| {
                ElfError::ParseError(
                    trace_path.into(),
                    format!(
                        "Invalid hexadecimal address at line {}",
                        line_num + 1
                    ),
                )
            })?;

        operations.push((op, address));
    }

    Ok(operations)
}

/// Run simulation on the given trace file
pub fn run_trace(
    cache: &mut impl StorageInterface,
    trace_path: &str,
) -> SimulatorResult<f64> {
    let operations = fetch_operations(trace_path)?;

    {
        // Borrow the MMU for initialization
        let mmu = &mut cache.mmu();
        // Allocate pages beforehand
        for (_, address) in &operations {
            mmu.allocate_page(*address);
        }
    }

    // Simulate the trace
    for (op, address) in &operations {
        let mut dummy: Option<i32> = Some(0);
        match op {
            'r' => {
                cache.get8(*address, &mut dummy)?;
            }
            'w' => {
                cache.set8(*address, 0, &mut dummy)?;
            }
            _ => {
                // This should never happen due to validation in
                // fetch_operations
                return Err(ElfError::ParseError(
                    trace_path.into(),
                    format!("Unexpected operation: {}", op),
                )
                .into());
            }
        }
    }

    // Return the predicted AMAT
    Ok(cache.get_amat())
}
