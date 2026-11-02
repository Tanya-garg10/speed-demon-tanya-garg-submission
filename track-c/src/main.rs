use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use hashbrown::HashMap as HashMapB;

#[derive(Default)]
struct Country {
    count: u64,
    id_sum: u64,
    amount_sum: i64,
    amount_min: i64,
    amount_max: i64,
    names: HashMapB<Vec<u8>, ()>,
    name_bytes: u64,
    note_bytes: u64,
}

impl Country {
    fn add(&mut self, id: u64, name: &[u8], note: &[u8], amount_cents: i64) {
        self.count += 1;
        self.id_sum += id;
        self.amount_sum += amount_cents;
        
        if self.count == 1 {
            self.amount_min = amount_cents;
            self.amount_max = amount_cents;
        } else {
            if amount_cents < self.amount_min {
                self.amount_min = amount_cents;
            }
            if amount_cents > self.amount_max {
                self.amount_max = amount_cents;
            }
        }
        
        self.names.insert(name.to_vec(), ());
        self.name_bytes += name.len() as u64;
        self.note_bytes += note.len() as u64;
    }
    
    fn line(&self, key: &str) -> String {
        format!(
            "{} {} {} {} {} {} {} {}",
            key,
            self.count,
            self.id_sum,
            format_cents(self.amount_sum),
            format_cents(self.amount_min),
            format_cents(self.amount_max),
            self.names.len(),
            self.name_bytes,
            self.note_bytes
        )
    }
}

fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    format!("{}{}.{}", sign, abs / 100, abs % 100)
}

fn parse_cents(s: &[u8]) -> i64 {
    let s = std::str::from_utf8(s).unwrap();
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 2 {
        panic!("Invalid amount format");
    }
    let sign = if s.starts_with('-') { -1 } else { 1 };
    let whole: i64 = parts[0].parse().unwrap();
    let frac: i64 = parts[1].parse().unwrap();
    sign * (whole.abs() * 100 + frac)
}

struct CsvParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> CsvParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        CsvParser { data, pos: 0 }
    }
    
    fn skip_bom(&mut self) {
        if self.data.len() >= 3 && &self.data[0..3] == b"\xEF\xBB\xBF" {
            self.pos = 3;
        }
    }
    
    fn peek(&self) -> Option<u8> {
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }
    
    fn consume(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let c = self.data[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }
    
    fn read_field(&mut self) -> Vec<u8> {
        let mut result = Vec::new();
        
        if let Some(b'"') = self.peek() {
            self.consume();
            loop {
                match self.consume() {
                    Some(b'"') => {
                        if let Some(b'"') = self.peek() {
                            self.consume();
                            result.push(b'"');
                        } else {
                            break;
                        }
                    }
                    Some(c) => result.push(c),
                    None => break,
                }
            }
        } else {
            while let Some(c) = self.peek() {
                if c == b',' || c == b'\r' || c == b'\n' {
                    break;
                }
                result.push(c);
                self.consume();
            }
        }
        
        result
    }
    
    fn read_record(&mut self) -> Option<Vec<Vec<u8>>> {
        if self.pos >= self.data.len() {
            return None;
        }
        
        let mut fields = Vec::new();
        
        fields.push(self.read_field());
        
        while let Some(b',') = self.peek() {
            self.consume();
            fields.push(self.read_field());
        }
        
        while let Some(b'\r') = self.peek() {
            self.consume();
        }
        if let Some(b'\n') = self.peek() {
            self.consume();
        }
        
        Some(fields)
    }
}

fn load_regions(path: &Path) -> HashMap<u32, String> {
    let data = std::fs::read(path).unwrap();
    let mut parser = CsvParser::new(&data);
    parser.skip_bom();
    
    let header = parser.read_record().unwrap();
    assert_eq!(header, vec![b"region_id", b"country"]);
    
    let mut regions = HashMap::new();
    while let Some(record) = parser.read_record() {
        let region_id: u32 = std::str::from_utf8(&record[0]).unwrap().parse().unwrap();
        let country = std::str::from_utf8(&record[1]).unwrap().to_string();
        regions.insert(region_id, country);
    }
    
    regions
}

fn main() {
    let data_dir = std::env::var("DATA").unwrap_or_else(|_| "/data".to_string());
    let out_dir = std::env::var("OUT").unwrap_or_else(|_| "/out".to_string());
    
    let records_path = Path::new(&data_dir).join("records.csv");
    let regions_path = Path::new(&data_dir).join("regions.csv");
    let output_path = Path::new(&out_dir).join("aggregates.txt");
    
    let regions = load_regions(&regions_path);
    
    let data = std::fs::read(&records_path).unwrap();
    let mut parser = CsvParser::new(&data);
    parser.skip_bom();
    
    let header = parser.read_record().unwrap();
    assert_eq!(header, vec![b"id", b"name", b"note", b"region_id", b"amount"]);
    
    let mut countries: HashMap<String, Country> = HashMap::new();
    
    while let Some(record) = parser.read_record() {
        let id: u64 = std::str::from_utf8(&record[0]).unwrap().parse().unwrap();
        let name = &record[1];
        let note = &record[2];
        let region_id: u32 = std::str::from_utf8(&record[3]).unwrap().parse().unwrap();
        let amount_cents = parse_cents(&record[4]);
        
        let key = regions.get(&region_id).cloned().unwrap_or_else(|| "_UNMATCHED".to_string());
        
        countries.entry(key).or_default().add(id, name, note, amount_cents);
    }
    
    let mut keys: Vec<String> = countries.keys().cloned().collect();
    keys.sort();
    
    std::fs::create_dir_all(&out_dir).unwrap();
    let mut output = File::create(&output_path).unwrap();
    
    for key in keys {
        writeln!(output, "{}", countries[&key].line(&key)).unwrap();
    }
}
