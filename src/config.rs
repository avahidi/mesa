use std::env;

pub struct Config {
    pub program: String,
    pub program_args: Vec<String>,
}

impl Config {
    pub fn new(args: env::Args) -> Result<Config, String> {
        let collected_args: Vec<String> = args.collect();
        if let Some(pos) = collected_args.iter().position(|arg| arg == "--") {
            let program_args_slice = &collected_args[pos + 1..];
            if program_args_slice.is_empty() {
                return Err("No program specified after '--'".to_string());
            }

            let program = program_args_slice[0].clone();
            let program_args = program_args_slice[1..].to_vec();

            Ok(Config {
                program,
                program_args,
            })
        } else {
            Err("Usage: mesa -- <program> [args...]".to_string())
        }
    }
}