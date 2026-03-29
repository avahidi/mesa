use std::env;

use crate::capture;

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
    pub capture : Option<capture::Capture>,
    pub database: String,
    pub output: String,
    pub filter: FilterMode,
    pub show: usize,
    pub runs: usize,
    pub runs_warmup: usize,
    pub ignore_failure: bool,
    pub dry_run: bool,
    pub verbose: bool,
    pub reverse : bool,
}

impl Config {
    pub fn help() {
        let me = env::args().next().unwrap();
        eprintln!("
Usage: {me} [mesa options] -- <program> [program arguments]
Run options
    --database=<filename>          the database
    --note=<note>                  describe this run
    --runs=<number>                number of times target is run
    --warmups=<number>             number of warm up runs before the measurement
    --ignore                       ignore if application returned non-zero exit code

Output options
    --output=<filename>            output file (CSV/JSON/TXT/XML/...) or stdout
    --show=<number>                max number of items to show
    --filter=<mode>                filter mode: all, exe, exact
    --reverse                      bigger is better, makes only sense with capture

Record options
    --dry-run                      do not save this run to the database
    --capture=...                  capture from output, instead of measuring time

Misc
    --verbose                      be more verbose
");

    }
    pub fn build() -> Result<Config, String> {
        let args: Vec<_> = env::args().skip(1).collect();
        Config::new(args)
    }

    fn new(args: Vec<String>) -> Result<Config, String> {
        // default values
        let mut database = String::from("timing.mesa");
        let mut output = String::from("stdout.txt");
        let mut note = String::from("");
        let mut capture = None;
        let mut filter = FilterMode::Exe;
        let mut show = 8;
        let mut runs = 1;
        let mut runs_warmup = 0;
        let mut ignore_failure = false;
        let mut dry_run = false;
        let mut verbose = false;
        let mut reverse = false;

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
                    "-w" | "--warmups" =>
                        runs_warmup = value.parse::<usize>().map_err(|_| format!("Bad number: {}", arg))?,
                    "--capture" => {
                        let pattern = capture::parse(value)?;
                        capture = Some(pattern)
                    },
                    _ => return Err(format!("Unknown parameter: {}", arg)),
                }
            } else {
                match &arg[..] {
                    "-h" | "--help" => {
                        Config::help();
                        std::process::exit(0); // Exit successfully
                    },
                    "-i" | "--ignore" => ignore_failure = true,
                    "-N" | "--dry-run" => dry_run = true,
                    "-V" | "--verbose" => verbose = true,
                    "--reverse" => reverse = true,
                    _ => return Err(format!("Unknown flag: {}", arg)),
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
            executable: yours[0].to_string(),
            arguments: yours[1..].to_vec(),
            note,
            database,
            output,
            filter,
            show,
            runs,
            runs_warmup,
            ignore_failure,
            dry_run,
            verbose,
            capture,
            reverse,
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
