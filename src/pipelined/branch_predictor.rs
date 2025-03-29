//! Black-box branch predictor
//! that supports predicting and updating based on observed branch behavior

const PREDICTOR_BUFFER_SIZE: usize = 4096;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum PredictorHeuristic {
    AlwaysNotTaken,
    #[default]
    BufferedPrediction,
}

#[derive(Clone, Copy)]
enum PredictorState {
    Strongly = 0,
    Weakly = 1,
    WeaklyNot = 2,
    StronglyNot = 3,
}

/// Reference: <https://github.com/hehao98/RISCV-Simulator/blob/master/src/BranchPredictor.cpp>
pub struct BranchPredictor {
    heuristic: PredictorHeuristic,
    buffer: Box<[PredictorState; PREDICTOR_BUFFER_SIZE]>,
}

impl BranchPredictor {
    pub fn new(heuristic: PredictorHeuristic) -> Self {
        Self {
            heuristic,
            buffer: Box::new([PredictorState::Weakly; PREDICTOR_BUFFER_SIZE]),
        }
    }

    pub fn predict(&self, pc: u32) -> bool {
        if self.heuristic != PredictorHeuristic::BufferedPrediction {
            // Always not taken
            return false;
        }

        let index = (pc as usize) % PREDICTOR_BUFFER_SIZE;
        match self.buffer[index] {
            PredictorState::Strongly | PredictorState::Weakly => true,
            PredictorState::WeaklyNot | PredictorState::StronglyNot => false,
        }
    }

    pub fn update(&mut self, pc: u32, branch: bool) {
        if self.heuristic != PredictorHeuristic::BufferedPrediction {
            // Do nothing
            return;
        }

        let index = (pc as usize) % PREDICTOR_BUFFER_SIZE;
        let state = &mut self.buffer[index];
        if branch {
            // Branch taken: decrement the state
            *state = match state {
                PredictorState::StronglyNot => PredictorState::WeaklyNot,
                PredictorState::WeaklyNot => PredictorState::Weakly,
                PredictorState::Weakly => PredictorState::Strongly,
                PredictorState::Strongly => PredictorState::Strongly,
            };
        } else {
            // Branch not taken: increment the state
            *state = match state {
                PredictorState::Strongly => PredictorState::Weakly,
                PredictorState::Weakly => PredictorState::WeaklyNot,
                PredictorState::WeaklyNot => PredictorState::StronglyNot,
                PredictorState::StronglyNot => PredictorState::StronglyNot,
            };
        }
    }
}
