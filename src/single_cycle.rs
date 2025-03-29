//! Single cycle implementation

use crate::cpu::CPUState;
use crate::instruction::Opcode;
use crate::memory::StorageInterface;
use crate::stages_simple::*;

/// Returns the exiting PC address
pub fn run(cpu: &mut CPUState, mem: &mut impl StorageInterface) -> u32 {
    loop {
        // Detect stack overflow
        if cpu.stack_overflow() {
            panic!("Stack overflow");
        }

        // Increment CPU cycle count
        cpu.update_cycle_count(1);

        // Read and increment PC
        let pc = cpu.pc.read();
        cpu.pc.write(pc + 4);

        if cpu.policy.verbose {
            eprintln!("[VERBOSE] PC: {:#010x}", pc);
        }

        // IF
        let raw_inst = instruction_fetch(pc, cpu, mem);
        // ID
        let inst = instruction_decode(raw_inst);
        let (rs1, rs2) = register_read(&inst, cpu);
        // EX
        let exec_result = execute(cpu, mem, &inst, rs1, rs2);
        // MEM
        let wb_result = memory_access(pc, &inst, cpu, mem, exec_result, rs2);
        // WB
        write_back(pc, &inst, cpu, wb_result);

        // System call: exit
        if inst.opcode == Opcode::System && rs2 == 3 {
            return pc;
        }

        // Update PC on branch
        if inst.controls.branch
            && !(inst.opcode == Opcode::Branch && exec_result != 0)
        {
            let imm = inst.attributes.imm.unwrap() as i32;
            let new_pc = match inst.opcode {
                Opcode::Jalr => (exec_result as u32) & !1u32,
                _ => ((pc as i32) + imm) as u32,
            };
            if cpu.policy.verbose {
                // Print the opcode that caused this branch
                eprintln!(
                    "[VERBOSE] Branching from {:#010x} to: {:#010x}",
                    pc, new_pc
                );
            }
            cpu.pc.write(new_pc);
        }
    }
}
