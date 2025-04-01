use std::process;

use sim_lib::error::SimulatorResult;
use sim_lib::memory::cache::CachePolicy;
use sim_lib::memory::exclusive::ExclusiveCache;
use sim_lib::memory::inclusive::InclusiveCache;
use sim_lib::run_wrapper::run_trace;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> SimulatorResult<()> {
    let param_tokens: Vec<String> = std::env::args().collect();
    let trace_path = param_tokens.get(1).ok_or_else(|| {
        sim_lib::error::SimulatorError::ConfigError(
            "You should specify exactly one trace file".to_string(),
        )
    })?;

    let trace_base_name =
        String::from(trace_path.split('/').last().unwrap_or("unknown"));
    let output_path = format!("eval/multi_eval_{}.csv", trace_base_name);

    let mut writer = csv::Writer::from_path(&output_path).map_err(|e| {
        sim_lib::error::SimulatorError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create CSV file '{}': {}", output_path, e),
        ))
    })?;

    writer.write_record(["Policy", "AMAT"]).map_err(|e| {
        sim_lib::error::SimulatorError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to write header to CSV: {}", e),
        ))
    })?;

    // Default single-level cache
    {
        let mut mem = InclusiveCache::make(
            vec![CachePolicy::default()],
            Default::default(),
            Default::default(),
            100,
            false,
        );
        let amat = run_trace(&mut mem, trace_path)?;
        writer
            .write_record(["Single-level", &format!("{:.3}", amat)])
            .map_err(|e| {
                sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to write record to CSV: {}", e),
                ))
            })?;
    }

    // Default 3-level inclusive cache
    {
        let mut mem = InclusiveCache::default();
        let amat = run_trace(&mut mem, trace_path)?;
        mem.verify_inclusiveness()?;
        writer
            .write_record(["Multi-level inclusive", &format!("{:.3}", amat)])
            .map_err(|e| {
                sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to write record to CSV: {}", e),
                ))
            })?;
    }

    // 3-level inclusive cache with victim cache
    {
        let mut mem = InclusiveCache::default();
        mem.use_victim_cache = true;
        let amat = run_trace(&mut mem, trace_path)?;
        mem.verify_inclusiveness()?;
        writer
            .write_record([
                "Multi-level inclusive with VC",
                &format!("{:.3}", amat),
            ])
            .map_err(|e| {
                sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to write record to CSV: {}", e),
                ))
            })?;
    }

    // 3-level exclusive cache
    {
        let mut mem = ExclusiveCache::default();
        let amat = run_trace(&mut mem, trace_path)?;
        mem.verify_exclusiveness()?;
        writer
            .write_record(["Multi-level exclusive", &format!("{:.3}", amat)])
            .map_err(|e| {
                sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to write record to CSV: {}", e),
                ))
            })?;
    }

    Ok(())
}
