[![Crates.io](https://img.shields.io/crates/v/thread-share?style=for-the-badge)](https://crates.io/crates/thread-share)
[![Documentation](https://img.shields.io/badge/docs-docs.rs-blue?style=for-the-badge)](https://docs.rs/thread-share/)
[![GitHub issues](https://img.shields.io/badge/github-issues-orange?style=for-the-badge)](https://github.com/s00d/thread-share/issues)
[![GitHub stars](https://img.shields.io/badge/github-stars-yellow?style=for-the-badge)](https://github.com/s00d/thread-share/stargazers)
[![Donate](https://img.shields.io/badge/Donate-Donationalerts-ff4081?style=for-the-badge)](https://www.donationalerts.com/r/s00d88)

# Thread-Share

> **"I got tired of playing around with data passing between threads and decided to write this library"**

A powerful Rust library for safe data exchange between threads with automatic thread management.

## 🎯 What Problem Does This Solve?

Working with shared data between threads in Rust is often frustrating:
- Manual `Arc<Mutex<T>>` or `Arc<RwLock<T>>` combinations
- Complex ownership patterns
- Boilerplate code for every thread-safe structure
- Manual thread spawning and joining

**Thread-Share** provides a simple API that handles all the complexity for you.

## 🚀 Features

- **🔒 Thread-Safe**: Built-in synchronization with `RwLock` and `AtomicPtr`
- **⚡ High Performance**: Efficient `parking_lot` synchronization primitives
- **🧵 Automatic Thread Management**: Spawn and manage multiple threads with one macro
- **📦 Zero-Copy**: Support for working without cloning data between threads
- **🔄 Change Detection**: Built-in waiting mechanisms for data changes
- **✨ Macro Support**: Convenient macros for quick setup
- **📄 Serialization Support**: JSON serialization for all types with `serialize` feature

## 📦 Installation

```bash
# Basic installation
cargo add thread-share

# With serialization support
cargo add thread-share --features serialize
```

## 🚀 Quick Start

### Basic Usage with Thread Management

```rust
use thread_share::{share, enhanced_share, spawn_workers};
use std::time::Duration;

fn main() {
    // Create shared counter
    let counter = enhanced_share!(0);
    
    // Spawn multiple workers with one macro call
    let manager = spawn_workers!(counter, {
        incrementer: |counter| {
            for i in 1..=10 {
                counter.set(i);
                std::thread::sleep(Duration::from_millis(100));
            }
        },
        monitor: |counter| {
            for _ in 0..10 {
                println!("Value: {}", counter.get());
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    });
    
    // Main thread reads values
    while counter.get() < 10 {
        println!("Counter: {}", counter.get());
        std::thread::sleep(Duration::from_millis(150));
    }
    
    // Add additional worker dynamically
    let counter_clone = counter.clone();
    let additional_worker = std::thread::spawn(move || {
        for _ in 0..3 {
            counter_clone.update(|x| *x *= 2);
            std::thread::sleep(Duration::from_millis(300));
        }
    });
    
    manager.add_worker("multiplier", additional_worker)?;
    
    // Demonstrate worker control
    println!("Active workers: {}", manager.active_workers());
    println!("Worker names: {:?}", manager.get_worker_names());
    
    // Pause incrementer worker temporarily
    manager.pause_worker("incrementer")?;
    println!("Incrementer paused for 1 second");
    std::thread::sleep(Duration::from_secs(1));
    
    // Resume incrementer worker
    manager.resume_worker("incrementer")?;
    println!("Incrementer resumed");
    
    // Remove monitor worker early
    manager.remove_worker("monitor")?;
    println!("Monitor worker stopped and removed");
    
    // Stop all remaining workers at once
    manager.remove_all_workers()?;
    println!("All workers stopped");
    
    // Check final state
    println!("Final active workers: {}", manager.active_workers());
    
    // Wait for remaining workers to complete
    manager.join_all().expect("Failed to join threads");
}
```

## 🔧 Core Types

### 1. ThreadShare<T> - Basic Thread-Safe Data

```rust
use thread_share::share;

let data = share!(vec![1, 2, 3]);

// Basic operations
data.set(vec![4, 5, 6]);           // Set new value
let value = data.get();             // Get copy of data
data.update(|v| v.push(7));        // Update in place

// Thread-safe operations
data.wait_for_change_forever();     // Wait for changes
data.wait_for_change(timeout);      // Wait with timeout
```

### 2. EnhancedThreadShare<T> - Automatic Thread Management

```rust
use thread_share::{enhanced_share, spawn_workers};

let data = enhanced_share!(0);

// Spawn multiple threads with one macro
let manager = spawn_workers!(data, {
    worker1: |data| { /* thread logic */ },
    worker2: |data| { /* thread logic */ }
});

// Automatic thread joining
manager.join_all().expect("Failed to join threads");
```

### 3. WorkerManager - Fine-Grained Control

```rust
let manager = spawn_workers!(data, {
    worker: |data| { /* logic */ }
});

// Control individual workers
manager.pause_worker("worker")?;
manager.resume_worker("worker")?;
manager.remove_worker("worker")?;

// Monitor workers
println!("Active workers: {}", manager.active_workers());
println!("Worker names: {:?}", manager.get_worker_names());
```

#### Creating WorkerManager Directly

```rust
use thread_share::worker_manager::WorkerManager;
use std::thread;
use std::time::Duration;

fn main() {
    // Create empty manager
    let manager = WorkerManager::new();
    
    // Add workers programmatically
    let handle1 = thread::spawn(|| {
        for i in 1..=5 {
            println!("Worker 1: {}", i);
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    let handle2 = thread::spawn(|| {
        for i in 1..=3 {
            println!("Worker 2: {}", i);
            thread::sleep(Duration::from_millis(150));
        }
    });
    
    // Add workers to manager
    manager.add_worker("worker1", handle1)?;
    manager.add_worker("worker2", handle2)?;
    
    println!("Active workers: {}", manager.active_workers());
    println!("Worker names: {:?}", manager.get_worker_names());
    
    // Wait for completion
    manager.join_all()?;
}
```

#### Creating WorkerManager with Existing Threads

```rust
use thread_share::{enhanced_share, worker_manager::WorkerManager};

fn main() {
    let data = enhanced_share!(0);
    
    // Get existing threads from EnhancedThreadShare
    let existing_threads = data.get_threads();
    
    // Create manager with existing threads
    let manager = WorkerManager::new_with_threads(existing_threads);
    
    // Add new worker
    let data_clone = data.clone();
    let new_worker = thread::spawn(move || {
        for _ in 0..3 {
            data_clone.update(|x| *x += 10);
            thread::sleep(Duration::from_millis(200));
        }
    });
    
    manager.add_worker("additional_worker", new_worker)?;
    
    // Monitor and control
    println!("Active workers: {}", manager.active_workers());
    manager.join_all()?;
}
```

#### Worker State Management

```rust
use thread_share::{enhanced_share, spawn_workers};

fn main() {
    let data = enhanced_share!(0);
    
    let manager = spawn_workers!(data, {
        counter: |data| {
            for i in 1..=10 {
                data.set(i);
                thread::sleep(Duration::from_millis(500));
            }
        },
        monitor: |data| {
            for _ in 0..10 {
                println!("Value: {}", data.get());
                thread::sleep(Duration::from_millis(1000));
            }
        }
    });
    
    // Control worker states
    thread::sleep(Duration::from_secs(2));
    
    // Pause counter worker
    manager.pause_worker("counter")?;
    println!("Counter worker paused");
    
    thread::sleep(Duration::from_secs(1));
    
    // Resume counter worker
    manager.resume_worker("counter")?;
    println!("Counter worker resumed");
    
    // Remove monitor worker
    manager.remove_worker("monitor")?;
    println!("Monitor worker removed");
    
    // Wait for remaining workers
    manager.join_all()?;
}
```

## 📚 Examples

### Producer-Consumer Pattern

```rust
use thread_share::{enhanced_share, spawn_workers};

fn main() {
    let queue = enhanced_share!(Vec::<String>::new());
    
    // Spawn producer and consumer threads
    let manager = spawn_workers!(queue, {
        producer: |queue| {
            for i in 0..5 {
                queue.update(|q| q.push(format!("Message {}", i)));
                std::thread::sleep(Duration::from_millis(100));
            }
        },
        consumer: |queue| {
            let mut processed = 0;
            while processed < 5 {
                let messages = queue.get();
                if !messages.is_empty() {
                    queue.update(|q| {
                        if let Some(msg) = q.pop() {
                            println!("Processed: {}", msg);
                            processed += 1;
                        }
                    });
                } else {
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    });
    
    // Wait for all workers
    manager.join_all().expect("Failed to join threads");
}
```

### HTTP Server with Visit Counter

```rust
use thread_share::{enhanced_share, spawn_workers};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn main() {
    let visits = enhanced_share!(0);
    
    // Spawn server thread
    let manager = spawn_workers!(visits, {
        server: |visits| {
            let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
            
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut buffer = [0; 1024];
                
                stream.read(&mut buffer).unwrap();
                
                // Increment visit counter for main page requests
                if buffer.starts_with(b"GET / ") {
                    visits.update(|v| *v += 1);
                }
                
                let response = "HTTP/1.1 200 OK\r\n\r\nHello World!";
                stream.write(response.as_bytes()).unwrap();
            }
        }
    });
    
    // Monitor visits
    for _ in 0..10 {
        std::thread::sleep(Duration::from_secs(1));
        println!("Total visits: {}", visits.get());
    }
}
```

### Dynamic Worker Management

```rust
use thread_share::{enhanced_share, spawn_workers, worker_manager::WorkerManager};

fn main() {
    let data = enhanced_share!(0);
    
    // Start with basic workers
    let manager = spawn_workers!(data, {
        counter: |data| {
            for i in 1..=5 {
                data.set(i);
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    });
    
    // Add new worker dynamically
    let data_clone = data.clone();
    let new_worker = std::thread::spawn(move || {
        for _ in 0..3 {
            data_clone.update(|x| *x *= 2);
            std::thread::sleep(Duration::from_millis(300));
        }
    });
    
    manager.add_worker("multiplier", new_worker)?;
    
    // Control workers
    manager.pause_worker("counter")?;
    std::thread::sleep(Duration::from_secs(1));
    manager.resume_worker("counter")?;
    
    // Wait for all
    manager.join_all()?;
}
```

### Custom Types with Change Detection

```rust
use thread_share::{enhanced_share, spawn_workers};

#[derive(Clone, Debug)]
struct User {
    name: String,
    age: u32,
    is_online: bool,
}

fn main() {
    let user = enhanced_share!(User {
        name: "Alice".to_string(),
        age: 25,
        is_online: true,
    });
    
    // Spawn multiple workers that update user data
    let manager = spawn_workers!(user, {
        age_updater: |user| {
            for _ in 0..5 {
                user.update(|u| u.age += 1);
                std::thread::sleep(Duration::from_millis(200));
            }
        },
        status_toggler: |user| {
            for _ in 0..5 {
                user.update(|u| u.is_online = !u.is_online);
                std::thread::sleep(Duration::from_millis(300));
            }
        },
        monitor: |user| {
            for _ in 0..10 {
                let current = user.get();
                println!("User: {} ({}), Age: {}, Online: {}", 
                    current.name, current.age, current.age, current.is_online);
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    });
    
    // Wait for all workers
    manager.join_all().expect("Failed to join threads");
    
    let final_user = user.get();
    println!("Final state: {:?}", final_user);
}
```

## 🔍 When to Use Each Type

| Type | Use Case | Description |
|------|----------|-------------|
| **ThreadShare** | Simple data sharing | Basic thread-safe data with manual thread management |
| **EnhancedThreadShare** | Multi-threaded apps | Automatic thread spawning and joining |
| **WorkerManager** | Complex workflows | Fine-grained control over individual workers |

## ⚠️ Important Notes

### ArcThreadShare<T> - Use with Caution

```rust
use thread_share::ArcThreadShare;

let data = ArcThreadShare::new(0);

// ⚠️ Complex operations are NOT atomic
data.update(|x| *x += 1);  // May lose updates under high contention

// ✅ Use atomic methods instead
data.increment();           // Atomic increment
data.add(5);               // Atomic addition
```

**When NOT to use ArcThreadShare:**
- High-frequency updates (>1000 ops/second)
- Critical data integrity requirements
- Predictable performance needs

## 🧪 Running Examples

```bash
# Basic examples
cargo run --example basic_usage
cargo run --example atomic_usage

# Advanced examples with WorkerManager
cargo run --example worker_management
cargo run --example http_integration_helpers
cargo run --example socket_client_usage

# Run tests
cargo test
```

## 🔧 Requirements

- **Rust**: 1.85.0 or higher
- **Dependencies**: `parking_lot` (required), `serde` (optional)

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions welcome! Please submit a Pull Request.

## 📚 More Examples

Check the `examples/` directory for complete working examples:
- HTTP servers with Tokio and async-std
- Socket client with Node.js server
- Advanced worker management patterns
- Performance benchmarks