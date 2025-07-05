use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::fs::{File};

use crate::*;

pub fn write_output(output: &str, measurements: Vec<&Entry>) -> Result<(), String> {
    if measurements.is_empty() {
        eprintln!("Nothing to output...");
        return Ok(());
    }

    let path = Path::new(output);
    let stem = path.file_stem().and_then(OsStr::to_str).unwrap_or("");
    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");

    let terminal = stem == "" || stem == "stdout";

    let writer = if terminal {
        Box::new(io::stdout()) as Box<dyn Write>
    } else {
        let file = File::create(&path).map_err(|e| format!("Unable to create output file: {:?}", e))?;
        Box::new(file) as Box<dyn Write>
    };

    match ext {
        "" | "txt" | "table" => output_table(writer, terminal, measurements),
        "csv" => output_csv(writer, measurements),
        "json" => output_json(writer, measurements),
        "xml" => output_xml(writer, measurements),
        _ => return Err(format!("Unknown output format for {}", output)),
    }.map_err(|_| format!("write output failed"))
}

fn output_table(mut wr: Box<dyn Write>, color: bool, measurements: Vec<&Entry>) -> Result<(), io::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save yourself, end of time is here") // no point recovering from this :(
        .as_secs();

    // figure out what data we want to print, starting with the headers
    let mut columns: Vec<Vec<String>> = vec![
        vec!["".to_string()], // New column for color
        vec!["Age".to_string()],
        vec!["Executable".to_string()],
        vec!["Arguments".to_string()],
        vec!["Runs".to_string()],
        vec!["Mean (s)".to_string()],
        vec!["StdDev (s)".to_string()],
        vec!["Change (%)".to_string()],
        vec!["Note".to_string()],

    ];

    // set color for header and anything over 1% better or worse
    let mut first_mean = measurements.first().map(|entry| entry.time_mean).unwrap();
    if first_mean <= 0.0 {
        first_mean = 0.00001f64; // avoid divide by zero
    }


    // use colors only if requested
    let (red, green, bold, reset) = if color {
        ("\x1B[31m", "\x1B[32m", "\x1B[1m", "\x1B[0m")
    } else {
        ("", "", "", "")
    };

    columns[0].extend(measurements.iter().enumerate().map(|(i, entry)| {
        let modifier = if i == 0 {
            bold
        } else if entry.time_mean * 1.01 < first_mean {
            red
        } else if entry.time_mean > first_mean * 1.01 {
            green
        } else {
            ""
        };
        modifier.to_string()
    }));

    columns[1].extend(measurements.iter().map(|entry| entry.age(now)));
    columns[2].extend(measurements.iter().map(|entry| entry.executable.clone()));
    columns[3].extend(measurements.iter().map(|entry| entry.arguments.clone()));
    columns[4].extend(measurements.iter().map(|entry| entry.runs.to_string()));
    columns[5].extend(measurements.iter().map(|entry| format!("{:.4}", entry.time_mean)));
    columns[6].extend(measurements.iter().map(|entry| format!("{:.4}", entry.time_stddev)));
    columns[7].extend(measurements.iter().enumerate().map(|(i, entry)| {
        if i == 0 {
            " ".to_string() // Empty string for the first entry
        } else {
            format!("{:.2}", ((entry.time_mean - first_mean) / first_mean) * 100.0)
        }
    }));
    columns[8].extend(measurements.iter().map(|entry| entry.note.to_string()));

    // to print a nice table we will need to know max width for each column
    let widths: Vec<usize> = columns.iter()
        .map(|col| col.iter().map(|s| s.len()).max().unwrap_or(0) + 2 )
        .collect();

    // lets print the table now
    for i in 0..columns[0].len() {
        let color_prefix = &columns[0][i];
        let row_strings: Vec<String> = columns.iter().skip(1).zip(&widths.iter().skip(1).collect::<Vec<_>>()).enumerate()
            .map(|(_j, (col, w))| format!("{:^width$}", col[i], width = w) )
            .collect();

        write!(wr, "{}{}{}\n", color_prefix, row_strings.join("|"), reset)?;

        // line separator between header and data
        if i == 0 {
            write!(wr, "{}\n", widths.iter().skip(1).map(|w| "-".repeat(*w)).collect::<Vec<_>>().join("+") )?;
        }
    }
    Ok(())
}

// naive CSV escape string
fn escape_csv(s : &String) -> String {
    s.replace("\"", "\"\"")
}

fn output_csv(mut wr: Box<dyn Write>, measurements: Vec<&Entry>) -> Result<(), io::Error> {
    write!(wr,"Timestamp,Executable,Arguments,Runs,Mean,StdDev,Note\n")?;

    for m in measurements  {
        write!(wr, "{},\"{}\",\"{}\",{},{},{},\"{}\"\n",
               m.timestamp, escape_csv(&m.executable), escape_csv(&m.arguments),
               m.runs, m.time_mean, m.time_stddev, escape_csv(&m.note))?;
    }
    Ok(())
}

// naive JSON escape string attempt...
fn escape_json(s : &String) -> String {
    s.replace("\"", "\\\"")
}

fn output_json(mut wr: Box<dyn Write>, measurements: Vec<&Entry>) -> Result<(), io::Error> {
    write!(wr, "[\n")?;

    for (i, m) in measurements.iter().enumerate() {
        if i != 0 {
             write!(wr, ",\n")?;
        }
        write!(wr, " {{\"timestamp\": {}, \"executable\": \"{}\", \"arguments\": \"{}\",  \
                    \"runs\": {}, \"mean\": {:3.3}, \"stddev\": {:3.3}, \"{}\"}}",
               m.timestamp, escape_json(&m.executable), escape_json(&m.arguments),
               m.runs, m.time_mean, m.time_stddev, escape_json(&m.note) )?;
    }

    write!(wr, "\n]\n")?;
    Ok(())
}

// my naive XML escape attempt...
fn escape_xml(s : &String) -> String {
    s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
}

fn output_xml(mut wr: Box<dyn Write>, measurements: Vec<&Entry>) -> Result<(), io::Error> {
    write!(wr, "<Measurements>\n")?;

    for m in measurements {
        write!(wr, "  <Measurement>\n")?;
        write!(wr, "    <Timestamp>{}</Timestamp>\n", m.timestamp)?;
        write!(wr, "    <Executable>{}</Executable>\n", escape_xml(&m.executable))?;
        write!(wr, "    <Arguments>{}</Arguments>\n", escape_xml(&m.arguments))?;
        write!(wr, "    <Note>{}</Note>\n", escape_xml(&m.note))?;
        write!(wr, "    <Runs>{}</Runs>\n", m.runs)?;
        write!(wr, "    <Mean>{:3.3}</Mean>\n", m.time_mean)?;
        write!(wr, "    <StdDev>{:3.3}</StdDev>\n", m.time_stddev)?;
        write!(wr, "  </Measurement>\n")?;
    }

    write!(wr, "</Measurements>\n")?;
    Ok(())
}
