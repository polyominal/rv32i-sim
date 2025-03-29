//! Pipelined implementation

use core::panic;

use crate::cpu::CPUState;
use crate::instruction::Instruction;
use crate::instruction::Opcode;
use crate::instruction::NOP;
use crate::memory::StorageInterface;
use crate::pipelined::pipeline::PipelineState;

pub mod branch_predictor;
pub mod pipeline;
pub mod stages;

/// Returns the exiting PC address
pub fn run(cpu: &mut CPUState, mem: &mut impl StorageInterface) -> u32 {
    let mut current_state = PipelineState::default();
    let mut next_state = PipelineState::default();

    let mut branch_predictor =
        branch_predictor::BranchPredictor::new(cpu.policy.heuristic);
    let mut predicted_branch_taken: bool = false;

    loop {
        // Check for stack overflow
        if cpu.stack_overflow() {
            panic!("Stack overflow");
        }

        // Print the initial PC of this cycle
        if cpu.policy.verbose {
            eprintln!("[VERBOSE] New cycle; PC: {:#010x}", cpu.pc.read());
        }

        // Increment CPU cycle count
        cpu.update_cycle_count(1);

        if current_state.load_hazard() {
            // Must insert a NOP
            next_state.id_ex.inst = Instruction::default();
            cpu.update_inst_count(-1);
            if cpu.policy.verbose {
                eprintln!("[VERBOSE] Inserting NOP due to load hazard");
            }
        } else {
            stages::instruction_fetch(cpu, mem, &mut next_state);
            stages::instruction_decode(cpu, &current_state, &mut next_state);
        }

        stages::execute(cpu, mem, &current_state, &mut next_state);
        stages::memory_access(cpu, mem, &current_state, &mut next_state);
        stages::write_back(cpu, &current_state);

        let exec_inst = next_state.ex_mem.inst;
        if exec_inst.opcode == Opcode::System && next_state.ex_mem.op2 == 3 {
            return next_state.ex_mem.pc;
        }

        let exec_result = next_state.ex_mem.exec_result;
        if exec_inst.controls.branch {
            // Do branch, conditional or unconditional
            let imm = exec_inst.attributes.imm.unwrap() as i32;
            let exec_pc = next_state.ex_mem.pc;
            let actual_new_pc: u32;
            let branch_taken: bool;
            if !(exec_inst.opcode == Opcode::Branch && exec_result != 0) {
                // Branch taken
                branch_taken = true;
                actual_new_pc = match exec_inst.opcode {
                    Opcode::Jalr => (exec_result as u32) & !1u32,
                    Opcode::Branch => next_state.ex_mem.taken_pc.unwrap(),
                    _ => ((exec_pc as i32) + imm) as u32,
                };
            } else {
                // Branch not taken
                branch_taken = false;
                actual_new_pc = exec_pc + 4;
            }

            let mut do_jump: bool = true;
            if exec_inst.opcode == Opcode::Branch {
                // Update the branch predictor
                branch_predictor.update(exec_pc, branch_taken);
                if branch_taken == predicted_branch_taken {
                    do_jump = false;
                }
            }

            if do_jump {
                if cpu.policy.verbose {
                    eprintln!(
                        "[VERBOSE] Jumping from {:#010x} to {:#010x}",
                        cpu.pc.read(),
                        actual_new_pc
                    );
                }

                cpu.pc.write(actual_new_pc);
                // Flush
                next_state.if_id.raw_inst = NOP;
                next_state.id_ex.inst = Instruction::default();
                // We're replacing 2 instructions with NOP
                cpu.update_inst_count(-2);
            }
        }

        predicted_branch_taken = false;
        // Try branch prediction
        let id_inst = next_state.id_ex.inst;
        if id_inst.opcode == Opcode::Branch {
            match branch_predictor.predict(next_state.id_ex.pc) {
                true => {
                    // Predicted taken; let's do this
                    // Jump to taken_pc
                    cpu.pc.write(next_state.id_ex.taken_pc.unwrap());
                    // Flush
                    next_state.if_id.raw_inst = NOP;
                    // We're dumping 1 instruction
                    cpu.update_inst_count(-1);
                    // Set the taken flag
                    predicted_branch_taken = true;
                }
                false => {
                    // Do nothing
                }
            }
        }

        // Advance the pipeline state
        current_state = next_state;
    }
}
