use std::env;


#[derive(Debug)]
pub struct Config {
    pub executable: String,
    pub arguments: Vec<String>,
    pub database: String,
}

impl Config {
    pub fn help() {
        let me = env::args().next().unwrap();
        eprintln!("Usage: {me} [mesa options] -- <program> [program arguments]");
        eprintln!("Where options are:");
        eprintln!("   --database=<filename>          name of time database");
        eprintln!("");

    }
    pub fn build() -> Result<Config, String> {
        let args: Vec<_> = env::args().skip(1).collect();
        Config::new(args)
    }

    fn new(args: Vec<String>) -> Result<Config, String> {
        // default values
        let mut database = String::from(".mesa.data");

        // separate our own and targets arguments
        let sep_pos = args.iter().position(|arg| arg == "--");
        let mine = if let Some(pos) = sep_pos {
            &args[..pos]
        } else {
            &args[..]
        };

        // parse our own arguments
        for arg in mine {
            if let Some((key, value)) = arg.split_once('=') {
                match key {
                    "-d" | "--database" => database = value.to_string(),
                    _ => return Err(format!("Unknown parameter: {}", arg)),
                }
            } else {
                match &arg[..] {
                    "-h" | "--help" => {
                        Config::help();
                        std::process::exit(0); // Exit successfully
                    },
                    _ => return Err(format!("Unknown parameter: {}", arg)),
                }
            }
        }

        // Verify target arguments
        let yours = if let Some(pos) = sep_pos {
            &args[pos + 1..]
        } else {
            &[] // No separator means no program args.
        };

        if yours.is_empty() {
            return Err("The target program is missing. Use '--' to separate `mesa` options from the program to be executed.".to_string());
        }

        // Step 5: If all checks pass, build and return the final config.
        Ok(Config {
            executable: yours[0].clone(),
            arguments: yours[1..].to_vec(),
            database,
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_args() {
        let args = vec![];
        let config = Config::new(args);
        assert!(config.is_err());
    }

    #[test]
    fn test_no_arguments() {
        let args = vec!["--".to_string()];
        let config = Config::new(args);
        assert!(config.is_err());
    }

    #[test]
    fn test_no_program_executable() {
        let args = vec!["--".to_string()];
        let config = Config::new(args);
        assert!(config.is_err());
    }

    #[test]
    fn test_valid_config() {
        let args = vec!["--database=test.db".to_string(), "--".to_string(), "ls".to_string(), "-l".to_string()];
        let config = Config::new(args).unwrap();
        assert_eq!(config.database, "test.db");
        assert_eq!(config.executable, "ls");
        assert_eq!(config.arguments, vec!["-l"]);
    }
}
