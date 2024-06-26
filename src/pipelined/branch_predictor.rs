//! Black-box branch predictor
//! that supports predicting and updating based on observed branch behavior

const PREDICTOR_BUFFER_SIZE: usize = 4096;

#[derive(Clone, Copy, PartialEq)]
pub enum PredictorHeuristic {
    AlwaysNotTaken,
    BufferedPrediction,
}

impl Default for PredictorHeuristic {
    fn default() -> Self {
        PredictorHeuristic::BufferedPrediction
    }
}

#[derive(Clone, Copy)]
enum PredictorState {
    StronglyTaken = 0,
    WeaklyTaken = 1,
    WeaklyNotTaken = 2,
    StronglyNotTaken = 3,
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
            buffer: Box::new(
                [PredictorState::WeaklyTaken; PREDICTOR_BUFFER_SIZE],
            ),
        }
    }

    pub fn predict(&self, pc: u32) -> bool {
        if self.heuristic != PredictorHeuristic::BufferedPrediction {
            // Always not taken
            return false;
        }

        let index = (pc as usize) % PREDICTOR_BUFFER_SIZE;
        match self.buffer[index] {
            PredictorState::StronglyTaken | PredictorState::WeaklyTaken => true,
            PredictorState::WeaklyNotTaken
            | PredictorState::StronglyNotTaken => false,
        }
    }

    pub fn update(&mut self, pc: u32, branch: bool) {
        if self.heuristic != PredictorHeuristic::BufferedPrediction {
            // Do nothing
            return;
        }

        let index = (pc as usize) % PREDICTOR_BUFFER_SIZE;
        let state = &mut self.buffer[index];
        use PredictorState::*;
        if branch {
            // Branch taken: decrement the state
            *state = match state {
                StronglyNotTaken => WeaklyNotTaken,
                WeaklyNotTaken => WeaklyTaken,
                WeaklyTaken => StronglyTaken,
                StronglyTaken => StronglyTaken,
            };
        } else {
            // Branch not taken: increment the state
            *state = match state {
                StronglyTaken => WeaklyTaken,
                WeaklyTaken => WeaklyNotTaken,
                WeaklyNotTaken => StronglyNotTaken,
                StronglyNotTaken => StronglyNotTaken,
            };
        }
    }
}
