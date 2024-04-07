//! ALU implementation

use crate::instruction::Instruction;

/// Performs an atomic ALU operation
/// Do signed arithmetic for good
pub fn alu(inst: &Instruction, op1: i32, op2: i32) -> i32 {
    use self::ALUOp::*;

    match inst.controls.alu_op {
        ADD => op1.wrapping_add(op2),
        SUB => op1.wrapping_sub(op2),
        AND => op1 & op2,
        OR => op1 | op2,
        XOR => op1 ^ op2,
        BEQ => (op1 != op2) as i32,
        BNE => (op1 == op2) as i32,
        BLT => !(op1 < op2) as i32,
        BLTU => !((op1 as u32) < (op2 as u32)) as i32,
        BGE => !(op1 >= op2) as i32,
        BGEU => !((op1 as u32) >= (op2 as u32)) as i32,
        SLL => op1 << op2,
        SRL => ((op1 as u32) >> op2) as i32,
        SRA => op1.wrapping_shr(op2 as u32),
        SLT => (op1 < op2) as i32,
        SLTU => ((op1 as u32) < (op2 as u32)) as i32,
    }
}

/// Selector for ALU src2 input
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ALUSrc {
    // From register
    REG,
    // From immediate
    IMM,
}

impl Default for ALUSrc {
    fn default() -> Self {
        ALUSrc::REG
    }
}

/// Set of ALU operations needed for rv32i
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ALUOp {
    // Arithmetic
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

impl Default for ALUOp {
    fn default() -> Self {
        ALUOp::ADD
    }
}
