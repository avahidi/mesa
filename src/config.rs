use std::env;

#[derive(Debug,PartialEq)]
pub enum FilterMode {
    All,
    Exe,
    Exact,
}

#[derive(Debug)]
pub struct Config {
    pub executable: String,
    pub arguments: Vec<String>,
    pub note: String,
    pub database: String,
    pub output: String,
    pub filter: FilterMode,
    pub show: usize,
    pub runs: usize,
    pub ignore_failure: bool,
    pub dry_run: bool,
}

impl Config {
    pub fn help() {
        let me = env::args().next().unwrap();
        eprintln!("Usage: {me} [mesa options] -- <program> [program arguments]");
        eprintln!("Where options are:");
        eprintln!("   --database=<filename>          name of time database");
        eprintln!("   --output=<filename>            output file (CSV/JSON/TXT/XML/...) or stdout");
        eprintln!("   --note=...                     description about this run");
        eprintln!("   --runs=<number>                number of times target is run");
        eprintln!("   --filter=<mode>                filter mode: all, exe, exact");
        eprintln!("   --show=<number>                max number of items to show");
        eprintln!("   --ignore                       ignore if application returns non-zero");
        eprintln!("   --dry-run                      do not save this run to the database");

        eprintln!("");

    }
    pub fn build() -> Result<Config, String> {
        let args: Vec<_> = env::args().skip(1).collect();
        Config::new(args)
    }

    fn new(args: Vec<String>) -> Result<Config, String> {
        // default values
        let mut database = String::from(".mesa.data");
        let mut output = String::from("stdout");
        let mut note = String::from("");
        let mut filter = FilterMode::Exe;
        let mut show = 5;
        let mut runs = 1;
        let mut ignore_failure = false;
        let mut dry_run = false;

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
                    "-o" | "--output" => output = value.to_string(),
                    "--note" => note = value.to_string(),

                    "-f" | "--filter" => filter = match value {
                        "all" => FilterMode::All,
                        "exe" => FilterMode::Exe,
                        "exact" => FilterMode::Exact,
                        _ => return Err(format!("Unknown mode: {}", arg)),
                    },
                    "-s" | "--show" =>
                        show = value.parse::<usize>().map_err(|_| format!("Bad number: {}", arg))?,
                    "-r" | "--runs" =>
                        runs = value.parse::<usize>().map_err(|_| format!("Bad number: {}", arg))?,
                    _ => return Err(format!("Unknown parameter: {}", arg)),
                }
            } else {
                match &arg[..] {
                    "-h" | "--help" => {
                        Config::help();
                        std::process::exit(0); // Exit successfully
                    },
                    "-i" | "--ignore" => ignore_failure = true,
                    "--dry-run" => dry_run = true,
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
            note,
            database,
            output,
            filter,
            show,
            runs,
            ignore_failure,
            dry_run,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_args() {
        let args = vec![];
        let config = Config::new(args);
        assert!(config.is_err());

        let args = vec!["--".to_string()];
        let config = Config::new(args);
        assert!(config.is_err());

        let args = vec!["-i".to_string(), "--".to_string()];
        let config = Config::new(args);
        assert!(config.is_err());
    }

    #[test]
    fn test_config_options() {
        let args: Vec<String> = vec![
            "--database=/this/mesa.data",
            "--output=/that/output.txt",
            "--note=this is just a test",
            "--filter=all",
            "--show=10",
            "--runs=17",
            "--ignore",
            "--dry-run",
            "--",
            "proggy",
            "arg1",
        ].into_iter().map(String::from).collect();
        let config = Config::new(args).unwrap();
        assert_eq!(config.database, "/this/mesa.data");
        assert_eq!(config.output, "/that/output.txt");
        assert_eq!(config.filter, FilterMode::All);
        assert_eq!(config.show, 10);
        assert_eq!(config.runs, 17);
        assert_eq!(config.ignore_failure, true);
        assert_eq!(config.dry_run, true);
        assert_eq!(config.note, "this is just a test");
        assert_eq!(config.executable, "proggy");
        assert_eq!(config.arguments, vec!["arg1"]);
    }
}
