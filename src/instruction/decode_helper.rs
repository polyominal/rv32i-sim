//! Decoding helper functions.
//! Many drawn from <https://github.com/djanderson/riscv-5stage-simulator/blob/master/src/instruction/decoder.rs>

use super::{Attributes, Controls, Format, Function, Instruction, Opcode};

/// Extracts the sign-extended immediate from an instruction
fn get_imm_sign_extended(inst: &Instruction) -> Option<u32> {
    let shamt = match inst.opcode {
        Opcode::Lui | Opcode::AuiPc => 0,
        Opcode::Jal => 12,
        Opcode::Branch => 19,
        _ => 20,
    };

    match inst.attributes.imm {
        Some(v) => Some((((v as i32) << shamt) >> shamt) as u32),
        None => None,
    }
}

/// Determines an instruction's mnemonic, e.g., JAL, XOR, or SRA
fn get_function(inst: &Instruction) -> Function {
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
        return function;
    }

    match (
        inst.opcode,
        inst.attributes.funct3.unwrap(),
        (inst.raw_inst & 0x40000000) >> 30,
    ) {
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
        _ => panic!("Failed to decode instruction {:#0x}", inst.raw_inst),
    }
}

pub fn get_controls(inst: &Instruction) -> Controls {
    use crate::alu::{ALUOp, ALUSrc};
    use Function::*;
    use Opcode::*;

    Controls {
        branch: match inst.opcode {
            Branch | Jal | Jalr => true,
            _ => false,
        },
        mem_read: match inst.opcode {
            Opcode::Load => true,
            _ => false,
        },
        mem_write: match inst.opcode {
            Opcode::Store => true,
            _ => false,
        },
        reg_write: match inst.opcode {
            Branch | Store => false,
            _ => true,
        },
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
pub fn raw_to_opcode(raw_inst: u32) -> Opcode {
    let opcode = raw_inst & (0x7f as u32);
    match opcode {
        0x37 => Opcode::Lui,
        0x17 => Opcode::AuiPc,
        0x6f => Opcode::Jal,
        0x67 => Opcode::Jalr,
        0x63 => Opcode::Branch,
        0x03 => Opcode::Load,
        0x23 => Opcode::Store,
        0x33 => Opcode::Op,
        0x13 => Opcode::OpImm,
        0x73 => Opcode::System,
        _ => panic!("Unknown opcode: {:07b}", opcode),
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
pub fn parse(inst: &mut Instruction) {
    inst.attributes = match inst.format {
        Format::R => parse_format_r(inst.raw_inst),
        Format::I => parse_format_i(inst.raw_inst),
        Format::S => parse_format_s(inst.raw_inst),
        Format::B => parse_format_b(inst.raw_inst),
        Format::U => parse_format_u(inst.raw_inst),
        Format::J => parse_format_j(inst.raw_inst),
        Format::Sys => parse_format_sys(inst.raw_inst),
    };
    inst.attributes.imm = get_imm_sign_extended(&inst);
    inst.function = get_function(&inst);
    inst.controls = get_controls(&inst);
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
            && (attributes.funct3 == Some(0b001) || attributes.funct3 == Some(0b101))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_r() {
        // add x5, x6, x7
        let inst = 0x7302b3;
        let attributes = parse_format_r(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x33); // Op
        assert_eq!(attributes.funct3.unwrap(), 0x0);
        assert_eq!(attributes.rd.unwrap(), 0x05);
        assert_eq!(attributes.rs1.unwrap(), 0x06);
        assert_eq!(attributes.rs2.unwrap(), 0x07);
    }

    #[test]
    fn type_i_arithmetic() {
        // addi x5, x6, 20
        let inst = 0x01430293;
        let attributes = parse_format_i(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x13);
        assert_eq!(attributes.funct3.unwrap(), 0x0);
        assert_eq!(attributes.rd.unwrap(), 0x05);
        assert_eq!(attributes.rs1.unwrap(), 0x06);
        assert_eq!(attributes.imm.unwrap(), 20);
    }

    #[test]
    fn type_i_shift() {
        // slli x5, x6, 3
        let inst = 0x00331293;
        let attributes = parse_format_i(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x13);
        assert_eq!(attributes.funct3.unwrap(), 0x1);
        assert_eq!(attributes.rd.unwrap(), 0x05);
        assert_eq!(attributes.rs1.unwrap(), 0x06);
        assert_eq!(attributes.imm.unwrap(), 3);
    }

    #[test]
    fn type_i_load() {
        // lw a0, 0(sp)
        let inst = 0x00012503;
        let attributes = parse_format_i(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x03);
        assert_eq!(attributes.funct3.unwrap(), 0x2);
        assert_eq!(attributes.rd.unwrap(), 0x0a);
        assert_eq!(attributes.rs1.unwrap(), 0x02);
        assert_eq!(attributes.imm.unwrap(), 0);
    }

    #[test]
    fn type_i_jalr() {
        // jalr -96(a3)
        let inst = 0xfa0680e7;
        let wrapped_inst = Instruction::new(inst);
        assert_eq!(wrapped_inst.function, Function::JALR);
        let attributes = parse_format_i(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x67);
        assert_eq!(attributes.rs1.unwrap(), 13);
        let imm = get_imm_sign_extended(&wrapped_inst).unwrap() as i32;
        assert_eq!(imm, -96);
    }

    #[test]
    fn type_s() {
        // sw ra, 28(sp)
        let inst = 0x00112e23;
        let attributes = parse_format_s(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x23);
        assert_eq!(attributes.funct3.unwrap(), 0x2);
        assert_eq!(attributes.rs1.unwrap(), 0x02);
        assert_eq!(attributes.rs2.unwrap(), 0x01);
        assert_eq!(attributes.imm.unwrap(), 28);
    }

    #[test]
    fn type_b() {
        // beq x5, x6, 100
        let inst = 0x6628263;
        //let parsed_insn = Instruction::new(raw_insn);
        let attributes = parse_format_b(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x63);
        assert_eq!(attributes.funct3.unwrap(), 0x0);
        assert_eq!(attributes.rs1.unwrap(), 0x05);
        assert_eq!(attributes.rs2.unwrap(), 0x06);
        assert_eq!(attributes.imm.unwrap(), 100);

        let wrapped_inst = Instruction::new(inst);
        let imm = get_imm_sign_extended(&wrapped_inst).unwrap();
        assert_eq!(imm, 100);

        // bltu x13, x14, 16
        let inst = 0x00e6e863;
        let attributes = parse_format_b(inst);
        assert_eq!(attributes.imm.unwrap(), 16);
    }

    #[test]
    fn type_u() {
        // lui x5, 0x12345
        let insn = 0x123452b7;
        let fields = parse_format_u(insn);
        assert_eq!(fields.opcode.unwrap(), 0x37);
        assert_eq!(fields.rd.unwrap(), 0x05);
        assert_eq!(fields.imm.unwrap(), 0x12345000);
    }

    #[test]
    fn type_j() {
        // jal x1, 100
        let inst = 0x64000ef;
        let attributes = parse_format_j(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x6f);
        assert_eq!(attributes.rd.unwrap(), 0x01);
        assert_eq!(attributes.imm.unwrap(), 100);

        // jal x0, -136
        let inst = 0xf79ff06f;
        let attributes = parse_format_j(inst);
        assert_eq!(attributes.opcode.unwrap(), 0x6f);
        assert_eq!(attributes.rd.unwrap(), 0x00);

        let wrapped_inst = Instruction::new(inst);
        let imm = get_imm_sign_extended(&wrapped_inst).unwrap();
        assert_eq!(imm as i32, -136);

        let inst = 0x735000ef;
        let wrapped_inst = Instruction::new(inst);
        assert_eq!(wrapped_inst.attributes.imm.unwrap() as i32, 3892);
    }
}
