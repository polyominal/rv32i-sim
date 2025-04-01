use std::process;

use sim_lib::cpu::CPUPolicy;
use sim_lib::error::SimulatorResult;
use sim_lib::run_wrapper::run;

fn main() {
    if let Err(e) = run_eval() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_eval() -> SimulatorResult<()> {
    let output_path = "eval/sim_eval.csv".to_string();
    let mut writer = csv::Writer::from_path(&output_path).map_err(|e| {
        sim_lib::error::SimulatorError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create CSV file '{}': {}", output_path, e),
        ))
    })?;

    writer
        .write_record([
            "Program",
            "CPI (ideal)",
            "CPI (caching)",
            "CPI (no caching)",
            "Ratio",
        ])
        .map_err(|e| {
            sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write header to CSV: {}", e),
            ))
        })?;

    let programs = vec![
        "add",
        "mul-div",
        "n!",
        "qsort",
        "simple-function",
        "ackermann",
        "helloworld",
        "matrixmulti",
        "quicksort",
        "test_arithmetic",
        "test_branch",
    ];

    for program in programs {
        let program_path = format!("test/{}.riscv", program);
        eprintln!("Running program: {}", program_path);

        match run(&program_path, CPUPolicy::default()) {
            Ok((ideal_cpi, caching_cpi, no_caching_cpi, ratio)) => {
                writer
                    .write_record([
                        program,
                        &format!("{:.3}", ideal_cpi),
                        &format!("{:.3}", caching_cpi),
                        &format!("{:.3}", no_caching_cpi),
                        &format!("{:.3}", ratio),
                    ])
                    .map_err(|e| {
                        sim_lib::error::SimulatorError::IoError(
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Failed to write record to CSV: {}", e),
                            ),
                        )
                    })?;
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to run program '{}': {}",
                    program, e
                );
                writer
                    .write_record([program, "Error", "Error", "Error", "Error"])
                    .map_err(|e| {
                        sim_lib::error::SimulatorError::IoError(
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Failed to write record to CSV: {}", e),
                            ),
                        )
                    })?;
            }
        }
    }

    Ok(())
}
