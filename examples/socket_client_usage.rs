use std::net::TcpStream;
use std::time::Duration;
use thread_share::{enhanced_share, spawn_workers};

// Структуры для работы с сокет-клиентом
#[derive(Clone, Debug)]
struct SocketStatus {
    is_connected: bool,
    last_error: Option<String>,
    address: String,
}

#[derive(Clone, Debug)]
struct Message {
    content: String,
}

#[derive(Clone, Debug)]
struct ClientStats {
    messages_sent: u32,
    messages_received: u32,
    connection_attempts: u32,
    total_bytes_sent: u64,
    total_bytes_received: u64,
}

#[derive(Clone, Debug)]
struct SocketClient {
    status: SocketStatus,
    stats: ClientStats,
    message_queue: Vec<Message>,
}

impl SocketClient {
    fn new(address: String) -> Self {
        Self {
            status: SocketStatus {
                is_connected: false,
                last_error: None,
                address,
            },
            stats: ClientStats {
                messages_sent: 0,
                messages_received: 0,
                connection_attempts: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
            },
            message_queue: Vec::new(),
        }
    }

    fn connect(&mut self) -> Result<(), String> {
        match TcpStream::connect(&self.status.address) {
            Ok(_) => {
                self.status.is_connected = true;
                self.status.last_error = None;
                self.stats.connection_attempts += 1;
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to connect: {}", e);
                self.status.last_error = Some(error_msg.clone());
                self.status.is_connected = false;
                self.stats.connection_attempts += 1;
                Err(error_msg)
            }
        }
    }

    fn disconnect(&mut self) {
        self.status.is_connected = false;
        self.status.last_error = Some("Disconnected by user".to_string());
    }

    fn send_message(&mut self, message: &str) -> Result<usize, String> {
        if !self.status.is_connected {
            return Err("Not connected".to_string());
        }

        // Симулируем отправку (в реальном приложении здесь был бы TcpStream)
        let bytes_sent = message.len();
        self.stats.messages_sent += 1;
        self.stats.total_bytes_sent += bytes_sent as u64;

        // Track message count (content field removed to avoid warning)
        self.message_queue.push(Message {
            content: message.to_string(),
        });

        Ok(bytes_sent)
    }

    fn receive_message(&mut self) -> Result<String, String> {
        if !self.status.is_connected {
            return Err("Not connected".to_string());
        }

        // Симулируем получение ответа
        let response = format!(
            "Server response to message {}",
            self.stats.messages_received + 1
        );
        let bytes_received = response.len();

        self.stats.messages_received += 1;
        self.stats.total_bytes_received += bytes_received as u64;

        Ok(response)
    }
}

