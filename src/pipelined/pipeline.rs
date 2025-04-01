//! Pipeline state
use crate::error::SimulatorResult;
use crate::instruction::Instruction;

/// Pipeline state = 4 pipeline registers
#[derive(Clone, Copy, Default)]
pub struct PipelineState {
    pub if_id: IFIDRegister,
    pub id_ex: IDEXRegister,
    pub ex_mem: EXMEMRegister,
    pub mem_wb: MEMWBRegister,
}

impl PipelineState {
    /// Load-use hazard
    /// Let's modify the definition a little bit:
    /// We care about those instructions where the
    /// write-back result is determined AFTER the MEM stage.
    /// Thus we'll include LUI, AUIPC, JAL, JALR additionally.
    pub fn load_hazard(&self) -> SimulatorResult<bool> {
        use crate::instruction::Opcode::*;
        Ok(match self.id_ex.inst.opcode {
            Lui | AuiPc | Jal | Jalr | Load => {
                let if_id_inst = Instruction::new(self.if_id.raw_inst)?;
                if_id_inst.attributes.rs1 == self.id_ex.inst.attributes.rd
                    || if_id_inst.attributes.rs2
                        == self.id_ex.inst.attributes.rd
            }
            _ => false,
        })
    }

    /// Operand 1 can be forwarded from previous execution result
    /// See P&H p. 300
    pub fn ex_hazard_op1(&self) -> bool {
        self.id_ex.inst.attributes.rs1 != Some(0)
            && self.ex_mem.inst.controls.reg_write
            && self.ex_mem.inst.attributes.rd == self.id_ex.inst.attributes.rs1
    }

    /// Operand 2 can be forwarded from previous execution result
    /// See P&H p. 300
    pub fn ex_hazard_op2(&self) -> bool {
        self.id_ex.inst.attributes.rs2 != Some(0)
            && self.ex_mem.inst.controls.reg_write
            && self.ex_mem.inst.attributes.rd == self.id_ex.inst.attributes.rs2
    }

    /// Operand 1 can be forwarded from previous memory access result
    /// Precontidion: ex_hazard_op1 is false
    /// See P&H p. 301
    pub fn mem_hazard_op1(&self) -> bool {
        self.id_ex.inst.attributes.rs1 != Some(0)
            && self.mem_wb.inst.controls.reg_write
            && self.mem_wb.inst.attributes.rd == self.id_ex.inst.attributes.rs1
    }

    /// Operand 2 can be forwarded from previous memory access result
    /// Precontidion: ex_hazard_op2 is false
    /// See P&H p. 301
    pub fn mem_hazard_op2(&self) -> bool {
        self.id_ex.inst.attributes.rs2 != Some(0)
            && self.mem_wb.inst.controls.reg_write
            && self.mem_wb.inst.attributes.rd == self.id_ex.inst.attributes.rs2
    }

    /// Operand 1 was just written
    /// Must forward this explicitly due to load-use hazard
    /// See P&H p. 301
    pub fn wb_hazard_op1(&self, inst: &Instruction) -> bool {
        inst.attributes.rs1 != Some(0)
            && self.mem_wb.inst.controls.reg_write
            && inst.attributes.rs1 == self.mem_wb.inst.attributes.rd
    }

    /// Operand 2 was just written
    /// Must forward this explicitly due to load-use hazard
    /// See P&H p. 301
    pub fn wb_hazard_op2(&self, inst: &Instruction) -> bool {
        inst.attributes.rs2 != Some(0)
            && self.mem_wb.inst.controls.reg_write
            && inst.attributes.rs2 == self.mem_wb.inst.attributes.rd
    }
}

/// IF/ID register
#[derive(Clone, Copy)]
pub struct IFIDRegister {
    /// Program counter
    pub pc: u32,

    /// Raw instruction
    pub raw_inst: u32,
}

impl Default for IFIDRegister {
    fn default() -> Self {
        use crate::instruction::NOP;
        Self { pc: 0, raw_inst: NOP }
    }
}

/// ID/EX register
#[derive(Clone, Copy, Default)]
pub struct IDEXRegister {
    /// Program counter
    pub pc: u32,

    /// Wrapped instruction
    pub inst: Instruction,

    /// Operand 1
    pub op1: i32,
    /// Operand 2
    pub op2: i32,

    /// PC if branch is taken
    pub taken_pc: Option<u32>,
}

/// EX/MEM register
#[derive(Clone, Copy, Default)]
pub struct EXMEMRegister {
    /// Program counter
    pub pc: u32,

    /// Wrapped instruction
    pub inst: Instruction,

    /// Execution result
    pub exec_result: i32,

    /// Operand 2
    pub op2: i32,

    /// PC if branch is taken
    pub taken_pc: Option<u32>,

    /// PC of exitting
    pub exit_pc: Option<u32>,
}

/// MEM/WB register
#[derive(Clone, Copy, Default)]
pub struct MEMWBRegister {
    /// Program counter
    pub pc: u32,

    /// Wrapped instruction
    pub inst: Instruction,

    /// Actual write back result,
    /// which is computed during the MEM stage
    pub wb_result: u32,
}
