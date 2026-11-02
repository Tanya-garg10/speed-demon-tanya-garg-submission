use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const INPUT_NAME: &str = "requests.log";
const OUTPUT_NAME: &str = "summary.txt";

// Hour strings are always 20 bytes: YYYY-MM-DDTHH:00:00Z
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HourString([u8; 20]);

impl HourString {
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}

// Simple string interning for endpoints
struct StringInterner {
    strings: Vec<Vec<u8>>,
    map: HashMap<Vec<u8>, usize>,
}

impl StringInterner {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            map: HashMap::new(),
        }
    }
    
    fn intern(&mut self, bytes: &[u8]) -> usize {
        if let Some(&id) = self.map.get(bytes) {
            return id;
        }
        let id = self.strings.len();
        self.strings.push(bytes.to_vec());
        self.map.insert(bytes.to_vec(), id);
        id
    }
    
    fn get(&self, id: usize) -> &[u8] {
        &self.strings[id]
    }
    
    fn get_str(&self, id: usize) -> &str {
        std::str::from_utf8(&self.strings[id]).unwrap()
    }
}

type Key = (HourString, usize); // (hour_string, endpoint_id)

fn percentile(sorted_values: &[u32], p: u32) -> u32 {
    let n = sorted_values.len();
    // ceil(p * n / 100) in integer arithmetic: (p * n + 99) / 100
    let rank = (p as usize * n + 99) / 100;
    sorted_values[rank - 1]
}



fn parse_u32_from_bytes(bytes: &[u8]) -> u32 {
    // Parse ASCII digits to u32 without allocation
    // Bytes are guaranteed to be ASCII digits (0-9) only
    let mut result = 0u32;
    for &byte in bytes {
        result = result * 10 + (byte - b'0') as u32;
    }
    result
}

fn parse_hour_from_timestamp_bytes(bytes: &[u8]) -> HourString {
    // Parse timestamp bytes directly (format: YYYY-MM-DDTHH:MM:SS.ffffffZ)
    // Extract hour and format as YYYY-MM-DDTHH:00:00Z (20 bytes)
    // We know timestamp is exactly 27 bytes: YYYY-MM-DDTHH:MM:SS.ffffffZ
    let mut hour_bytes = [0u8; 20];
    
    // Copy YYYY-MM-DDTHH: (first 13 bytes)
    hour_bytes[0..13].copy_from_slice(&bytes[0..13]);
    // Add ":00:00Z" (7 bytes)
    hour_bytes[13] = b':';
    hour_bytes[14] = b'0';
    hour_bytes[15] = b'0';
    hour_bytes[16] = b':';
    hour_bytes[17] = b'0';
    hour_bytes[18] = b'0';
    hour_bytes[19] = b'Z';
    
    HourString(hour_bytes)
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
    let mut interner = StringInterner::new();

    // Read entire file into memory
    let data = fs::read(&input_path)?;
    
    // Process file byte by byte
    let mut pos = 0;
    let len = data.len();
    
    while pos < len {
        // Find end of line
        let line_start = pos;
        while pos < len && data[pos] != b'\n' {
            pos += 1;
        }
        let line_end = pos;
        
        // Skip empty lines
        if line_start == line_end {
            pos += 1; // Skip the newline
            continue;
        }
        
        // Parse line directly from bytes
        let line = &data[line_start..line_end];
        
        // Parse fields in a single pass
        let mut field_starts = [0usize; 5];
        let mut field_idx = 0;
        field_starts[0] = 0;
        
        for (i, &byte) in line.iter().enumerate() {
            if byte == b' ' {
                field_idx += 1;
                if field_idx < 5 {
                    field_starts[field_idx] = i + 1;
                } else {
                    // More than 4 spaces - malformed line
                    break;
                }
            }
        }
        
        if field_idx != 4 {
            // Malformed line (shouldn't happen with valid input)
            pos += 1; // Skip the newline
            continue;
        }
        
        // Extract fields as byte slices
        let ts_bytes = &line[field_starts[0]..(field_starts[1] - 1)];
        let endpoint_bytes = &line[field_starts[2]..(field_starts[3] - 1)];
        let status_bytes = &line[field_starts[3]..(field_starts[4] - 1)];
        let latency_bytes = &line[field_starts[4]..];
        
        // Parse hour directly from timestamp bytes (format: YYYY-MM-DDTHH:MM:SS.ffffffZ)
        let hour_bytes = parse_hour_from_timestamp_bytes(ts_bytes);
        
        // Intern endpoint
        let endpoint_id = interner.intern(endpoint_bytes);
        
        // Parse status from bytes
        let status_code = parse_u32_from_bytes(status_bytes);
        
        // Parse latency from bytes
        let latency = parse_u32_from_bytes(latency_bytes);
        
        let key = (hour_bytes, endpoint_id);
        
        // Update latencies
        latencies.entry(key.clone()).or_insert_with(Vec::new).push(latency);
        
        // Count errors (500-599)
        if status_code >= 500 && status_code <= 599 {
            *errors.entry(key).or_insert(0) += 1;
        }
        
        pos += 1; // Skip the newline
    }

    // Prepare output
    let mut output_lines = Vec::new();
    
    // Collect and sort keys: first by hour, then by endpoint (bytewise order)
    let mut keys: Vec<Key> = latencies.keys().cloned().collect();
    keys.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| {
            let a_bytes = interner.get(a.1);
            let b_bytes = interner.get(b.1);
            a_bytes.cmp(b_bytes)
        })
    });
    
    for key in keys {
        let (hour_string, endpoint_id) = &key;
        let hour_str = hour_string.as_str();
        let endpoint_str = interner.get_str(*endpoint_id);
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
            hour_str, endpoint_str, count, p50, p95, p99, errs, ppm
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