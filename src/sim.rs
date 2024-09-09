use sim_lib::cpu::{CPUPolicy, Implementation};
use sim_lib::pipelined::branch_predictor::PredictorHeuristic;
use sim_lib::run_wrapper;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let elf_file = args.next().ok_or("You should specify exactly one ELF file")?;

    let mut policy = CPUPolicy::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-v" => policy.verbose = true,
            "-h" => policy.history = true,
            "-i" => {
                let impl_arg = args
                    .next()
                    .ok_or("You should specify an implementation after -i")?;
                policy.implementation = match impl_arg.as_str() {
                    "S" => Implementation::SingleCycle,
                    "P" => Implementation::Pipelined,
                    _ => {
                        return Err(
                            "Invalid implementation specified after -i".into()
                        )
                    }
                };
            }
            "-p" => {
                let heuristic_arg = args
                    .next()
                    .ok_or("You should specify a BP heuristic after -p")?;
                policy.heuristic = match heuristic_arg.as_str() {
                    "BP" => PredictorHeuristic::BufferedPrediction,
                    "ANT" => PredictorHeuristic::AlwaysNotTaken,
                    _ => {
                        return Err("Invalid BP heuristic specified after -p".into())
                    }
                };
            }
            _ => return Err(format!("Unknown parameter: {}", arg).into()),
        }
    }

    run_wrapper::run(&elf_file, policy)?;

    Ok(())
}
