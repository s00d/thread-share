//! # Thread-Share
//!
//! A comprehensive Rust library for safe and efficient data exchange between threads.
//!
//! ## 🚀 Features
//!
//! - **Simple API** for data exchange between threads
//! - **Automatic synchronization** and thread safety
//! - **Support for any data types** that implement `Clone`
//! - **Efficient synchronization** using `parking_lot`
//! - **Multiple complexity levels**: from simple locks to zero-copy atomic operations
//! - **Enhanced thread management** with automatic spawning and joining
//! - **Smart macros** for cleaner thread management syntax
//! - **Real-world examples** including HTTP server and socket client
//!
//! ## 🎯 Quick Start
//!
//! ```rust
//! use thread_share::share;
//!
//! // Create shared data
//! let counter = share!(0);
//! let clone = counter.clone();
//!
//! // Spawn a simple thread
//! std::thread::spawn(move || {
//!     clone.update(|x| *x = *x + 1);
//! });
//!
//! // Wait a bit and check result
//! std::thread::sleep(std::time::Duration::from_millis(100));
//! println!("Final value: {}", counter.get());
//! ```
//!
//! ## 🏗️ Architecture Overview
//!
//! The library provides several levels of abstraction:
//!
//! ### 🔒 Lock-Based (Safe & Predictable)
//! - **`ThreadShare<T>`** - Full-featured synchronization with change detection
//! - **`SimpleShare<T>`** - Lightweight alternative for basic use cases
//! - **`ArcThreadShareLocked<T>`** - Safe zero-copy with read/write locks
//!
//! ### ⚡ Atomic-Based (High Performance)
//! - **`ArcThreadShare<T>`** - Zero-copy atomic operations (use with caution)
//!
//! ### 🧵 Enhanced Management
//! - **`EnhancedThreadShare<T>`** - Automatic thread spawning and joining
//! - **`ThreadManager`** - Standalone thread management utility
//!
//! ## 📚 Core Concepts
//!
//! ### ThreadShare<T> - Main Structure
//! Provides comprehensive thread synchronization with automatic cloning:
//!
//! ```rust
//! use thread_share::share;
//!
//! let data = share!(vec![1, 2, 3]);
//! let clone = data.clone();
//!
//! // Thread 1
//! std::thread::spawn(move || {
//!     clone.update(|v| v.push(4));
//! });
//!
//! // Thread 2 (main)
//! data.wait_for_change_forever();
//! println!("Updated: {:?}", data.get());
//! ```
//!
//! ### ThreadShare<T> - Basic Management
//! Simple thread management with manual control:
//!
//! ```rust
//! use thread_share::share;
//!
//! let data = share!(vec![1, 2, 3]);
//! let clone = data.clone();
//!
//! std::thread::spawn(move || {
//!     clone.update(|d| d.push(4));
//! });
//!
//! std::thread::sleep(std::time::Duration::from_millis(100));
//! println!("Updated: {:?}", data.get());
//! ```
//!
//! ## 🔧 Usage Patterns
//!
//! ### Serialization Support (Optional Feature)
//!
//! The library provides optional serialization support through the `serialize` feature:
//!
//! ```bash
//! # Enable serialization support
//! cargo add thread-share --features serialize
//! ```
//!
//! ```rust
//! use thread_share::ThreadShare;
//!
//! // Note: This example requires the 'serialize' feature
//! let data = ThreadShare::new(vec![1, 2, 3]);
//!
//! // Serialize to JSON (available with serialize feature)
//! // let json = data.to_json().expect("Failed to serialize");
//! // println!("JSON: {}", json); // Output: [1,2,3]
//!
//! // Deserialize from JSON (available with serialize feature)
//! // data.from_json("[4,5,6]").expect("Failed to deserialize");
//! // assert_eq!(data.get(), vec![4, 5, 6]);
//! ```
//!
//! **Note**: Serialization methods are only available when the `serialize` feature is enabled.
//!
//! ### Basic Data Sharing
//! ```rust
//! use thread_share::share;
//!
//! let counter = share!(0);
//! let clone = counter.clone();
//!
//! std::thread::spawn(move || {
//!     for _ in 0..100 {
//!         clone.update(|x| *x += 1);
//!     }
//! });
//!
//! std::thread::sleep(std::time::Duration::from_millis(100));
//! println!("Counter: {}", counter.get());
//! ```
//!
//! ### Zero-Copy Operations
//! ```rust
//! use thread_share::{share, ArcThreadShare};
//!
//! let data = share!(String::from("Hello"));
//! let arc_data = data.as_arc();
//! let arc_share = ArcThreadShare::from_arc(arc_data);
//!
//! // Use atomic operations for performance
//! arc_share.update(|s| s.push_str(" World"));
//! ```
//!
//! ### Enhanced Thread Management
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//!
//! let data = enhanced_share!(vec![1, 2, 3]);
//!
//! spawn_workers!(data, {
//!     processor: |data| { data.update(|v| v.sort()); },
//!     validator: |data| { assert!(data.get().is_sorted()); }
//! });
//!
//! data.join_all().expect("Failed to join");
//! ```
//!
//! ## ⚠️ Important Notes
//!
//! ### ArcThreadShare Limitations
//! - **Complex operations are not atomic** - use `increment()` and `add()` methods
//! - **High contention can cause lost updates** - test with realistic load
//! - **Memory allocation overhead** per operation
//!
//! ### Best Practices
//! - Use **`ThreadShare<T>`** for most applications
//! - Use **`ArcThreadShare<T>`** only when you understand its limitations
//! - Use **`EnhancedThreadShare<T>`** for simplified thread management
//! - Always test with realistic contention levels
//!
//! ## 📖 Examples
//!
//! Check the `examples/` directory for complete working examples:
//!
//! - **`basic_usage.rs`** - Fundamental concepts
//! - **`http_integration_helpers.rs`** - Complete HTTP server
//! - **`socket_client_usage.rs`** - Enhanced socket client
//! - **`atomic_usage.rs`** - Zero-copy patterns
//!
//! ## 🧪 Testing
//!
//! Run the comprehensive test suite:
//!
//! ```bash
//! cargo test                    # All tests
//! cargo test --test core_tests # Specific test file
//! cargo test -- --nocapture    # With output
//! ```
//!
//! ## 📄 License
//!
//! This project is licensed under the MIT License.
//!
//! ## 🤝 Contributing
//!
//! Contributions are welcome! Please feel free to submit a Pull Request.

pub mod atomic;
pub mod core;
pub mod enhanced;
pub mod locked;
pub mod macros;
pub mod thread_pool;
pub mod worker_manager;

// Re-export main structures
pub use atomic::ArcThreadShare;
pub use core::{SimpleShare, ThreadShare};
pub use enhanced::EnhancedThreadShare;
pub use locked::ArcThreadShareLocked;
pub use thread_pool::ThreadManager;


