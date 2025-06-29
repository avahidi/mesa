mod config;
mod database;

use std::process::Command;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
use crate::config::Config;
use crate::database::{Database, Entry};

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

fn show(measurements: Vec<&Entry>) -> Result<(), String> {
    if measurements.is_empty() {
        println!("No previous runs to show.");
        return Ok(());
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get system time: {}", e))?.as_secs();

    // figure out what data we want to print, starting with the headers
    let mut columns: Vec<Vec<String>> = vec![
        vec!["Age".to_string()],
        vec!["Executable".to_string()],
        vec!["Arguments".to_string()],
        vec!["Runs".to_string()],
        vec!["Mean (s)".to_string()],
        vec!["StdDev (s)".to_string()],
        vec!["Change (%)".to_string()],
    ];

    columns[0].extend(measurements.iter().map(|entry| entry.age(now)));
    columns[1].extend(measurements.iter().map(|entry| entry.executable.clone()));
    columns[2].extend(measurements.iter().map(|entry| entry.arguments.clone()));
    columns[3].extend(measurements.iter().map(|entry| entry.runs.to_string()));
    columns[4].extend(measurements.iter().map(|entry| format!("{:.4}", entry.time_mean)));
    columns[5].extend(measurements.iter().map(|entry| format!("{:.4}", entry.time_stddev)));

    let first_mean = measurements.first().map(|entry| entry.time_mean).unwrap_or(0.0);
    columns[6].extend(measurements.iter().enumerate().map(|(i, entry)| {
        if i == 0 {
            " ".to_string() // Empty string for the first entry
        } else if first_mean == 0.0 {
            "N/A".to_string()
        } else {
            format!("{:.2}", ((entry.time_mean - first_mean) / first_mean) * 100.0)
        }
    }));

    // to print a nice table we will need to know max width for each column
    let widths: Vec<usize> = columns.iter().map(|col| {
        col.iter().map(|s| s.len()).max().unwrap_or(0)
    }).collect();

    // lets print the table now
    for i in 0..columns[0].len() {
        let row_strings: Vec<String> = columns.iter().zip(&widths).enumerate()
            .map(|(_j, (col, w))| format!("{:>width$}", col[i], width = w) )
            .collect();
        println!("{}", row_strings.join(" "));

        // line separator between header and data
        if i == 0 {
            println!("{}", widths.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>().join(" ") );
        }
    }
    Ok(())
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
    db.save()?;

    // show me what you get
    let search_result = db.search(&config);
    show(search_result)?;

    Ok(())
}
