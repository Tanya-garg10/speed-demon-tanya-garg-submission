use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const INPUT_NAME: &str = "requests.log";
const OUTPUT_NAME: &str = "summary.txt";

type Key = (String, String); // (hour_string, endpoint)

fn percentile(sorted_values: &[u32], p: u32) -> u32 {
    let n = sorted_values.len();
    // ceil(p * n / 100) in integer arithmetic: (p * n + 99) / 100
    let rank = (p as usize * n + 99) / 100;
    sorted_values[rank - 1]
}

fn parse_timestamp_to_hour(ts: &str) -> String {
    // Input format: YYYY-MM-DDTHH:MM:SS.ffffffZ (27 bytes fixed)
    // Truncate to hour: keep date and hour part, set minutes/seconds/microseconds to 0
    // Example: "2026-10-30T18:59:59.999999Z" -> "2026-10-30T18:00:00Z"
    
    // Parse year, month, day, hour from the timestamp
    // Format: YYYY-MM-DDTHH:MM:SS.ffffffZ
    let year: i32 = ts[0..4].parse().unwrap();
    let month: u32 = ts[5..7].parse().unwrap();
    let day: u32 = ts[8..10].parse().unwrap();
    let hour: u32 = ts[11..13].parse().unwrap();
    
    // Format back as YYYY-MM-DDTHH:00:00Z
    format!("{:04}-{:02}-{:02}T{:02}:00:00Z", year, month, day, hour)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments similar to Python reference
    let args: Vec<String> = std::env::args().collect();
    
    let mut data_dir = "/data";
    let mut out_dir = "/out";
    
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--data" && i + 1 < args.len() {
            data_dir = &args[i + 1];
            i += 2;
        } else if args[i] == "--out" && i + 1 < args.len() {
            out_dir = &args[i + 1];
            i += 2;
        } else {
            i += 1;
        }
    }

    let input_path = Path::new(data_dir).join(INPUT_NAME);
    let output_path = Path::new(out_dir).join(OUTPUT_NAME);

    // Data structures
    let mut latencies: HashMap<Key, Vec<u32>> = HashMap::new();
    let mut errors: HashMap<Key, u32> = HashMap::new();

    // Read and process file
    let file = File::open(&input_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() != 5 {
            continue; // Skip malformed lines (shouldn't happen with valid input)
        }

        let ts = parts[0];
        let endpoint = parts[2];
        let status = parts[3];
        let latency_str = parts[4];

        // Parse hour from timestamp
        let hour = parse_timestamp_to_hour(ts);
        
        // Parse latency
        let latency: u32 = latency_str.parse()?;
        
        // Parse status
        let status_code: u32 = status.parse()?;
        
        let key = (hour, endpoint.to_string());
        
        // Update latencies
        latencies.entry(key.clone()).or_insert_with(Vec::new).push(latency);
        
        // Count errors (500-599)
        if status_code >= 500 && status_code <= 599 {
            *errors.entry(key).or_insert(0) += 1;
        }
    }

    // Prepare output
    let mut output_lines = Vec::new();
    
    // Collect and sort keys: first by hour, then by endpoint (bytewise order)
    let mut keys: Vec<Key> = latencies.keys().cloned().collect();
    keys.sort(); // Rust's default sort works for (String, String) tuple
    
    for key in keys {
        let (hour, endpoint) = &key;
        let mut latency_vec = latencies.remove(&key).unwrap();
        latency_vec.sort_unstable();
        
        let count = latency_vec.len() as u32;
        let errs = errors.get(&key).copied().unwrap_or(0);
        let ppm = errs * 1_000_000 / count;
        
        let p50 = percentile(&latency_vec, 50);
        let p95 = percentile(&latency_vec, 95);
        let p99 = percentile(&latency_vec, 99);
        
        output_lines.push(format!(
            "{} {} {} {} {} {} {} {}\n",
            hour, endpoint, count, p50, p95, p99, errs, ppm
        ));
    }

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(out_dir)?;
    
    // Write output file
    let mut output_file = File::create(&output_path)?;
    for line in output_lines {
        output_file.write_all(line.as_bytes())?;
    }

    Ok(())
}