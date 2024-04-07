use sim_lib::cpu::CPUPolicy;
use sim_lib::run_wrapper::run;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = format!("eval/sim_eval.csv");
    let mut writer = csv::Writer::from_path(output_path)?;
    writer.write_record(&[
        "Program",
        "CPI (ideal)",
        "CPI (caching)",
        "CPI (no caching)",
        "Ratio",
    ])?;

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
        let (ideal_cpi, caching_cpi, no_caching_cpi, ratio) =
            run(&program_path, CPUPolicy::default())?;
        writer.write_record(&[
            program,
            &format!("{:.3}", ideal_cpi),
            &format!("{:.3}", caching_cpi),
            &format!("{:.3}", no_caching_cpi),
            &format!("{:.3}", ratio),
        ])?;
    }

    Ok(())
}
