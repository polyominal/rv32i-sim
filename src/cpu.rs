//! rv32i CPU implementation

use crate::pipelined::branch_predictor::PredictorHeuristic;

/// CPU state
#[derive(Clone, Copy)]
pub struct CPUState {
    /// Stack base address
    pub stack_base: u32,
    /// Stack size
    pub stack_size: u32,
    /// Program counter
    pub pc: Register,
    /// General purpose registers
    pub gpr: [Register; 32],

    /// CPU policy
    pub policy: CPUPolicy,

    /// History of execution
    pub history: CPUHistory,
}

impl CPUState {
    pub fn make(policy: CPUPolicy) -> Self {
        Self {
            stack_base: 0,
            stack_size: 0,
            pc: Register::new(0),
            gpr: [Register::new(0); 32],
            policy,
            history: CPUHistory::default(),
        }
    }

    /// Checks for stack overflow
    pub fn stack_overflow(&self) -> bool {
        self.gpr[2].read() < self.stack_base - self.stack_size
    }

    /// Increments history cycle count
    pub fn update_cycle_count(&mut self, value: i32) {
        self.history.cycle_count += value;
    }

    /// Increments history instruction count
    pub fn update_inst_count(&mut self, value: i32) {
        self.history.inst_count += value;
    }
}

/// Register file simulation
#[derive(Clone, Copy)]
pub struct Register {
    /// Current data in the register
    data: u32,
}

impl Register {
    pub fn new(data: u32) -> Self {
        Self { data }
    }

    /// Reads the register
    pub fn read(&self) -> u32 {
        self.data
    }

    /// Writes to register
    pub fn write(&mut self, value: u32) {
        self.data = value;
    }
}

/// Implementation enum
#[derive(Clone, Copy, Default)]
pub enum Implementation {
    SingleCycle,
    #[default]
    Pipelined,
}

/// CPU policy
#[derive(Clone, Copy, Default)]
pub struct CPUPolicy {
    pub verbose: bool,
    pub implementation: Implementation,
    pub history: bool,
    pub heuristic: PredictorHeuristic,
}

/// History module
#[derive(Clone, Copy, Default)]
pub struct CPUHistory {
    pub cycle_count: i32,
    pub mem_stall_count: i32,
    pub mem_stall_worst_count: i32,
    pub inst_count: i32,
}
