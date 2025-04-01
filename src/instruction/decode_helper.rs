//! Decoding helper functions.
//! Many drawn from <https://github.com/djanderson/riscv-5stage-simulator/blob/master/src/instruction/decoder.rs>

use super::Attributes;
use super::Controls;
use super::Format;
use super::Function;
use super::Instruction;
use super::Opcode;
use crate::error::SimulatorError;
use crate::error::SimulatorResult;

/// Extracts the sign-extended immediate from an instruction
fn get_imm_sign_extended(inst: &Instruction) -> Option<u32> {
    let shamt = match inst.opcode {
        Opcode::Lui | Opcode::AuiPc => 0,
        Opcode::Jal => 12,
        Opcode::Branch => 19,
        _ => 20,
    };

    inst.attributes.imm.map(|v| (((v as i32) << shamt) >> shamt) as u32)
}

/// Determines an instruction's mnemonic, e.g., JAL, XOR, or SRA
fn get_function(inst: &Instruction) -> SimulatorResult<Function> {
    use Function::*;
    use Opcode::*;
    // Opcode-determined ones
    let function = match inst.opcode {
        Lui => LUI,
        AuiPc => AUIPC,
        Jal => JAL,
        Jalr => JALR,
        System => ECALL,
        _ => Function::default(),
    };
    if function != Function::default() {
        return Ok(function);
    }

    let funct3 = inst
        .attributes
        .funct3
        .ok_or(SimulatorError::InvalidInstructionError(inst.raw_inst, 0))?;

    let funct7_bit = (inst.raw_inst & 0x40000000) >> 30;

    Ok(match (inst.opcode, funct3, funct7_bit) {
        (Branch, 0b000, _) => BEQ,
        (Branch, 0b001, _) => BNE,
        (Branch, 0b100, _) => BLT,
        (Branch, 0b101, _) => BGE,
        (Branch, 0b110, _) => BLTU,
        (Branch, 0b111, _) => BGEU,
        (Load, 0b000, _) => LB,
        (Load, 0b001, _) => LH,
        (Load, 0b010, _) => LW,
        (Load, 0b100, _) => LBU,
        (Load, 0b101, _) => LHU,
        (Store, 0b000, _) => SB,
        (Store, 0b001, _) => SH,
        (Store, 0b010, _) => SW,
        (OpImm, 0b000, _) => ADDI,
        (OpImm, 0b010, _) => SLTI,
        (OpImm, 0b011, _) => SLTIU,
        (OpImm, 0b100, _) => XORI,
        (OpImm, 0b110, _) => ORI,
        (OpImm, 0b111, _) => ANDI,
        (OpImm, 0b001, _) => SLLI,
        (OpImm, 0b101, 0b0) => SRLI,
        (OpImm, 0b101, 0b1) => SRAI,
        (Op, 0b000, 0b0) => ADD,
        (Op, 0b000, 0b1) => SUB,
        (Op, 0b001, _) => SLL,
        (Op, 0b010, _) => SLT,
        (Op, 0b011, _) => SLTU,
        (Op, 0b100, _) => XOR,
        (Op, 0b101, 0b0) => SRL,
        (Op, 0b101, 0b1) => SRA,
        (Op, 0b110, _) => OR,
        (Op, 0b111, _) => AND,
        _ => {
            return Err(SimulatorError::InvalidInstructionError(
                inst.raw_inst,
                0,
            ))
        }
    })
}

