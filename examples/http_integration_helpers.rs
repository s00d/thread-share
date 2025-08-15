use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
use thread_share::{enhanced_share, share};

// Простой HTTP сервер
#[derive(Clone, Debug)]
struct HttpServer {
    port: u16,
    is_running: bool,
    requests_handled: u32,
    start_time: Option<Instant>,
    active_connections: u32,
}

impl HttpServer {
    fn new(port: u16) -> Self {
        Self {
            port,
            is_running: false,
            requests_handled: 0,
            start_time: None,
            active_connections: 0,
        }
    }

    fn start(&mut self) -> Result<(), String> {
        self.is_running = true;
        self.start_time = Some(Instant::now());
        println!("🚀 HTTP Server started on port {}", self.port);
        Ok(())
    }

    fn stop(&mut self) {
        self.is_running = false;
        println!("🛑 HTTP Server stopped");
    }

    fn handle_request(&mut self, stream: &mut TcpStream) -> Result<bool, String> {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).map_err(|e| e.to_string())?;

        if bytes_read == 0 {
            return Ok(false);
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let lines: Vec<&str> = request.lines().collect();

        if lines.is_empty() {
            return Ok(false);
        }

        let first_line = lines[0];
        let parts: Vec<&str> = first_line.split_whitespace().collect();

        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];

            println!("📥 {} {}", method, path);

            // Обрабатываем запрос
            let response = self.process_request(method, path);

            // Отправляем ответ
            stream
                .write_all(response.as_bytes())
                .map_err(|e| e.to_string())?;
            stream.flush().map_err(|e| e.to_string())?;

            // Ждем немного перед закрытием соединения
            std::thread::sleep(Duration::from_millis(100));

            // Увеличиваем счетчик запросов только для основных страниц
            let is_main_page = matches!(path, "/" | "/status" | "/health");
            if is_main_page {
                self.requests_handled += 1;
            }

            // Возвращаем true если это основная страница (не статика)
            return Ok(is_main_page);
        }

        Ok(false)
    }

    fn process_request(&self, method: &str, path: &str) -> String {
        match (method, path) {
            ("GET", "/") => {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n\
                    <html><body><h1>Hello from Rust HTTP Server!</h1>\
                    <p>Server running on port {}</p>\
                    <p>Requests handled: {}</p>\
                    <p>Uptime: {}</p>\
                    </body></html>",
                    self.get_home_page_length(),
                    self.port,
                    self.requests_handled,
                    self.get_uptime()
                )
            }
            ("GET", "/status") => {
                let status = format!(
                    "{{\"status\": \"running\", \"port\": {}, \"requests\": {}, \"uptime\": \"{}\"}}",
                    self.port,
                    self.requests_handled,
                    self.get_uptime()
                );
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status.len(),
                    status
                )
            }
            ("GET", "/health") => {
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK".to_string()
            }
            _ => {
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 21\r\nConnection: close\r\n\r\n404 - Page not found".to_string()
            }
        }
    }

    fn get_uptime(&self) -> String {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            let secs = elapsed.as_secs();
            let mins = secs / 60;
            let secs = secs % 60;
            format!("{}m {}s", mins, secs)
        } else {
            "0s".to_string()
        }
    }

    fn increment_connections(&mut self) {
        self.active_connections += 1;
    }

    fn decrement_connections(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    fn get_home_page_length(&self) -> usize {
        let content = format!(
            "<html><body><h1>Hello from Rust HTTP Server!</h1>\
            <p>Server running on port {}</p>\
            <p>Requests handled: {}</p>\
            <p>Uptime: {}</p>\
            </body></html>",
            self.port,
            self.requests_handled,
            self.get_uptime()
        );
        content.len()
    }
}

