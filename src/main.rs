mod config;
mod database;

use std::process::Command;
use std::time::Instant;
use crate::config::Config;
use crate::database::Database;

fn main() -> Result<(), String> {
    let config = Config::build().map_err(|e| {
        Config::help();
        e
    })?;

    let mut db = Database::new(&config.database);
    db.load().unwrap_or_else( |err| eprintln!("Failed loading database: {:?}", err) );

    let start_time = Instant::now();
    let mut command = Command::new(&config.executable);
    command.args(&config.arguments);

    let status = command.status().map_err(|e| format!("Error executing program: {}", e))?;
    let duration = start_time.elapsed();
    println!("Program finished in: {:?}", duration);
    if status.success() {
        db.insert(&config.executable, &config.arguments, duration.as_secs_f64())?;
        db.save()?;
    } else {
        eprintln!("Program exited with non-zero status: {}", status);
    }

    Ok(())
}
