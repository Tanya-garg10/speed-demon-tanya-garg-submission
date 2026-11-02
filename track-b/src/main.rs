use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use hashbrown::HashMap;

#[inline]
fn is_alpha_num(b: u8) -> bool {
    (b'a'..=b'z').contains(&b) || (b'0'..=b'9').contains(&b)
}

#[inline]
fn to_lower(b: u8) -> u8 {
    if (b'A'..=b'Z').contains(&b) {
        b + 32
    } else {
        b
    }
}

fn tokenize(text: &[u8]) -> Vec<&[u8]> {
    let mut tokens = Vec::new();
    let mut start = None;
    
    for (i, &b) in text.iter().enumerate() {
        let lower = to_lower(b);
        if is_alpha_num(lower) {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start {
            tokens.push(&text[s..i]);
            start = None;
        }
    }
    
    if let Some(s) = start {
        tokens.push(&text[s..]);
    }
    
    tokens
}

struct Index {
    postings: HashMap<Vec<u8>, Vec<u32>>,
    texts: Vec<Vec<u8>>,
}

impl Index {
    fn build(docs_path: &Path) -> Self {
        let file = File::open(docs_path).unwrap();
        let reader = BufReader::new(file);
        
        let mut postings: HashMap<Vec<u8>, Vec<u32>> = HashMap::new();
        let mut texts = Vec::new();
        
        for (doc_id, line) in reader.lines().enumerate() {
            let line = line.unwrap();
            let tab_pos = line.find('\t').unwrap();
            let text = &line.as_bytes()[tab_pos + 1..];
            texts.push(text.to_vec());
            
            let tokens: BTreeSet<Vec<u8>> = tokenize(text)
                .iter()
                .map(|&t| t.to_vec())
                .collect();
            
            for token in tokens {
                postings.entry(token).or_default().push(doc_id as u32);
            }
        }
        
        Index { postings, texts }
    }
    
    fn get(&self, term: &[u8]) -> &[u32] {
        self.postings.get(term).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

fn intersect_sorted(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            result.push(a[i]);
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else {
            j += 1;
        }
    }
    
    result
}

fn union_sorted(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            result.push(a[i]);
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            result.push(a[i]);
            i += 1;
        } else {
            result.push(b[j]);
            j += 1;
        }
    }
    
    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    
    result
}

fn difference_sorted(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            result.push(a[i]);
            i += 1;
        } else {
            j += 1;
        }
    }
    
    result.extend_from_slice(&a[i..]);
    
    result
}

fn bool_query(index: &Index, words: &[&[u8]]) -> Vec<u32> {
    let mut result: Vec<u32> = index.get(words[0]).to_vec();
    
    let mut i = 1;
    while i < words.len() {
        let op = words[i];
        let term = words[i + 1];
        let postings = index.get(term);
        
        result = match op {
            b"AND" => intersect_sorted(&result, postings),
            b"OR" => union_sorted(&result, postings),
            b"NOT" => difference_sorted(&result, postings),
            _ => panic!("Unknown operator"),
        };
        
        i += 2;
    }
    
    result
}

fn phrase_query(index: &Index, terms: &[&[u8]]) -> Vec<u32> {
    let mut candidates: Vec<u32> = index.get(terms[0]).to_vec();
    
    for term in &terms[1..] {
        candidates = intersect_sorted(&candidates, index.get(term));
    }
    
    let k = terms.len();
    let mut matches = Vec::new();
    
    for &doc_id in &candidates {
        let doc_id = doc_id as usize;
        let tokens = tokenize(&index.texts[doc_id]);
        
        for i in 0..=(tokens.len().saturating_sub(k)) {
            if tokens[i..i + k] == *terms {
                matches.push(doc_id as u32);
                break;
            }
        }
    }
    
    matches
}

fn format_result(doc_ids: &[u32]) -> String {
    let mut sorted = doc_ids.to_vec();
    sorted.sort();
    sorted.dedup();
    
    let count = sorted.len();
    let sum: u64 = sorted.iter().map(|&x| x as u64).sum();
    
    let mut result = format!("{} {}", count, sum);
    for id in sorted.iter().take(10) {
        result.push(' ');
        result.push_str(&id.to_string());
    }
    
    result
}

fn main() {
    let data_dir = std::env::var("DATA").unwrap_or_else(|_| "/data".to_string());
    let out_dir = std::env::var("OUT").unwrap_or_else(|_| "/out".to_string());
    
    let docs_path = Path::new(&data_dir).join("docs.txt");
    let queries_path = Path::new(&data_dir).join("queries.txt");
    let results_path = Path::new(&out_dir).join("results.txt");
    
    let index = Index::build(&docs_path);
    
    let queries_file = File::open(&queries_path).unwrap();
    let queries_reader = BufReader::new(queries_file);
    
    let mut results = Vec::new();
    
    for line in queries_reader.lines() {
        let line = line.unwrap();
        let bytes = line.as_bytes();
        
        if bytes.starts_with(b"BOOL ") {
            let words: Vec<&[u8]> = bytes[5..].split(|&b| b == b' ').collect();
            let doc_ids = bool_query(&index, &words);
            results.push(format_result(&doc_ids));
        } else if bytes.starts_with(b"PHRASE ") {
            let inner = &bytes[7..bytes.len() - 1];
            let terms: Vec<&[u8]> = inner.split(|&b| b == b' ').collect();
            let doc_ids = phrase_query(&index, &terms);
            results.push(format_result(&doc_ids));
        }
    }
    
    std::fs::create_dir_all(&out_dir).unwrap();
    let mut output = File::create(&results_path).unwrap();
    for result in results {
        writeln!(output, "{}", result).unwrap();
    }
}