fn main() {
    println!("=== Simple HTTP Server with ThreadShare ===");

    // Создаем HTTP сервер на порту 8445
    let port = 8445;
    println!("🔍 Using port: {}", port);

    // Создаем HTTP сервер
    let server = enhanced_share!(HttpServer::new(port));

    // Создаем счетчик переходов (как в basic_usage.rs) - без клонов!
    let visits = share!(0);

    // Запускаем потоки напрямую через EnhancedThreadShare
    let visits_clone = visits.clone();
    server
        .spawn(
            "server_main",
            move |server: thread_share::ThreadShare<HttpServer>| {
                println!("🌐 Server main thread started");

                // Запускаем сервер
                server.update(|s| s.start().expect("Failed to start server"));

                // Создаем TCP listener
                let port = server.get().port;
                let listener =
                    TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind");
                println!("🔌 Listening on http://127.0.0.1:{}", port);
                println!("🌐 Server URLs:");
                println!("   • Main page: http://127.0.0.1:{}/", port);
                println!("   • Status: http://127.0.0.1:{}/status", port);
                println!("   • Health: http://127.0.0.1:{}/health", port);

                // Принимаем соединения
                for stream in listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                            // Увеличиваем счетчик соединений
                            server.update(|s| s.increment_connections());

                            // Обрабатываем запрос и проверяем, была ли это основная страница
                            let is_main_page = match server.write(|s| s.handle_request(&mut stream))
                            {
                                Ok(is_main) => is_main,
                                Err(e) => {
                                    eprintln!("❌ Error handling request: {}", e);
                                    false
                                }
                            };

                            // Увеличиваем счетчик переходов только для основных страниц (не для статики)
                            if is_main_page {
                                visits_clone.update(|v| *v += 1);
                                println!("🌐 Main page visit - visits: {}", visits_clone.get());
                            }

                            // Уменьшаем счетчик соединений
                            server.update(|s| s.decrement_connections());

                            // Даем время браузеру получить ответ
                            std::thread::sleep(Duration::from_millis(200));
                        }
                        Err(e) => {
                            eprintln!("❌ Connection failed: {}", e);
                        }
                    }

                    // Проверяем, должен ли сервер остановиться
                    if !server.get().is_running {
                        break;
                    }
                }

                println!("🌐 Server main thread finished");
            },
        )
        .expect("Failed to spawn server_main");

    server.spawn("monitor", |server: thread_share::ThreadShare<HttpServer>| {
        println!("📊 Monitor thread started");

        // Мониторим сервер в реальном времени
        for _ in 1..=30 {
            let current_server = server.get();

            if current_server.is_running {
                println!("📊 Server Status: Running | Port: {} | Requests: {} | Connections: {} | Uptime: {}", 
                    current_server.port,
                    current_server.requests_handled,
                    current_server.active_connections,
                    current_server.get_uptime()
                );
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        // Останавливаем сервер через 1 минуту
        println!("⏰ Stopping server after 1 minute...");
        server.update(|s| s.stop());
        println!("📊 Monitor thread finished");
    }).expect("Failed to spawn monitor");

    // Убираем симуляцию - visits будет увеличиваться только при реальных HTTP запросах

    // Главный поток - ждем завершения
    println!("🚀 HTTP Server Example Started");
    println!("🌐 Server will run for 1 minute");
    println!("🧵 Active threads: {}", server.active_threads());

    // Ждем завершения всех потоков
    server.join_all().expect("Failed to join threads");

    // Финальная статистика
    let final_server = server.get();
    let final_visits = visits.get();

    println!("\n=== 🏁 Final Results ===");
    println!(
        "🌐 Server status: {}",
        if final_server.is_running {
            "Running"
        } else {
            "Stopped"
        }
    );
    println!(
        "📊 Total requests handled: {}",
        final_server.requests_handled
    );
    println!("⏱️  Total uptime: {}", final_server.get_uptime());
    println!("👁️  Total visits: {}", final_visits);

    println!("\n✅ Example completed successfully!");
    println!("🎯 This example demonstrates:");
    println!("   • Using thread_setup! macro for simplified server setup");
    println!("   • Simple HTTP server implementation");
    println!("   • Multi-threaded server with monitoring");
    println!("   • Real-time server statistics");
    println!("   • Graceful server shutdown");
}
