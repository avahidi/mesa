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

    // value is up here to not not include the verbose print time below
    let value = match &config.capture {
        Some(capture) => {
            // we are measuring something else
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let cap = capture.extract(&stdout).ok_or("search pattern not found")?;
            cap.as_str().trim().parse::<f64>()
                .map_err(|_| format!("Captured text {} is not a valid number", cap))?
        },
        _ => start_time.elapsed().as_secs_f64()
    };


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

fn execute(config: &Config, count: usize) -> Result<(f64, f64), String> {
    if count == 0 {
        return Ok((0.0, 0.0));
    }
    let durations: Vec<f64> = (0..count)
        .map(|_| execute_once(config))
        .collect::<Result<Vec<f64>, String>>()?;

    let sum: f64 = durations.iter().sum();
    let mean = sum / (count as f64);
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


    // warmup round:
    let (mean, std_dev) = execute(&config, config.runs_warmup)?;
    if config.runs_warmup > 0 && config.verbose {
        println!("After {} warmup rounds: mean={:3.3}s stddev={:3.3}",
                 config.runs_warmup, mean, std_dev);
    }

    // real rounds
    let (mean, std_dev) = execute(&config, config.runs)?;

    // record the outcome
    db.insert(&config, mean, std_dev)?;
    if !config.dry_run {
        db.save()?;
    }

    // show me what you get
    let search_result = db.search(&config);
    write_output(&config.output, search_result, config.reverse)
}
