use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: u64,
    node: u32,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn read_graph(path: &Path) -> Vec<Vec<(u32, u32)>> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    
    let mut lines = reader.lines();
    let first_line = lines.next().unwrap().unwrap();
    let mut parts = first_line.split_whitespace();
    let n: usize = parts.next().unwrap().parse().unwrap();
    let _m: usize = parts.next().unwrap().parse().unwrap();
    
    let mut adj: Vec<Vec<(u32, u32)>> = vec![Vec::new(); n];
    
    for line in lines {
        let line = line.unwrap();
        let mut parts = line.split_whitespace();
        let u: u32 = parts.next().unwrap().parse().unwrap();
        let v: u32 = parts.next().unwrap().parse().unwrap();
        let w: u32 = parts.next().unwrap().parse().unwrap();
        adj[u as usize].push((v, w));
    }
    
    adj
}

fn read_queries(path: &Path) -> Vec<(u32, u32)> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    
    reader.lines()
        .map(|line| {
            let line = line.unwrap();
            let mut parts = line.split_whitespace();
            let s: u32 = parts.next().unwrap().parse().unwrap();
            let t: u32 = parts.next().unwrap().parse().unwrap();
            (s, t)
        })
        .collect()
}

fn dijkstra(adj: &[Vec<(u32, u32)>], start: u32, target: u32) -> i64 {
    let n = adj.len();
    let mut dist: HashMap<u32, u64> = HashMap::new();
    let mut heap = BinaryHeap::new();
    
    dist.insert(start, 0);
    heap.push(State { cost: 0, node: start });
    
    while let Some(State { cost, node }) = heap.pop() {
        if let Some(&d) = dist.get(&node) {
            if cost > d {
                continue;
            }
        }
        
        if node == target {
            return cost as i64;
        }
        
        for &(next, weight) in &adj[node as usize] {
            let new_cost = cost.saturating_add(weight as u64);
            let entry = dist.entry(next).or_insert(u64::MAX);
            
            if new_cost < *entry {
                *entry = new_cost;
                heap.push(State { cost: new_cost, node: next });
            }
        }
    }
    
    -1
}

fn main() {
    let data_dir = std::env::var("DATA").unwrap_or_else(|_| "/data".to_string());
    let out_dir = std::env::var("OUT").unwrap_or_else(|_| "/out".to_string());
    
    let graph_path = Path::new(&data_dir).join("graph.txt");
    let queries_path = Path::new(&data_dir).join("queries.txt");
    let output_path = Path::new(&out_dir).join("distances.txt");
    
    let adj = read_graph(&graph_path);
    let queries = read_queries(&queries_path);
    
    let answers: Vec<i64> = queries
        .iter()
        .map(|&(s, t)| dijkstra(&adj, s, t))
        .collect();
    
    std::fs::create_dir_all(&out_dir).unwrap();
    let mut output = File::create(&output_path).unwrap();
    
    for answer in answers {
        writeln!(output, "{}", answer).unwrap();
    }
}
