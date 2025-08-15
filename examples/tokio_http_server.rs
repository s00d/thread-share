use std::time::{Duration, Instant};
use std::collections::HashMap;
use thread_share::{enhanced_share, spawn_workers};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// HTML helper functions for generating simple pages
fn render_page(data: &HashMap<String, String>) -> String {
    let mut html = String::new();
    for (key, value) in data {
        html.push_str(&format!("<p><strong>{}:</strong> {}</p>", key, value));
    }
    
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Server</title>
</head>
<body>
{}
</body>
</html>"#,
        html
    )
}

/// Async HTTP server using tokio and ThreadShare
/// 
/// This example demonstrates how to use ThreadShare with tokio for
/// building high-performance async HTTP servers with shared state.
#[tokio::main]
async fn main() {
    println!("=== Tokio HTTP Server with ThreadShare ===");
    println!("Building async HTTP server with shared state...\n");

    // Create shared server state
    let server = enhanced_share!(AsyncHttpServer {
        port: 8081,
        is_running: true,
        requests_handled: 0,
        active_connections: 0,
        start_time: Instant::now(),
    });

    // Create shared visit counter
    let visits = enhanced_share!(0);

    println!("🚀 Starting Tokio HTTP server on port 8081...");
    println!("📱 Server URLs:");
    println!("   • Main page: http://127.0.0.1:8081/");
    println!("   • Status: http://127.0.0.1:8081/status");
    println!("   • Health: http://127.0.0.1:8081/health");
    println!("   • Metrics: http://127.0.0.1:8081/metrics");
    println!();

    // Start all async workers with spawn_workers!
    let visits_clone = visits.clone();
    let manager = spawn_workers!(server, {
        server_main: move |server: thread_share::ThreadShare<AsyncHttpServer>| {
            println!("🌐 Tokio server main worker started");
            
            // Create runtime for this worker
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            rt.block_on(async {
                // Start the server
                server.update(|s| s.start().expect("Failed to start server"));
                
                // Create TCP listener
                let port = server.get().port;
                let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
                    .await
                    .expect("Failed to bind");
                
                println!("🔌 Listening on http://127.0.0.1:{}", port);
                
                // Accept connections
                loop {
                    match listener.accept().await {
                        Ok((stream, _addr)) => {
                            // Increment connection counter
                            server.update(|s| s.increment_connections());
                            
                            // Spawn task to handle connection
                            let server_clone = server.clone();
                            let visits_clone = visits_clone.clone();
                            
                            tokio::spawn(async move {
                                handle_connection(stream, server_clone, visits_clone).await;
                            });
                        }
                        Err(e) => {
                            eprintln!("❌ Connection failed: {}", e);
                        }
                    }
                    
                    // Check if server should stop
                    if !server.get().is_running {
                        break;
                    }
                }
                
                println!("🌐 Tokio server main worker finished");
            });
        }
    });

    // Demonstrate worker management
    println!("🔧 Worker Manager Demo:");
    println!("📋 Worker names: {:?}", manager.get_worker_names());
    println!("🔢 Active workers: {}", manager.active_workers());

    // Add stats monitor worker programmatically
    println!("\n➕ Adding stats monitor worker programmatically...");
    let server_clone = server.clone();
    let stats_handle = std::thread::spawn(move || {
        println!("📊 Stats monitor worker started");
        
        // Create runtime for this worker
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            // Monitor server statistics every 3 seconds
            for _i in 1..=20 { // 20 iterations * 3 seconds = 1 minute
                let current_server = server_clone.get();
                
                if current_server.is_running {
                    println!("📊 Tokio Server Stats | Port: {} | Requests: {} | Connections: {} | Uptime: {}", 
                        current_server.port,
                        current_server.requests_handled,
                        current_server.active_connections,
                        current_server.get_uptime()
                    );
                } else {
                    println!("📊 Server stopped, stats monitor exiting");
                    break;
                }
                
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
            
            // Stop server after 1 minute by stopping the main worker
            println!("⏰ Stopping tokio server after 1 minute...");
            server_clone.update(|s| s.stop());
            println!("📊 Stats monitor worker finished");
        });
    });

    // Add the stats monitor to the manager
    if let Err(e) = manager.add_worker("stats_monitor", stats_handle) {
        println!("❌ Failed to add stats monitor: {}", e);
    } else {
        println!("✅ Stats monitor worker added successfully");
    }

    println!("📋 Updated worker names: {:?}", manager.get_worker_names());
    println!("🔢 Updated active workers: {}", manager.active_workers());

    // Wait for all workers to complete
    manager.join_all().expect("Failed to join tokio workers");
    
    println!("✅ Tokio HTTP server completed successfully!");
}

