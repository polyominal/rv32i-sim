//! Primitive implementation of 5 stages

use crate::alu::{alu, ALUSrc};
use crate::cpu::CPUState;
use crate::instruction::Instruction;
use crate::memory::StorageInterface;
use crate::system_call::syscall;

/// IF: Fetch the instruction from memory
pub fn instruction_fetch(
    pc: u32,
    cpu: &mut CPUState,
    mem: &mut impl StorageInterface,
) -> u32 {
    let mut stall_count = Some(0);
    let mut stall_count_worst = Some(0);
    let raw_inst = mem.get(pc, 4, &mut stall_count, &mut stall_count_worst);
    cpu.history.mem_stall_count += stall_count.unwrap();
    cpu.history.mem_stall_worst_count += stall_count_worst.unwrap();
    assert!(raw_inst != 0, "Instruction fetch failed");
    raw_inst
}

/// ID: Instruction decode
pub fn instruction_decode(raw_inst: u32) -> Instruction {
    Instruction::new(raw_inst)
}

/// ID: Register read
pub fn register_read(inst: &Instruction, cpu: &CPUState) -> (i32, i32) {
    let rs1 = cpu.gpr[inst.attributes.rs1.unwrap_or(0) as usize].read() as i32;
    let rs2 = cpu.gpr[inst.attributes.rs2.unwrap_or(0) as usize].read() as i32;
    (rs1, rs2)
}

/// EX: Compute stuff
pub fn execute(
    cpu: &mut CPUState,
    mem: &mut impl StorageInterface,
    inst: &Instruction,
    op1: i32,
    op2: i32,
) -> i32 {
    // Increment instruction count
    cpu.update_inst_count(1);

    use crate::instruction::Opcode;
    if inst.opcode == Opcode::System {
        // Handle system calls
        syscall(op1, op2, mem)
    } else {
        // Handle ALU operations
        use ALUSrc::*;
        let op2 = match inst.controls.alu_src {
            REG => op2,
            IMM => inst.attributes.imm.unwrap() as i32,
        };
        if cpu.policy.verbose {
            // Print the instruction
            eprintln!("[VERBOSE] Executing: {:?}", inst);
            // Print the operands
            eprintln!("[VERBOSE] op1: {:#010x}; op2: {:#010x}", op1, op2);
        }
        alu(&inst, op1, op2)
    }
}

/// MEM: Access memory
pub fn memory_access(
    pc: u32,
    inst: &Instruction,
    cpu: &mut CPUState,
    mem: &mut impl StorageInterface,
    exec_result: i32,
    op2: i32,
) -> u32 {
    let mut mem_result: u32 = 0;

    let address = exec_result as u32;
    let mem_step = inst.controls.mem_step;

    let mut stall_count = Some(0);
    let mut stall_count_worst = Some(0);

    if inst.controls.mem_read {
        mem_result =
            mem.get(address, mem_step, &mut stall_count, &mut stall_count_worst);
    } else if inst.controls.mem_write {
        mem.set(
            address,
            mem_step,
            op2 as u32,
            &mut stall_count,
            &mut stall_count_worst,
        );
    }

    cpu.history.mem_stall_count += stall_count.unwrap();
    cpu.history.mem_stall_worst_count += stall_count_worst.unwrap();

    match inst.controls.mem_read {
        true => {
            // Write the memory result
            mem_result
        }
        false => {
            // Write the execution result
            // Special cases: LUI, AUIPC, JAL, JALR
            use crate::instruction::Function;
            let imm = inst.attributes.imm.unwrap_or(0) as i32;
            match inst.function {
                Function::LUI => imm as u32,
                Function::AUIPC => ((pc as i32) + imm) as u32,
                Function::JAL | Function::JALR => pc + 4,
                _ => exec_result as u32,
            }
        }
    }
}

/// WB: Write stuff back to the selected register
pub fn write_back(_: u32, inst: &Instruction, cpu: &mut CPUState, wb_result: u32) {
    // If you need to write
    if inst.controls.reg_write {
        let rd = inst.attributes.rd.unwrap() as usize;
        // You don't write to x0
        if rd != 0 {
            cpu.gpr[rd].write(wb_result);
        }
    }
}
