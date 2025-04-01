//! Instruction representation

use crate::alu::ALUOp;
use crate::alu::ALUSrc;
use crate::error::SimulatorResult;

pub mod decode_helper;

/// NOP: ADDI x0, x0, 0
pub(crate) const NOP: u32 = 0x13;

/// Wrapped instruction
#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    /// Raw representation
    pub raw_inst: u32,
    /// Opcode
    pub opcode: Opcode,
    /// Format
    pub format: Format,
    /// Function
    pub function: Function,
    /// Subfields
    pub attributes: Attributes,
    /// Control signals
    pub controls: Controls,
}

impl Instruction {
    pub fn new(raw_inst: u32) -> SimulatorResult<Self> {
        let opcode = decode_helper::raw_to_opcode(raw_inst)?;
        let format = decode_helper::opcode_to_format(opcode);
        let attributes = Attributes::default();
        let function = Function::default();
        let controls = Controls::default();

        let mut inst =
            Self { raw_inst, opcode, format, function, attributes, controls };

        decode_helper::parse(&mut inst)?;
        Ok(inst)
    }
}

impl Default for Instruction {
    fn default() -> Self {
        Self::new(NOP).unwrap()
    }
}

/// rv32i opcode
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Opcode {
    Lui,
    AuiPc,
    Jal,
    Jalr,
    Branch,
    Load,
    Store,
    Op,
    OpImm,
    System,
}

/// rv32i instruction format
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Format {
    R,
    I,
    S,
    B,
    U,
    J,
    Sys,
}

/// rv32i function (instruction?)
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Function {
    LUI,
    AUIPC,
    JAL,
    JALR,
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
    LB,
    LH,
    LW,
    LBU,
    LHU,
    SB,
    SH,
    SW,
    #[default]
    ADDI,
    SLTI,
    SLTIU,
    XORI,
    ORI,
    ANDI,
    SLLI,
    SRLI,
    SRAI,
    ADD,
    SUB,
    SLL,
    SLT,
    SLTU,
    XOR,
    SRL,
    SRA,
    OR,
    AND,
    ECALL,
}

/// Instruction attributes
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Attributes {
    // Take all you need
    pub opcode: Option<u32>,
    pub rs1: Option<u32>,
    pub rs2: Option<u32>,
    pub rd: Option<u32>,
    pub funct3: Option<u32>,
    pub funct7: Option<u32>,
    pub imm: Option<u32>,
}

/// Control signals
#[derive(Clone, Copy, Debug, Default)]
pub struct Controls {
    pub branch: bool,
    pub mem_read: bool,
    pub mem_write: bool,
    pub reg_write: bool,
    pub mem_step: u32,
    pub alu_op: ALUOp,
    pub alu_src: ALUSrc,
}
