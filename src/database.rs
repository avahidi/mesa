use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{Config, FilterMode};

const DB_HEADER: &str = "# mesa database|github.com/avahidi/mesa|version=1.2";

#[derive(Debug)]
pub struct Entry {
    pub timestamp: u64,
    pub executable: String,
    pub arguments: String,
    pub note: String,
    pub runs: usize,
    pub time_mean: f64,
    pub time_stddev: f64,
}

impl FromStr for Entry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 7 {
            return Err(format!("Invalid entry format: {}", s));
        }

        Ok(Entry {
            timestamp: parts[0].parse().map_err(|e| format!("Invalid timestamp: {}", e))?,
            executable: parts[1].to_string(),
            arguments: parts[2].to_string(),
            runs: parts[3].parse().map_err(|e| format!("Invalid run count: {}", e))?,
            time_mean: parts[4].parse().map_err(|e| format!("Invalid execution time: {}", e))?,
            time_stddev: parts[5].parse().map_err(|e| format!("Invalid std dev: {}", e))?,
            note: parts[6].to_string(),
        })
    }
}

impl ToString for Entry {
    fn to_string(&self) -> String {
        format!("{}|{}|{}|{}|{}|{}|{}",
                self.timestamp, self.executable, self.arguments, self.runs,
                self.time_mean, self.time_stddev, self.note,
        )
    }
}

impl Entry {
    pub fn age(&self, from: u64) -> String {
        let diff_secs = from.saturating_sub(self.timestamp);
        if diff_secs == 0 {
            "just now".to_string()
        } else {
            let hours = diff_secs / 3600;
            let minutes = (diff_secs % 3600) / 60;
            let seconds = diff_secs % 60;
            format!("{:4}:{:02}:{:02} ago", hours, minutes, seconds)
        }
    }
}

pub struct Database {
    pub entries: Vec<Entry>,
    filename: String,
}

impl Database {
    pub fn new(filename: &str) -> Database {
        Database {
            entries: Vec::new(),
            filename: filename.to_string(),
        }
    }

    pub fn load(&mut self) -> Result<(), String> {
        // no error if file doesn't exist
        if !std::path::Path::new(&self.filename).exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.filename)
            .map_err(|e| format!("Failed to read database file: {}", e))?;
        let mut lines = content.lines();

        // See if the header is correct
        let header = lines.next().ok_or("Error reading database header".to_string())?;
        if header != DB_HEADER {
            return Err(format!("Unsupported database version: {}", header));
        }

        // Read each line
        self.entries = lines
            .map(|line| line.parse::<Entry>())
            .collect::<Result<Vec<Entry>, _>>()
            .map_err(|e| format!("Failed to parse entry: {}", e))?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        let mut content = format!("{}\n", DB_HEADER);

        for entry in &self.entries {
            content.push_str(&format!("{}\n", entry.to_string()));
        }

        std::fs::write(&self.filename, &content)
            .map_err(|e| format!("Failed to write to database file: {}", e))?;

        Ok(())
    }

    pub fn insert(&mut self, config: &Config, time_mean: f64, time_stddev: f64) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get system time: {}", e))?
            .as_secs();

        let new_entry = Entry {
            timestamp,
            executable: config.executable.clone(),
            arguments: config.arguments.join(" ").to_string(),
            note: config.note.clone(),
            runs: config.runs,
            time_mean,
            time_stddev ,
        };

        self.entries.push(new_entry);
        Ok(())
    }

    pub fn search(&self, cfg: &Config) -> Vec<&Entry> {
        let arguments = cfg.arguments.join(" ").to_string();
        self.entries.iter().rev() // rev() so we have them in the order received
            .filter(|entry| match cfg.filter {
                FilterMode::All => true,
                FilterMode::Exe => entry.executable == cfg.executable,
                FilterMode::Exact => entry.executable == cfg.executable && entry.arguments == arguments,
            })
            .take(cfg.show)
            .collect()
    }
}
