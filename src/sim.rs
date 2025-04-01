use std::env;

use sim_lib::cpu::CPUPolicy;
use sim_lib::cpu::Implementation;
use sim_lib::error::SimulatorError;
use sim_lib::error::SimulatorResult;
use sim_lib::pipelined::branch_predictor::PredictorHeuristic;
use sim_lib::run_wrapper;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> SimulatorResult<()> {
    let mut args = env::args().skip(1);

    let elf_file = args.next().ok_or_else(|| {
        SimulatorError::ConfigError(
            "You should specify exactly one ELF file".to_string(),
        )
    })?;

    let mut policy = CPUPolicy::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-v" => policy.verbose = true,
            "-h" => policy.history = true,
            "-i" => {
                let impl_arg = args.next().ok_or_else(|| {
                    SimulatorError::ConfigError(
                        "You should specify an implementation after -i"
                            .to_string(),
                    )
                })?;

                policy.implementation = match impl_arg.as_str() {
                    "S" => Implementation::SingleCycle,
                    "P" => Implementation::Pipelined,
                    _ => {
                        return Err(SimulatorError::ConfigError(
                            "Invalid implementation specified after -i"
                                .to_string(),
                        ));
                    }
                };
            }
            "-p" => {
                let heuristic_arg = args.next().ok_or_else(|| {
                    SimulatorError::ConfigError(
                        "You should specify a BP heuristic after -p"
                            .to_string(),
                    )
                })?;

                policy.heuristic = match heuristic_arg.as_str() {
                    "BP" => PredictorHeuristic::BufferedPrediction,
                    "ANT" => PredictorHeuristic::AlwaysNotTaken,
                    _ => {
                        return Err(SimulatorError::ConfigError(
                            "Invalid BP heuristic specified after -p"
                                .to_string(),
                        ));
                    }
                };
            }
            _ => {
                return Err(SimulatorError::ConfigError(format!(
                    "Unknown parameter: {}",
                    arg
                )));
            }
        }
    }

    run_wrapper::run(&elf_file, policy)?;

    Ok(())
}
