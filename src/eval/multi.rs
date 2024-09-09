use sim_lib::memory::cache::CachePolicy;
use sim_lib::memory::exclusive::ExclusiveCache;
use sim_lib::memory::inclusive::InclusiveCache;
use sim_lib::run_wrapper::run_trace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let param_tokens: Vec<String> = std::env::args().collect();
    let trace_path =
        param_tokens.get(1).ok_or("You should specify exactly one trace file")?;
    let trace_base_name = String::from(trace_path.split('/').last().unwrap());
    let output_path = format!("eval/multi_eval_{}.csv", trace_base_name);

    let mut writer = csv::Writer::from_path(output_path)?;
    writer.write_record(&["Policy", "AMAT"])?;

    // Default single-level cache
    {
        let mut mem = InclusiveCache::make(
            vec![CachePolicy::default()],
            Default::default(),
            Default::default(),
            100,
            false,
        );
        let amat = run_trace(&mut mem, &trace_path);
        writer.write_record(&["Single-level", &format!("{:.3}", amat)])?;
    }

    // Default 3-level inclusive cache
    {
        let mut mem = InclusiveCache::default();
        let amat = run_trace(&mut mem, &trace_path);
        mem.verify_inclusiveness();
        writer.write_record(&["Multi-level inclusive", &format!("{:.3}", amat)])?;
    }

    // 3-level inclusive cache with victim cache
    {
        let mut mem = InclusiveCache::default();
        mem.use_victim_cache = true;
        let amat = run_trace(&mut mem, &trace_path);
        mem.verify_inclusiveness();
        writer.write_record(&[
            "Multi-level inclusive with VC",
            &format!("{:.3}", amat),
        ])?;
    }

    // 3-level exclusive cache
    {
        let mut mem = ExclusiveCache::default();
        let amat = run_trace(&mut mem, &trace_path);
        mem.verify_exclusiveness();
        writer.write_record(&["Multi-level exclusive", &format!("{:.3}", amat)])?;
    }

    Ok(())
}
