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
    pub capture: Option<capture::Capture>,
    pub database: String,
    pub output: String,
    pub filter: FilterMode,
    pub show: usize,
    pub runs: usize,
    pub warmups: usize,
    pub ignore_failure: bool,
    pub dry_run: bool,
    pub verbose: bool,
    pub reverse: bool,
    pub quiet: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            executable: String::new(),
            arguments: Vec::new(),
            note: String::new(),
            capture: None,
            database: String::from("timing.mesa"),
            output: String::from("stdout.txt"),
            filter: FilterMode::Exe,
            show: 8,
            runs: 3,
            warmups: 0,
            ignore_failure: false,
            dry_run: false,
            verbose: false,
            reverse: false,
            quiet: false,
        }
    }
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

Record options
    --dry-run                      do not save this run to the database
    --reverse                      bigger is better, makes only sense with capture
    --capture=...                  capture from output, instead of measuring time

Misc
    --verbose                      be more verbose
    -q, --quiet                    suppress progress display

Examples:
    {me} --note=\"important stuff\" --warmups=5 --runs=10 --output=stdout.json -- sleep 1
    {me} --runs=1 --capture=\"/bogomips/://\" --output=stdout.table -- cat /proc/cpuinfo
");
    }

    pub fn from_env() -> Result<Config, String> {
        let args: Vec<_> = env::args().skip(1).collect();
        Config::new(args)
    }

    fn new(args: Vec<String>) -> Result<Config, String> {
        let mut config = Config::default();

        let sep_pos = args.iter().position(|arg| arg == "--");
        let mine = if let Some(pos) = sep_pos {
            &args[..pos]
        } else {
            &args[..]
        };

        for arg in mine {
            if let Some((key, value)) = arg.split_once('=') {
                match key {
                    "-d" | "--database" => config.database = value.to_string(),
                    "-o" | "--output" => config.output = value.to_string(),
                    "--note" => config.note = value.to_string(),
                    "-f" | "--filter" => config.filter = match value {
                        "all" => FilterMode::All,
                        "exe" => FilterMode::Exe,
                        "exact" => FilterMode::Exact,
                        _ => return Err(format!("Unknown mode: {}", arg)),
                    },
                    "-s" | "--show" =>
                        config.show = value.parse::<usize>().map_err(|_| format!("Bad number: {}", arg))?,
                    "-r" | "--runs" =>
                        config.runs = value.parse::<usize>().map_err(|_| format!("Bad number: {}", arg))?,
                    "-w" | "--warmups" =>
                        config.warmups = value.parse::<usize>().map_err(|_| format!("Bad number: {}", arg))?,
                    "--capture" => {
                        let pattern = capture::parse(value)?;
                        config.capture = Some(pattern)
                    },
                    _ => return Err(format!("Unknown parameter: {}", arg)),
                }
            } else {
                match &arg[..] {
                    "-h" | "--help" => {
                        Config::help();
                        std::process::exit(0);
                    },
                    "-i" | "--ignore" => config.ignore_failure = true,
                    "-N" | "--dry-run" => config.dry_run = true,
                    "-V" | "--verbose" => config.verbose = true,
                    "--reverse" => config.reverse = true,
                    "-q" | "--quiet" => config.quiet = true,
                    _ => return Err(format!("Unknown flag: {}", arg)),
                }
            }
        }

        let yours = if let Some(pos) = sep_pos {
            &args[pos + 1..]
        } else {
            &[]
        };

        if yours.is_empty() {
            return Err("The target program is missing. Use '--' to separate `mesa` options from the program to be executed.".to_string());
        }

        config.executable = yours[0].to_string();
        config.arguments = yours[1..].to_vec();
        Ok(config)
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
        assert!(config.ignore_failure);
        assert!(config.dry_run);
        assert_eq!(config.note, "this is just a test");
        assert_eq!(config.executable, "proggy");
        assert_eq!(config.arguments, vec!["arg1"]);
    }
}
