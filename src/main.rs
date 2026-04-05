use std::process::{Command, Stdio};
use std::time::Instant;

use mesa::*;

fn execute_once(config: &Config) -> Result<f64, String> {
    let mut command = Command::new(&config.executable);
    command.args(&config.arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        ;

    let start_time = Instant::now();
    let output = command
        .output().map_err(|e| format!("Error executing program: {}", e))?;

    // value is up here to not include the verbose print time below
    let value = config.capture.as_ref()
        .and_then(|c| c.extract(&String::from_utf8_lossy(&output.stdout)))
        .map(|s| s.trim().parse::<f64>())
        .transpose()
        .map_err(|_| "search pattern not found".to_string())?
        .unwrap_or_else(|| start_time.elapsed().as_secs_f64());


    if config.verbose {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    let status = output.status;
    if !status.success() && !config.ignore_failure {
        Err( format!("Program failed with error code {}", status) )
    } else {
        Ok( value )
    }
}

fn execute(config: &Config, count: usize, warmup: bool, prev_mean: Option<f64>, prev_std: Option<f64>) -> Result<(f64, f64), String> {
    if count == 0 {
        return Ok((0.0, 0.0));
    }

    let label = if warmup { "Warmup"} else { "Benchmark" };
    let mut measurements = Vec::with_capacity(count);
    let mut bar = Progress::new(count, label, config.quiet, prev_mean, prev_std);
    for _ in 0..count {
        bar.start();
        let measurement = execute_once(config)?;
        measurements.push(measurement);
        bar.stop(measurement);
    }
    bar.finish(warmup);

    let sum: f64 = measurements.iter().sum();
    let mean = sum / (count as f64);
    let variance_sum: f64 = measurements.iter().map(|&d| (d - mean).powi(2)).sum();
    let std_dev = (variance_sum / (measurements.len() as f64)).sqrt();
    Ok((mean, std_dev))
}


fn main() -> Result<(), String> {
    let config = Config::from_env().map_err(|e| {
        Config::help();
        e
    })?;

    let mut db = Database::new(&config.database);
    db.load().unwrap_or_else( |err| eprintln!("Failed loading database: {:?}", err) );


    // warmup round:
    let (warmup_mean, warmup_std) = execute(&config, config.warmups, true, None, None)?;
    if config.warmups > 0 && config.verbose {
        eprintln!("After {} warmup rounds: mean={:3.3}s stddev={:3.3}",
                 config.warmups, warmup_mean, warmup_std);
    }

    let (prev_mean, prev_std) = if config.warmups > 0 { (Some(warmup_mean), Some(warmup_std)) } else { (None, None) };
    let (mean, std_dev) = execute(&config, config.runs, false, prev_mean, prev_std)?;

    // record the outcome
    db.insert(&config, mean, std_dev)?;
    if !config.dry_run {
        db.save()?;
    }

    // show me what you get
    let search_result = db.search(&config);
    write_output(&config.output, search_result, config.reverse)
}
