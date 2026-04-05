// show progress on stdout
use std::io::{self, Write};
use std::time::Instant;

pub struct Progress {
    total: usize,
    current: usize,
    quiet: bool,
    label: String,
    start_time: Instant,
    mean: f64,
    m2: f64,
    prev_mean: Option<f64>,
    prev_std: Option<f64>,
    last_line_len: usize,
}

impl Progress {
    pub fn new(total: usize, label: &str, quiet: bool, prev_mean: Option<f64>, prev_std: Option<f64>) -> Self {
        Progress {
            total, current: 0, quiet, label: label.to_string(),
            start_time: Instant::now(), mean: 0.0, m2: 0.0,
            prev_mean, prev_std, last_line_len: 0,
        }
    }

    pub fn start(&mut self) {
        if !self.quiet {
            self.print(&self.render());
         }
    }

    pub fn stop(&mut self, time: f64) {
        if !self.quiet {
            self.current += 1;
            self.update_stats(time);
            self.print(&self.render());
        }
    }

    fn update_stats(&mut self, time: f64) {
        let delta = time - self.mean;
        self.mean += delta / self.current as f64;
        let delta2 = time - self.mean;
        self.m2 += delta * delta2;
    }

    fn render(&self) -> String {
        let pct = if self.total > 0 { (self.current as f64 / self.total as f64) * 100.0 } else { 0.0 };
        let filled = (pct / 100.0 * 30.0) as usize;
        let empty = 30 - filled;
        let counter = format!("[{:2}/{:2}]", self.current, self.total);

        let stats = if self.current > 0 {
            let std = if self.current > 1 { (self.m2 / self.current as f64).sqrt() } else { 0.0 };
            let base = format!(", \u{03bc}={:.3} \u{03c3}={:.3}", self.mean, std);

            if self.current < self.total {
                // compute ETA
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let eta_secs = elapsed / self.current as f64 * (self.total - self.current) as f64;
                format!("{} ETA {}", base, format_eta(eta_secs))
            } else {
                // Last line doesn't need ETA, but we want to clear the previous ETA text
                base
            }
        } else if let (Some(m), Some(s)) = (self.prev_mean, self.prev_std) {
            format!(", \u{03bc}={:.3} \u{03c3}={:.3}", m, s)
        } else {
            String::new()
        };

        format!("\r{}  {}{} {}{}", counter, "\u{2588}".repeat(filled), "\u{2591}".repeat(empty), self.label, stats)
    }

    fn print(&mut self, line: &str) {
        let padding = if line.len() < self.last_line_len {
            self.last_line_len - line.len()
        } else {
            0
        };
        self.last_line_len = line.len();
        let _ = write!(io::stderr(), "{}{}", line, " ".repeat(padding));
        let _ = io::stderr().flush();
    }

    pub fn finish(&mut self, cleanup: bool) {
        if ! self.quiet {
            if cleanup {
                let _ = write!(io::stderr(), "\r{}\r", " ".repeat(self.last_line_len));
            } else {
                let _ = writeln!(io::stderr());
            }
            let _ = io::stderr().flush();
        }
    }
}

fn format_eta(secs: f64) -> String {
    let h = secs as u64 / 3600;
    let m = (secs as u64 % 3600) / 60;
    let s = secs as u64 % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
