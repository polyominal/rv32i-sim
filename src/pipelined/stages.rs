//! 5 stages adapted for piplined execution

use super::pipeline::PipelineState;
use crate::cpu::CPUState;
use crate::instruction::Opcode;
use crate::memory::StorageInterface;
use crate::stages_simple;

/// IF stage
pub fn instruction_fetch(
    cpu: &mut CPUState,
    mem: &mut impl StorageInterface,
    next_state: &mut PipelineState,
) {
    // Increment PC by 4
    let pc = cpu.pc.read();
    let new_pc = pc + 4;
    cpu.pc.write(new_pc);

    // Fetch the raw instruction
    let raw_inst = stages_simple::instruction_fetch(pc, cpu, mem);

    if cpu.policy.verbose {
        // Print the PC and the raw instruction
        eprintln!("PC: {:#010x}; Instruction: {:#032b}", pc, raw_inst);
    }

    // Update IF/ID register
    next_state.if_id.pc = pc;
    next_state.if_id.raw_inst = raw_inst;
}

/// ID stage
pub fn instruction_decode(
    cpu: &CPUState,
    current_state: &PipelineState,
    next_state: &mut PipelineState,
) {
    // Fetch the raw instruction
    let raw_inst = current_state.if_id.raw_inst;

    // Decode the instruction
    let inst = stages_simple::instruction_decode(raw_inst);

    // WB hazard -> Data in the register
    let op1: i32;
    if current_state.wb_hazard_op1(&inst) {
        op1 = current_state.mem_wb.wb_result as i32;
    } else {
        op1 = cpu.gpr[inst.attributes.rs1.unwrap_or(0) as usize].read() as i32;
    }

    let op2: i32;
    if current_state.wb_hazard_op2(&inst) {
        op2 = current_state.mem_wb.wb_result as i32;
    } else {
        op2 = cpu.gpr[inst.attributes.rs2.unwrap_or(0) as usize].read() as i32;
    }

    let pc = current_state.if_id.pc;
    next_state.id_ex.pc = pc;
    next_state.id_ex.inst = inst;
    next_state.id_ex.op1 = op1;
    next_state.id_ex.op2 = op2;

    if inst.opcode == Opcode::Branch {
        // Also precompute the branch target when needed
        let imm = inst.attributes.imm.unwrap() as i32;
        next_state.id_ex.taken_pc = Some(((pc as i32) + imm) as u32);
    } else {
        // No precomputed branch target
        next_state.id_ex.taken_pc = None;
    }
}

/// EX stage
pub fn execute(
    cpu: &mut CPUState,
    mem: &mut impl StorageInterface,
    current_state: &PipelineState,
    next_state: &mut PipelineState,
) {
    let pc = current_state.id_ex.pc;
    let inst = current_state.id_ex.inst;

    let op1: i32;
    // EX hazard -> MEM hazard -> Data in the register
    if current_state.ex_hazard_op1() {
        op1 = current_state.ex_mem.exec_result;
    } else if current_state.mem_hazard_op1() {
        op1 = current_state.mem_wb.wb_result as i32;
    } else {
        op1 = current_state.id_ex.op1;
    }

    let op2: i32;
    // EX hazard -> MEM hazard -> Data in the register
    if current_state.ex_hazard_op2() {
        op2 = current_state.ex_mem.exec_result;
    } else if current_state.mem_hazard_op2() {
        op2 = current_state.mem_wb.wb_result as i32;
    } else {
        op2 = current_state.id_ex.op2;
    }

    let exec_result = stages_simple::execute(cpu, mem, &inst, op1, op2);

    next_state.ex_mem.pc = pc;
    next_state.ex_mem.inst = inst;
    next_state.ex_mem.exec_result = exec_result;
    next_state.ex_mem.op2 = op2;
    next_state.ex_mem.taken_pc = current_state.id_ex.taken_pc;
}

/// MEM stage
pub fn memory_access(
    cpu: &mut CPUState,
    mem: &mut impl StorageInterface,
    current_state: &PipelineState,
    next_state: &mut PipelineState,
) {
    let pc = current_state.ex_mem.pc;
    let inst = current_state.ex_mem.inst;
    let exec_result = current_state.ex_mem.exec_result;
    let op2 = current_state.ex_mem.op2;

    next_state.mem_wb.pc = pc;
    next_state.mem_wb.inst = inst;
    next_state.mem_wb.wb_result =
        stages_simple::memory_access(pc, &inst, cpu, mem, exec_result, op2);
}

/// WB stage
pub fn write_back(cpu: &mut CPUState, current_state: &PipelineState) {
    let pc = current_state.mem_wb.pc;
    let inst = current_state.mem_wb.inst;
    let wb_result = current_state.mem_wb.wb_result;

    stages_simple::write_back(pc, &inst, cpu, wb_result);
}
