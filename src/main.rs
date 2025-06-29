mod config;

use std::env;
use std::process::Command;
use std::time::Instant;
use crate::config::Config;

fn main() -> Result<(), String> {
    let config = Config::new(env::args())?;

    let start_time = Instant::now();

    let mut command = Command::new(&config.program);
    command.args(&config.program_args);

    match command.status() {
        Ok(status) => {
            let duration = start_time.elapsed();
            println!("Program finished in: {:?}", duration);
            if !status.success() {
                eprintln!("Program exited with non-zero status: {}", status);
            }
        }
        Err(e) => {
            return Err(format!("Error executing program: {}", e));
        }
    }

    Ok(())
}