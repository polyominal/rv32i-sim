use std::path::PathBuf;
use std::str::FromStr;

use crate::cpu::Implementation;
use crate::pipelined::branch_predictor::PredictorHeuristic;

xflags::xflags! {
    /// RISC-V RV32I Instruction Set Simulator.
    cmd RvSimArgs {
        /// Path to the ELF file to simulate.
        required elf_file: PathBuf

        /// Enables history module, printing cycle and instruction counts after simulation.
        optional --history

        /// Specifies the simulator implementation.
        /// P: Pipelined (default)
        /// S: Naive single-cycle
        optional -i, --implementation backend: BackendArg

        /// Specifies the branch prediction heuristic.
        /// BP: Buffered prediction (default for pipelined)
        /// ANT: Always not taken
        optional -p, --prediction heuristic: HeuristicArg

        /// Enables verbose mode, printing detailed information during simulation.
        /// Largely used for debugging purposes.
        optional -v, --verbose
    }
}

#[derive(Debug)]
pub enum BackendArg {
    Pipelined,
    SingleCycle,
}

impl FromStr for BackendArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "P" => Ok(BackendArg::Pipelined),
            "S" => Ok(BackendArg::SingleCycle),
            _ => Err(format!(
                "Invalid implementation: '{}'. Expected 'P' or 'S'.",
                s
            )),
        }
    }
}

impl From<BackendArg> for Implementation {
    fn from(val: BackendArg) -> Self {
        match val {
            BackendArg::Pipelined => Implementation::Pipelined,
            BackendArg::SingleCycle => Implementation::SingleCycle,
        }
    }
}

#[derive(Debug)]
pub enum HeuristicArg {
    BufferedPrediction,
    AlwaysNotTaken,
}

impl FromStr for HeuristicArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BP" => Ok(HeuristicArg::BufferedPrediction),
            "ANT" => Ok(HeuristicArg::AlwaysNotTaken),
            _ => Err(format!(
                "Invalid branch prediction heuristic: '{}'. Expected 'BP' or 'ANT'.",
                s
            )),
        }
    }
}

impl From<HeuristicArg> for PredictorHeuristic {
    fn from(val: HeuristicArg) -> Self {
        match val {
            HeuristicArg::BufferedPrediction => {
                PredictorHeuristic::BufferedPrediction
            }
            HeuristicArg::AlwaysNotTaken => PredictorHeuristic::AlwaysNotTaken,
        }
    }
}
