use sim_lib::cpu::CPUPolicy;
use sim_lib::error::SimulatorResult;
use sim_lib::flags::RvSimArgs;
use sim_lib::run_wrapper;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> SimulatorResult<()> {
    let args = RvSimArgs::from_env_or_exit();
    let mut policy = CPUPolicy::default();
    if args.verbose {
        policy.verbose = true;
    }
    if args.history {
        policy.history = true;
    }

    if let Some(backend_arg) = args.implementation {
        policy.implementation = backend_arg.into();
    }
    if let Some(heuristic_arg) = args.prediction {
        policy.heuristic = heuristic_arg.into();
    }

    let elf_file_path_str = args.elf_file.display().to_string();
    run_wrapper::run(&elf_file_path_str, policy)?;

    Ok(())
}