fn main() {
    println!("=== Socket Client Example with EnhancedThreadShare ===");

    // Создаем общий клиент сокета с улучшенным управлением потоками
    let client = enhanced_share!(SocketClient::new("localhost:8080".to_string()));

    // Запускаем все потоки одной командой!
    spawn_workers!(client, {
        connection: |client: thread_share::ThreadShare<SocketClient>| {
            let mut attempts = 0;

            while attempts < 3 {
                println!("Attempting to connect to localhost:8080 (attempt {})", attempts + 1);

                match client.write(|client| client.connect()) {
                    Ok(_) => {
                        println!("✅ Successfully connected!");
                        break;
                    }
                    Err(e) => {
                        println!("❌ Connection failed: {}", e);
                        attempts += 1;
                        std::thread::sleep(Duration::from_millis(2000));
                    }
                }
            }

            if attempts >= 3 {
                println!("❌ Failed to connect after 3 attempts");
            }
        },

        sender: |client: thread_share::ThreadShare<SocketClient>| {
            // Ждем подключения
            while !client.get().status.is_connected {
                std::thread::sleep(Duration::from_millis(100));
            }

            println!("📤 Sender thread started");

            // Отправляем тестовые сообщения
            for i in 1..=5 {
                let message = format!("Hello Server! Message {}", i);

                match client.write(|client| client.send_message(&message)) {
                    Ok(bytes_sent) => {
                        println!("📤 Sent: {} ({} bytes)", message, bytes_sent);
                    }
                    Err(e) => {
                        println!("❌ Failed to send message: {}", e);
                    }
                }

                std::thread::sleep(Duration::from_millis(500));
            }

            // Отключаемся после отправки всех сообщений
            client.update(|client| client.disconnect());
            println!("📤 Sender thread finished");
        },

        receiver: |client| {
            // Ждем подключения
            while !client.get().status.is_connected {
                std::thread::sleep(Duration::from_millis(100));
            }

            println!("📥 Receiver thread started");

            // Получаем ответы от сервера
            for _ in 1..=5 {
                // Ждем немного перед "получением" ответа
                std::thread::sleep(Duration::from_millis(600));

                match client.write(|client| client.receive_message()) {
                    Ok(response) => {
                        println!("📥 Received: {} ({} bytes)", response, response.len());
                    }
                    Err(e) => {
                        println!("❌ Failed to receive message: {}", e);
                        break;
                    }
                }

                // Проверяем, не отключились ли
                if !client.get().status.is_connected {
                    break;
                }
            }

            println!("📥 Receiver thread finished");
        }
    });

    // Главный поток - мониторинг состояния
    println!("🚀 Socket Client Example Started");
    println!("🔌 Connecting to localhost:8080...");

    let mut last_stats = ClientStats {
        messages_sent: 0,
        messages_received: 0,
        connection_attempts: 0,
        total_bytes_sent: 0,
        total_bytes_received: 0,
    };

    // Мониторинг в реальном времени
    while client.get().status.is_connected
        || client.get().stats.messages_sent < 5
        || client.get().stats.messages_received < 5
    {
        let current_client = client.get();

        // Выводим изменения статистики
        if current_client.stats.messages_sent != last_stats.messages_sent
            || current_client.stats.messages_received != last_stats.messages_received
            || current_client.stats.connection_attempts != last_stats.connection_attempts
        {
            println!("\n=== 📊 Status Update ===");
            println!(
                "🔌 Connection: {}",
                if current_client.status.is_connected {
                    "✅ Connected"
                } else {
                    "❌ Disconnected"
                }
            );
            println!("📤 Messages sent: {}", current_client.stats.messages_sent);
            println!(
                "📥 Messages received: {}",
                current_client.stats.messages_received
            );
            println!(
                "🔄 Connection attempts: {}",
                current_client.stats.connection_attempts
            );
            println!(
                "📊 Total bytes sent: {}",
                current_client.stats.total_bytes_sent
            );
            println!(
                "📊 Total bytes received: {}",
                current_client.stats.total_bytes_received
            );
            println!(
                "📋 Message queue size: {}",
                current_client.message_queue.len()
            );
            if let Some(last_msg) = current_client.message_queue.last() {
                println!("📝 Last message: {}", last_msg.content);
            }
            println!("🧵 Active threads: {}", client.active_threads());

            if let Some(ref error) = current_client.status.last_error {
                println!("⚠️  Last error: {}", error);
            }
            println!("==================\n");

            last_stats = current_client.stats.clone();
        }

        std::thread::sleep(Duration::from_millis(200));
    }

    // Ждем завершения всех потоков одной командой!
    client.join_all().expect("Failed to join threads");

    // Финальная статистика
    let final_client = client.get();

    println!("\n=== 🏁 Final Results ===");
    println!(
        "🔌 Connection status: {}",
        if final_client.status.is_connected {
            "✅ Connected"
        } else {
            "❌ Disconnected"
        }
    );
    println!(
        "📤 Total messages sent: {}",
        final_client.stats.messages_sent
    );
    println!(
        "📥 Total messages received: {}",
        final_client.stats.messages_received
    );
    println!(
        "🔄 Total connection attempts: {}",
        final_client.stats.connection_attempts
    );
    println!(
        "📊 Total bytes sent: {}",
        final_client.stats.total_bytes_sent
    );
    println!(
        "📊 Total bytes received: {}",
        final_client.stats.total_bytes_received
    );
    println!(
        "📋 Final message queue size: {}",
        final_client.message_queue.len()
    );

    if let Some(ref error) = final_client.status.last_error {
        println!("⚠️  Final error: {}", error);
    }

    println!("\n✅ Example completed successfully!");
    println!("🎯 This example demonstrates:");
    println!("   • Using EnhancedThreadShare for simplified thread management");
    println!("   • Single macro call to spawn multiple threads");
    println!("   • Automatic thread joining with join_all()");
    println!("   • Real-time monitoring of client state and thread count");
    println!("   • Error handling and status tracking");
    println!("   • No more manual clone() and thread::spawn() calls!");
}