pub fn get_controls(inst: &Instruction) -> Controls {
    use Function::*;
    use Opcode::*;

    use crate::alu::ALUOp;
    use crate::alu::ALUSrc;

    Controls {
        branch: matches!(inst.opcode, Branch | Jal | Jalr),
        mem_read: matches!(inst.opcode, Opcode::Load),
        mem_write: matches!(inst.opcode, Opcode::Store),
        reg_write: !matches!(inst.opcode, Branch | Store),
        mem_step: match inst.function {
            LB | LBU | SB => 1,
            LH | LHU | SH => 2,
            LW | SW => 4,
            _ => 0,
        },
        alu_op: match inst.function {
            LUI | AUIPC => ALUOp::ADD,
            JAL => ALUOp::BEQ,
            JALR => ALUOp::ADD,
            BEQ => ALUOp::BEQ,
            BNE => ALUOp::BNE,
            BLT => ALUOp::BLT,
            BGE => ALUOp::BGE,
            BLTU => ALUOp::BLTU,
            BGEU => ALUOp::BGEU,
            LB | LH | LW | LBU | LHU | SB | SH | SW => ALUOp::ADD,
            ADDI => ALUOp::ADD,
            SLTI => ALUOp::SLT,
            SLTIU => ALUOp::SLTU,
            XORI => ALUOp::XOR,
            ORI => ALUOp::OR,
            ANDI => ALUOp::AND,
            SLLI => ALUOp::SLL,
            SRLI => ALUOp::SRL,
            SRAI => ALUOp::SRA,
            ADD => ALUOp::ADD,
            SUB => ALUOp::SUB,
            SLL => ALUOp::SLL,
            SLT => ALUOp::SLT,
            SLTU => ALUOp::SLTU,
            XOR => ALUOp::XOR,
            SRL => ALUOp::SRL,
            SRA => ALUOp::SRA,
            OR => ALUOp::OR,
            AND => ALUOp::AND,
            ECALL => ALUOp::default(),
        },
        alu_src: match inst.opcode {
            Branch | Op | Jal => ALUSrc::REG,
            _ => ALUSrc::IMM,
        },
    }
}

/// Returns the opcode from a raw instruction
pub fn raw_to_opcode(raw_inst: u32) -> SimulatorResult<Opcode> {
    let opcode = raw_inst & 0x7f_u32;
    match opcode {
        0x37 => Ok(Opcode::Lui),
        0x17 => Ok(Opcode::AuiPc),
        0x6f => Ok(Opcode::Jal),
        0x67 => Ok(Opcode::Jalr),
        0x63 => Ok(Opcode::Branch),
        0x03 => Ok(Opcode::Load),
        0x23 => Ok(Opcode::Store),
        0x33 => Ok(Opcode::Op),
        0x13 => Ok(Opcode::OpImm),
        0x73 => Ok(Opcode::System),
        _ => Err(SimulatorError::InvalidInstructionError(raw_inst, 0)),
    }
}

/// Returns the instruction format from an opcode
pub fn opcode_to_format(opcode: Opcode) -> Format {
    match opcode {
        Opcode::Lui => Format::U,
        Opcode::AuiPc => Format::U,
        Opcode::Jal => Format::J,
        Opcode::Jalr => Format::I,
        Opcode::Branch => Format::B,
        Opcode::Load => Format::I,
        Opcode::Store => Format::S,
        Opcode::Op => Format::R,
        Opcode::OpImm => Format::I,
        Opcode::System => Format::Sys,
    }
}

/// Parses other stuff
pub fn parse(inst: &mut Instruction) -> SimulatorResult<()> {
    inst.attributes = match inst.format {
        Format::R => parse_format_r(inst.raw_inst),
        Format::I => parse_format_i(inst.raw_inst),
        Format::S => parse_format_s(inst.raw_inst),
        Format::B => parse_format_b(inst.raw_inst),
        Format::U => parse_format_u(inst.raw_inst),
        Format::J => parse_format_j(inst.raw_inst),
        Format::Sys => parse_format_sys(inst.raw_inst),
    };
    inst.attributes.imm = get_imm_sign_extended(inst);
    inst.function = get_function(inst)?;
    inst.controls = get_controls(inst);

    Ok(())
}

/// Parses attributes for an R-type instruction
fn parse_format_r(raw_inst: u32) -> Attributes {
    Attributes {
        opcode: Some(get_opcode(raw_inst)),
        rs1: Some(get_rs1(raw_inst)),
        rs2: Some(get_rs2(raw_inst)),
        rd: Some(get_rd(raw_inst)),
        funct3: Some(get_funct3(raw_inst)),
        funct7: Some(get_funct7(raw_inst)),
        imm: None,
    }
}