/// Handle individual TCP connection
async fn handle_connection(
    mut stream: TcpStream,
    server: thread_share::ThreadShare<AsyncHttpServer>,
    visits: thread_share::EnhancedThreadShare<u32>,
) {
    let mut buffer = [0; 1024];
    
    // Read request
    let n = match stream.read(&mut buffer).await {
        Ok(n) if n == 0 => return, // Connection closed
        Ok(n) => n,
        Err(e) => {
            eprintln!("❌ Error reading from stream: {}", e);
            return;
        }
    };
    
    let request = String::from_utf8_lossy(&buffer[..n]);
    let request_lines: Vec<&str> = request.lines().collect();
    
    if request_lines.is_empty() {
        return;
    }
    
    let request_line = request_lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return;
    }
    
    let method = parts[0];
    let path = parts[1];
    
    // Increment request counter
    server.update(|s| s.increment_requests());
    
    // Handle different routes
    let (status_line, content, content_type) = match (method, path) {
        ("GET", "/") => {
            // Increment visit counter for main page
            visits.update(|v| *v += 1);
            let visit_count = visits.get();
            
            (
                "HTTP/1.1 200 OK",
                render_page(&{
                    let mut map = HashMap::new();
                    map.insert("Page Visits".to_string(), visit_count.to_string());
                    map.insert("Total Requests".to_string(), server.get().requests_handled.to_string());
                    map.insert("Active Connections".to_string(), server.get().active_connections.to_string());
                    map.insert("Uptime".to_string(), server.get().get_uptime());
                    map
                }),
                "text/html; charset=utf-8"
            )
        }
        
        ("GET", "/status") => {
            let server_data = server.get();
            (
                "HTTP/1.1 200 OK",
                render_page(&{
                    let mut map = HashMap::new();
                    map.insert("Status".to_string(), if server_data.is_running { "Running".to_string() } else { "Stopped".to_string() });
                    map.insert("Port".to_string(), server_data.port.to_string());
                    map.insert("Uptime".to_string(), server_data.get_uptime());
                    map.insert("Requests".to_string(), server_data.requests_handled.to_string());
                    map.insert("Connections".to_string(), server_data.active_connections.to_string());
                    map
                }),
                "text/html; charset=utf-8"
            )
        }
        
        ("GET", "/health") => {
            (
                "HTTP/1.1 200 OK",
                render_page(&{
                    let mut map = HashMap::new();
                    map.insert("Health".to_string(), "OK".to_string());
                    map
                }),
                "text/html; charset=utf-8"
            )
        }
        
        ("GET", "/metrics") => {
            let server_data = server.get();
            let visit_count = visits.get();
            
            (
                "HTTP/1.1 200 OK",
                render_page(&{
                    let mut map = HashMap::new();
                    map.insert("Requests".to_string(), server_data.requests_handled.to_string());
                    map.insert("Connections".to_string(), server_data.active_connections.to_string());
                    map.insert("Page Visits".to_string(), visit_count.to_string());
                    map.insert("Uptime".to_string(), format!("{}s", server_data.start_time.elapsed().as_secs()));
                    map
                }),
                "text/html; charset=utf-8"
            )
        }
        
        _ => {
            (
                "HTTP/1.1 404 Not Found",
                render_page(&{
                    let mut map = HashMap::new();
                    map.insert("Error".to_string(), "404 - Page not found".to_string());
                    map
                }),
                "text/html; charset=utf-8"
            )
        }
    };
    
    // Build response
    let response = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content_type,
        content.len(),
        content
    );
    
    // Write response
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        eprintln!("❌ Error writing to stream: {}", e);
    }
    
    // Decrement connection counter
    server.update(|s| s.decrement_connections());
}

/// Async HTTP server structure
#[derive(Clone, Debug)]
struct AsyncHttpServer {
    port: u16,
    is_running: bool,
    requests_handled: u64,
    active_connections: u32,
    start_time: Instant,
}

impl AsyncHttpServer {
    fn start(&mut self) -> Result<(), String> {
        self.is_running = true;
        self.start_time = Instant::now();
        Ok(())
    }
    
    fn stop(&mut self) {
        self.is_running = false;
    }
    
    fn increment_requests(&mut self) {
        self.requests_handled += 1;
    }
    
    fn increment_connections(&mut self) {
        self.active_connections += 1;
    }
    
    fn decrement_connections(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }
    
    fn get_uptime(&self) -> String {
        let duration = self.start_time.elapsed();
        let seconds = duration.as_secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes % 60, seconds % 60)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds % 60)
        } else {
            format!("{}s", seconds)
        }
    }
}
