use std::process::Command;
use std::fs;

fn main() {
    println!("Building...");
    Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .expect("Failed to build");
    
    // Simple test
    let test_data = r#"2026-10-30T18:00:00.000000Z GET /test 200 100
2026-10-30T18:30:00.000000Z GET /test 200 200
2026-10-30T19:00:00.000000Z GET /test 200 300
"#;
    
    fs::create_dir_all("test-dir").expect("Failed to create dir");
    fs::write("test-dir/requests.log", test_data).expect("Failed to write");
    
    let output = Command::new("target/release/solution")
        .args(["--data", "test-dir", "--out", "test-out"])
        .output()
        .expect("Failed to run");
    
    if output.status.success() {
        let result = fs::read_to_string("test-out/summary.txt").unwrap();
        println!("Output:\n{}", result);
        
        // Check hour parsing
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2); // Two hours: 18:00 and 19:00
        assert!(lines[0].contains("2026-10-30T18:00:00Z"));
        assert!(lines[1].contains("2026-10-30T19:00:00Z"));
        println!("✅ Hour parsing correct!");
    } else {
        eprintln!("Failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Clean up
    let _ = fs::remove_dir_all("test-dir");
    let _ = fs::remove_dir_all("test-out");
}