/// Parses attributes for an I-type instruction
fn parse_format_i(raw_inst: u32) -> Attributes {
    fn is_i_star(attributes: &Attributes) -> bool {
        attributes.opcode == Some(0x13)
            && (attributes.funct3 == Some(0b001)
                || attributes.funct3 == Some(0b101))
    }

    let mut attributes = Attributes {
        opcode: Some(get_opcode(raw_inst)),
        rs1: Some(get_rs1(raw_inst)),
        rs2: None,
        rd: Some(get_rd(raw_inst)),
        funct3: Some(get_funct3(raw_inst)),
        funct7: None,
        imm: None, // TBD
    };
    if !is_i_star(&attributes) {
        // I
        attributes.imm = Some((raw_inst & 0xfff00000) >> 20)
    } else {
        // I*
        // It happens to be the same as rs2
        attributes.imm = Some(get_rs2(raw_inst));
    }
    attributes
}

/// Parses attributes for an S-type instruction
fn parse_format_s(raw_inst: u32) -> Attributes {
    Attributes {
        opcode: Some(get_opcode(raw_inst)),
        rs1: Some(get_rs1(raw_inst)),
        rs2: Some(get_rs2(raw_inst)),
        rd: None,
        funct3: Some(get_funct3(raw_inst)),
        funct7: None,
        imm: Some(((raw_inst & 0xfe000000) >> 20) | ((raw_inst & 0xf80) >> 7)),
    }
}

/// Parses attributes for a B-type instruction
fn parse_format_b(raw_inst: u32) -> Attributes {
    Attributes {
        opcode: Some(get_opcode(raw_inst)),
        rs1: Some(get_rs1(raw_inst)),
        rs2: Some(get_rs2(raw_inst)),
        rd: None,
        funct3: Some(get_funct3(raw_inst)),
        funct7: None,
        imm: Some(
            ((raw_inst & 0x80000000) >> 19)
                | ((raw_inst & 0x80) << 4)
                | ((raw_inst & 0x7e000000) >> 20)
                | ((raw_inst & 0xf00) >> 7),
        ),
    }
}

/// Parses attributes for a U-type instruction
fn parse_format_u(raw_inst: u32) -> Attributes {
    Attributes {
        opcode: Some(get_opcode(raw_inst)),
        rs1: None,
        rs2: None,
        rd: Some(get_rd(raw_inst)),
        funct3: None,
        funct7: None,
        imm: Some(raw_inst & 0xfffff000),
    }
}

/// Parses attributes for a J-type instruction
fn parse_format_j(raw_inst: u32) -> Attributes {
    Attributes {
        opcode: Some(get_opcode(raw_inst)),
        rs1: None,
        rs2: None,
        rd: Some(get_rd(raw_inst)),
        funct3: None,
        funct7: None,
        imm: Some(
            ((raw_inst & 0x80000000) >> 11)
                | (raw_inst & 0xff000)
                | ((raw_inst & 0x100000) >> 9)
                | ((raw_inst & 0x7fe00000) >> 20),
        ),
    }
}

/// Parses attributes for a Sys-type instruction
fn parse_format_sys(raw_inst: u32) -> Attributes {
    // a0, a7
    Attributes {
        opcode: Some(get_opcode(raw_inst)),
        rs1: Some(10),
        rs2: Some(17),
        rd: Some(10),
        funct3: None,
        funct7: None,
        imm: None,
    }
}

/// Extracts opcode from a raw instruction
fn get_opcode(raw_inst: u32) -> u32 {
    raw_inst & 0x7f
}

/// Extracts funct3 from a raw instruction
fn get_funct3(raw_inst: u32) -> u32 {
    (raw_inst >> 12) & 0x7
}

/// Extracts the rs1 field from a raw instruction
fn get_rs1(raw_inst: u32) -> u32 {
    (raw_inst >> 15) & 0x1f
}

/// Extracts the rs2 field from a raw instruction
fn get_rs2(raw_inst: u32) -> u32 {
    (raw_inst >> 20) & 0x1f
}

/// Extracts the rd field from a raw instruction
fn get_rd(raw_inst: u32) -> u32 {
    (raw_inst >> 7) & 0x1f
}

/// Extracts the funct7 field from a raw instruction
fn get_funct7(raw_inst: u32) -> u32 {
    (raw_inst >> 25) & 0x7f
}
