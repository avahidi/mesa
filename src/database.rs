use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const DB_HEADER: &str = "mesa database|version=1";

#[derive(Debug)]
pub struct Entry {
    pub timestamp: u64,
    pub executable: String,
    pub arguments: Vec<String>,
    pub runs: u32,
    pub time_mean: f64,
    pub time_stddev: f64,
}

impl FromStr for Entry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 6 {
            return Err(format!("Invalid entry format: {}", s));
        }

        Ok(Entry {
            timestamp: parts[0].parse().map_err(|e| format!("Invalid timestamp: {}", e))?,
            executable: parts[1].to_string(),
            arguments: parts[2].split(' ').map(|s| s.to_string()).collect(),
            time_mean: parts[3].parse().map_err(|e| format!("Invalid execution time: {}", e))?,
            runs: parts[4].parse().map_err(|e| format!("Invalid run count: {}", e))?,
            time_stddev: parts[5].parse().map_err(|e| format!("Invalid std dev: {}", e))?,
        })
    }
}

impl ToString for Entry {
    fn to_string(&self) -> String {
        format!("{}|{}|{}|{}|{}|{}",
            self.timestamp, self.executable, self.arguments.join(" "), self.time_mean, self.runs, self.time_stddev
        )
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

    pub fn insert(&mut self, executable: &str, arguments: &[String], time_mean: f64) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get system time: {}", e))?
            .as_secs();

        let new_entry = Entry {
            timestamp,
            executable: executable.to_string(),
            arguments: arguments.to_vec(),
            time_mean,
            runs: 1,
            time_stddev: 0.0,
        };

        self.entries.push(new_entry);
        Ok(())
    }
}
