use std::process::Command;
use std::time::{Instant, Duration};

use mesa::*;

fn execute_once(config: &Config) -> Result<Duration, String> {
    let mut command = Command::new(&config.executable);
    command.args(&config.arguments);

    let start_time = Instant::now();
    let status = command.status().map_err(|e| format!("Error executing program: {}", e))?;
    if !status.success() && !config.ignore_failure {
        Err( format!("Program failed with error code {}", status) )
    } else {
        Ok( start_time.elapsed() )
    }
}

fn execute(config: &Config) -> Result<(f64, f64), String> {
    // run for <config.runs> time
    let durations: Vec<f64> = (0..config.runs)
        .map(|_| execute_once(config).map(|d| d.as_secs_f64()))
        .collect::<Result<Vec<f64>, String>>()?;

    // compute mean and standard deviation, return <0,0> if we have no data
    if durations.is_empty() {
        return Ok((0.0, 0.0));
    }
    let sum: f64 = durations.iter().sum();
    let mean = sum / (durations.len() as f64);
    let variance_sum: f64 = durations.iter().map(|&d| (d - mean).powi(2)).sum();
    let std_dev = (variance_sum / (durations.len() as f64)).sqrt();
    Ok((mean, std_dev))
}


fn main() -> Result<(), String> {
    let config = Config::build().map_err(|e| {
        Config::help();
        e
    })?;

    let mut db = Database::new(&config.database);
    db.load().unwrap_or_else( |err| eprintln!("Failed loading database: {:?}", err) );

    // run and save results to database
    let (mean, std_dev) = execute(&config)?;
    db.insert(&config, mean, std_dev)?;
    if !config.dry_run {
        db.save()?;
    }

    // show me what you get
    let search_result = db.search(&config);
    write_output(&config.output, search_result)
}
