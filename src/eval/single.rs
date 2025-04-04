use std::process;
use std::vec;

use sim_lib::error::SimulatorResult;
use sim_lib::memory::cache::CachePolicy;
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

    // Plot line series for each cache size
    // For a fixed cache size, varie the block size
    // Performance metric: miss rate
    // Cache sizes: 4KB, 16KB, 64KB, 256KB, 1MB
    let cache_sizes = [4 * 1024, 16 * 1024, 64 * 1024, 256 * 1024, 1024 * 1024];
    // Block sizes: 32B, 64B, 128B, 256B
    let block_sizes = [32, 64, 128, 256];

    // Propagate the data
    let mut data: Vec<Vec<(usize, f64)>> = vec![vec![]; cache_sizes.len()];
    let mut y_max: f64 = 0.;
    for (i, cache_size) in cache_sizes.iter().enumerate() {
        for block_size in block_sizes.iter() {
            let mut mem = InclusiveCache::make(
                vec![CachePolicy::make(*cache_size, *block_size, 1, 1)],
                Default::default(),
                Default::default(),
                100,
                false,
            );
            let amat = run_trace(&mut mem, trace_path)?;
            data[i].push((*block_size, amat));
            y_max = y_max.max(amat);
        }
    }
    // Plot the data
    use plotters::prelude::*;

    let trace_base_name =
        String::from(trace_path.split('/').last().unwrap_or("unknown"));
    let plot_title =
        format!("Single level evaluation (AMAT): {}", trace_base_name);
    let output_path = format!("eval/single_eval_{}.svg", trace_base_name);

    let root =
        SVGBackend::new(output_path.as_str(), (800, 600)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| {
        sim_lib::error::SimulatorError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create SVG: {}", e),
        ))
    })?;

    let mut ctx = ChartBuilder::on(&root)
        .caption(plot_title.as_str(), ("sans-serif", 40).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(32..256, 0.0..y_max * 1.1)
        .map_err(|e| {
            sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to build chart: {}", e),
            ))
        })?;

    ctx.configure_mesh().x_desc("Block size").y_desc("AMAT").draw().map_err(
        |e| {
            sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to configure mesh: {}", e),
            ))
        },
    )?;

    for (i, cache_size) in cache_sizes.iter().enumerate() {
        let series = data[i].iter().map(|(x, y)| (*x as i32, *y));
        let label = format!("Cache size = {}", cache_size);
        let color = Palette99::pick(i).to_rgba();
        ctx.draw_series(LineSeries::new(series, color))
            .map_err(|e| {
                sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to draw series: {}", e),
                ))
            })?
            .label(label)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color)
            });
    }

    ctx.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .map_err(|e| {
            sim_lib::error::SimulatorError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to draw labels: {}", e),
            ))
        })?;

    Ok(())
}
