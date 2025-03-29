//! ALU implementation

use crate::instruction::Instruction;

/// Performs an atomic ALU operation
/// Do signed arithmetic for good
pub fn alu(inst: &Instruction, op1: i32, op2: i32) -> i32 {
    match inst.controls.alu_op {
        ALUOp::ADD => op1.wrapping_add(op2),
        ALUOp::SUB => op1.wrapping_sub(op2),
        ALUOp::AND => op1 & op2,
        ALUOp::OR => op1 | op2,
        ALUOp::XOR => op1 ^ op2,
        ALUOp::BEQ => (op1 != op2) as i32,
        ALUOp::BNE => (op1 == op2) as i32,
        ALUOp::BLT => (op1 >= op2) as i32,
        ALUOp::BLTU => ((op1 as u32) >= (op2 as u32)) as i32,
        ALUOp::BGE => (op1 < op2) as i32,
        ALUOp::BGEU => ((op1 as u32) < (op2 as u32)) as i32,
        ALUOp::SLL => op1 << op2,
        ALUOp::SRL => ((op1 as u32) >> op2) as i32,
        ALUOp::SRA => op1.wrapping_shr(op2 as u32),
        ALUOp::SLT => (op1 < op2) as i32,
        ALUOp::SLTU => ((op1 as u32) < (op2 as u32)) as i32,
    }
}

/// Selector for ALU src2 input
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ALUSrc {
    // From register
    #[default]
    REG,
    // From immediate
    IMM,
}

/// Set of ALU operations needed for rv32i
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ALUOp {
    // Arithmetic
    #[default]
    ADD,
    SUB,
    // Logical
    AND,
    OR,
    XOR,
    // Set
    SLT,
    SLTU,
    // Shift
    SLL,
    SRL,
    SRA,
    // Branch
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
}
