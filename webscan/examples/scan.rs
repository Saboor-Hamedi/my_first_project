use webscan::{scan, ScanOptions};

fn main() {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://example.com".to_string());

    println!("Starting scan for: {}", url);
    let opts = ScanOptions {
        full: true,
        probe_forms: true,
        delay_ms: 100,
        timeout_secs: 10,
        note: Some("CLI example test".into()),
    };

    match scan(&url, &opts) {
        Ok(res) => {
            println!("Status Code: {}", res.status_code);
            println!("Response Time: {}ms", res.response_time_ms);
            println!("Page Size: {} bytes", res.page_size_bytes);
            if let Some(srv) = res.server_header {
                println!("Server: {}", srv);
            }
            println!("\nFindings ({}):", res.findings.len());
            for f in &res.findings {
                println!("[{:?}] [{}] {}", f.category, f.severity.as_str(), f.title);
            }
        }
        Err(e) => {
            eprintln!("Scan error: {:#}", e);
        }
    }
}
