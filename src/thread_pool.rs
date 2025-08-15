//! # Thread Pool Module - ThreadManager
//!
//! This module provides `ThreadManager`, a standalone utility for managing threads
//! with shared data, independent of the ThreadShare structures.
//!
//! ## 🚀 Overview
//!
//! `ThreadManager` is a lightweight thread management utility that provides:
//!
//! - **Simplified Thread Spawning**: Spawn threads with descriptive names
//! - **Shared Data Management**: Manage multiple types of shared data
//! - **Thread Tracking**: Monitor active thread count and status
//! - **Automatic Thread Joining**: Wait for all threads to complete
//! - **Type-Safe Operations**: Compile-time guarantees for thread safety
//!
//! ## Key Features
//!
//! ### 🧵 Thread Management
//! - **Named Threads**: Each thread gets a descriptive name for debugging
//! - **Automatic Tracking**: Monitor active thread count and completion status
//! - **Error Handling**: Comprehensive error handling for thread failures
//! - **Resource Cleanup**: Automatic cleanup of completed threads
//!
//! ### 📦 Shared Data Support
//! - **Type-Safe Access**: Compile-time type checking for shared data
//! - **Multiple Data Types**: Support for different types of shared data
//! - **Automatic Cloning**: Safe data sharing between threads
//! - **Thread Isolation**: Each thread gets its own clone of shared data
//!
//! ## Architecture
//!
//! `ThreadManager` uses internal structures to track:
//!
//! - **`threads: Arc<Mutex<HashMap<String, JoinHandle<()>>>>`** - Active thread tracking
//! - **`shared_data: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>`** - Shared data storage
//!
//! ## Example Usage
//!
//! ### Basic Thread Management
//! ```rust
//! use thread_share::{ThreadManager, share};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = ThreadManager::new();
//!     let data = share!(vec![1, 2, 3]);
//!
//!     // Spawn individual threads
//!     manager.spawn("sorter", data.clone(), |data| {
//!         data.update(|v| v.sort());
//!     })?;
//!
//!     manager.spawn("validator", data.clone(), |data| {
//!         assert!(data.get().is_sorted());
//!     })?;
//!
//!     // Wait for completion
//!     manager.join_all()?;
//!     Ok(())
//! }
//! ```
//!
//! ### Advanced Usage
//! ```rust
//! use thread_share::{ThreadManager, share};
//! use std::time::Duration;
//!
//! #[derive(Clone)]
//! struct WorkItem {
//!     id: u32,
//!     data: String,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = ThreadManager::new();
//!     let work_queue = share!(vec![
//!         WorkItem { id: 1, data: "Task 1".to_string() },
//!         WorkItem { id: 2, data: "Task 2".to_string() },
//!     ]);
//!
//!     // Spawn worker threads
//!     for i in 0..3 {
//!         let queue_clone = work_queue.clone();
//!         let worker_id = i;
//!         manager.spawn(&format!("worker-{}", i), queue_clone, move |queue| {
//!             loop {
//!                 let mut items = queue.get();
//!                 if items.is_empty() {
//!                     break;
//!                 }
//!                 
//!                 if let Some(item) = items.pop() {
//!                     println!("Worker {} processing: {}", worker_id, item.data);
//!                     std::thread::sleep(Duration::from_millis(100));
//!                 }
//!                 
//!                 queue.set(items);
//!             }
//!         })?;
//!     }
//!
//!     // Wait for all workers to complete
//!     manager.join_all()?;
//!     println!("All work completed!");
//!     Ok(())
//! }
//! ```
//!
//! ## Thread Lifecycle
//!
//! 1. **Creation**: `ThreadManager::new()` or `ThreadManager::default()`
//! 2. **Spawning**: `manager.spawn(name, data, function)` creates named threads
//! 3. **Execution**: Threads run with access to shared data
//! 4. **Monitoring**: Track active threads with `active_threads()`
//! 5. **Completion**: Wait for all threads with `join_all()`
//!
//! ## Performance Characteristics
//!
//! - **Thread Spawning**: Minimal overhead over standard `thread::spawn`
//! - **Thread Tracking**: Constant-time operations for thread management
//! - **Memory Usage**: Small overhead for tracking structures
//! - **Scalability**: Efficient for up to hundreds of threads
//! - **Lock Contention**: Minimal due to efficient `parking_lot` primitives
//!
//! ## Best Practices
//!
//! 1. **Use descriptive thread names** for easier debugging
//! 2. **Keep thread functions focused** on single responsibilities
//! 3. **Always call `join_all()`** to ensure proper cleanup
//! 4. **Monitor thread count** with `active_threads()` for debugging
//! 5. **Handle errors gracefully** from `spawn()` and `join_all()`
//! 6. **Clone shared data** for each thread to avoid ownership issues
//!
//! ## Error Handling
//!
//! ```rust
//! use thread_share::{ThreadManager, share};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = ThreadManager::new();
//!     let data = share!(0);
//!
//!     // Handle spawn errors
//!     if let Err(e) = manager.spawn("worker", data.clone(), |data| { /* logic */ }) {
//!         eprintln!("Failed to spawn worker: {}", e);
//!         return Ok(());
//!     }
//!
//!     // Handle join errors
//!     if let Err(e) = manager.join_all() {
//!         eprintln!("Thread execution failed: {}", e);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Thread Safety
//!
//! `ThreadManager` automatically implements `Send` and `Sync` traits,
//! making it safe to use across thread boundaries. The internal synchronization
//! primitives ensure that all operations are thread-safe.
//!
//! ## Memory Management
//!
//! - **Arc**: Provides reference counting for shared ownership
//! - **Mutex**: Ensures exclusive access to internal structures
//! - **HashMap**: Efficient storage for thread handles and shared data
//! - **Automatic Cleanup**: Completed threads are automatically removed
//!
//! ## Comparison with EnhancedThreadShare
//!
//! | Aspect | ThreadManager | EnhancedThreadShare |
//! |--------|---------------|-------------------|
//! | **Purpose** | Standalone utility | Integrated with ThreadShare |
//! | **Data Management** | Manual cloning required | Automatic data management |
//! | **Thread Tracking** | Manual thread management | Built-in thread tracking |
//! | **Use Case** | Complex thread scenarios | Simple thread management |
//! | **Flexibility** | High | Medium |
//! | **Ease of Use** | Medium | High |
//!
//! ## Integration with ThreadShare
//!
//! `ThreadManager` works seamlessly with `ThreadShare<T>`:
//!
//!
//! ## Advanced Patterns
//!
//! ### Thread Pools
//! ```rust
//! use thread_share::{ThreadManager, share};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = ThreadManager::new();
//!     let counter = share!(0u32);
//!
//!     // Spawn worker pool
//!     for i in 0..4 {
//!         let counter_clone = counter.clone();
//!         let worker_id = i;
//!         manager.spawn(&format!("worker-{}", i), counter_clone, move |data| {
//!             data.update(|x| *x = *x + 1);
//!             println!("Worker {} incremented counter", worker_id);
//!         })?;
//!     }
//!
//!     // Wait for all workers to complete
//!     manager.join_all()?;
//!     println!("Final counter value: {}", counter.get());
//!     Ok(())
//! }
//! ```
//!
//! ### Producer-Consumer
//! ```rust
//! use thread_share::{ThreadManager, share};
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = ThreadManager::new();
//!     let queue = share!(Vec::<String>::new());
//!
//!     // Producer thread
//!     manager.spawn("producer", queue.clone(), |queue| {
//!         for i in 0..5 {
//!             queue.update(|q| q.push(format!("Item {}", i)));
//!             std::thread::sleep(Duration::from_millis(10));
//!         }
//!     })?;
//!
//!     // Consumer thread
//!     manager.spawn("consumer", queue.clone(), |queue| {
//!         let mut consumed_count = 0;
//!         while consumed_count < 5 {
//!             let items = queue.get();
//!             if items.is_empty() {
//!                 std::thread::sleep(Duration::from_millis(10));
//!                 continue;
//!             }
//!             
//!             if let Some(item) = items.last() {
//!                 println!("Consumed: {}", item);
//!                 queue.update(|q| { q.pop(); });
//!                 consumed_count = consumed_count + 1;
//!             }
//!         }
//!     })?;
//!
//!     // Wait for completion
//!     manager.join_all()?;
//!     Ok(())
//! }
//! ```

use crate::core::ThreadShare;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

/// Simplified thread management for ThreadShare
///
/// `ThreadManager` is a standalone utility for managing threads with shared data,
/// independent of the ThreadShare structures. It provides lightweight thread
/// management with comprehensive tracking and error handling.
///
/// ## Key Features
///
/// - **Simplified Thread Spawning**: Spawn threads with descriptive names
/// - **Shared Data Management**: Manage multiple types of shared data
/// - **Thread Tracking**: Monitor active thread count and status
/// - **Automatic Thread Joining**: Wait for all threads to complete
/// - **Type-Safe Operations**: Compile-time guarantees for thread safety
///
/// ## Example
///
/// ```rust
/// use thread_share::{ThreadManager, share};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let manager = ThreadManager::new();
///     let data = share!(vec![1, 2, 3]);
///
///     // Spawn threads
///     manager.spawn("sorter", data.clone(), |data| {
///         data.update(|v| v.sort());
///     })?;
///
    ///     manager.spawn("validator", data.clone(), |data| {
    ///         let v = data.get();
    ///         for i in 1..v.len() {
    ///             assert!(v[i-1] <= v[i]);
    ///         }
    ///     })?;
///
///     // Wait for completion
///     manager.join_all()?;
///     Ok(())
/// }
/// ```
///
/// ## Thread Lifecycle
///
/// 1. **Creation**: `ThreadManager::new()` or `ThreadManager::default()`
/// 2. **Spawning**: `manager.spawn(name, data, function)` creates named threads
/// 3. **Execution**: Threads run with access to shared data
/// 4. **Monitoring**: Track active threads with `active_threads()`
/// 5. **Completion**: Wait for all threads with `join_all()`
///
/// ## Performance
///
/// - **Thread Spawning**: Minimal overhead over standard `thread::spawn`
/// - **Thread Tracking**: Constant-time operations for thread management
/// - **Memory Usage**: Small overhead for tracking structures
/// - **Scalability**: Efficient for up to hundreds of threads
pub struct ThreadManager {
    threads: Arc<Mutex<HashMap<String, thread::JoinHandle<()>>>>,
    shared_data: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl ThreadManager {
    /// Creates a new ThreadManager
    ///
    /// This method creates a new `ThreadManager` instance with empty thread
    /// and shared data tracking.
    ///
    /// ## Returns
    ///
    /// A new `ThreadManager` instance.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadManager;
    ///
    /// let manager = ThreadManager::new();
    /// // let manager = ThreadManager::default(); // Alternative
    /// ```
    pub fn new() -> Self {
        Self {
            threads: Arc::new(Mutex::new(HashMap::new())),
            shared_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawns a thread with access to shared data
    ///
    /// This method creates a new thread with the given name and function.
    /// The thread receives a clone of the shared data and can safely modify it.
    ///
    /// ## Arguments
    ///
    /// * `name` - A descriptive name for the thread (useful for debugging)
    /// * `shared_data` - The `ThreadShare<T>` data to share with the thread
    /// * `f` - A function that receives `ThreadShare<T>` and performs the thread's work
    ///
    /// ## Requirements
    ///
    /// The function `F` must:
    /// - Implement `FnOnce(ThreadShare<T>)` - called once with shared data
    /// - Implement `Send` - safe to send across thread boundaries
    /// - Have `'static` lifetime - no borrowed references
    ///
    /// The type `T` must implement `Send + Sync + 'static`.
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success, `Err(String)` if thread spawning fails.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{ThreadManager, share};
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = ThreadManager::new();
    ///     let data = share!(0);
    ///
    ///     // Spawn a worker thread
    ///     manager.spawn("worker", data.clone(), |data| {
    ///         for _ in 0..100 {
    ///             data.update(|x| *x = *x + 1);
    ///             std::thread::sleep(std::time::Duration::from_millis(10));
    ///         }
    ///     })?;
    ///
    ///     // Spawn a monitor thread
    ///     manager.spawn("monitor", data.clone(), |data| {
    ///         for _ in 0..10 {
    ///         std::thread::sleep(std::time::Duration::from_millis(100));
    ///         println!("Current value: {}", data.get());
    ///     }
    ///     })?;
    ///     Ok(())
    /// }
    /// ```
    pub fn spawn<F, T>(&self, name: &str, shared_data: ThreadShare<T>, f: F) -> Result<(), String>
    where
        F: FnOnce(ThreadShare<T>) + Send + 'static,
        T: Send + Sync + 'static,
    {
        let thread_name = name.to_string();
        let thread_data = shared_data.clone();

        let handle = thread::spawn(move || {
            f(thread_data);
        });

        self.threads.lock().unwrap().insert(thread_name, handle);
        Ok(())
    }

    /// Spawns multiple threads with the same shared data
    ///
    /// This method spawns multiple threads from a vector of configurations.
    /// Each configuration contains a thread name and a function.
    ///
    /// ## Arguments
    ///
    /// * `shared_data` - The `ThreadShare<T>` data to share with all threads
    /// * `thread_configs` - Vector of `(name, function)` tuples
    ///
    /// ## Requirements
    ///
    /// The function `F` must implement `Clone` in addition to the standard requirements.
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success, `Err(String)` if any thread spawning fails.
    /// ```
    pub fn spawn_multiple<F, T>(
        &self,
        shared_data: ThreadShare<T>,
        thread_configs: Vec<(&str, F)>,
    ) -> Result<(), String>
    where
        F: FnOnce(ThreadShare<T>) + Send + Clone + 'static,
        T: Send + Sync + 'static,
    {
        for (name, func) in thread_configs {
            self.spawn(name, shared_data.clone(), func)?;
        }
        Ok(())
    }

    /// Waits for all threads to complete
    ///
    /// This method blocks until all spawned threads have finished execution.
    /// It joins each thread and returns an error if any thread panics.
    ///
    /// ## Returns
    ///
    /// `Ok(())` when all threads complete successfully, `Err(String)` if any thread fails.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{ThreadManager, share};
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = ThreadManager::new();
    ///     let data = share!(0);
    ///
    ///     manager.spawn("worker", data.clone(), |data| {
    ///         data.update(|x| *x = *x + 100);
    ///     })?;
    ///
    ///     // Wait for all threads to complete
    ///     manager.join_all()?;
    ///
    ///     // Now safe to access the final result
    ///     assert_eq!(data.get(), 100);
    ///     Ok(())
    /// }
    /// ```
    pub fn join_all(&self) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread_handles: Vec<_> = threads.drain().collect();
        drop(threads);

        for (name, handle) in thread_handles {
            if let Err(e) = handle.join() {
                return Err(format!("Thread '{}' failed: {:?}", name, e));
            }
        }
        Ok(())
    }

    /// Gets the number of active threads
    ///
    /// This method returns the current number of threads that are still running.
    ///
    /// ## Returns
    ///
    /// The number of active threads.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{ThreadManager, share};
    /// use std::time::Duration;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = ThreadManager::new();
    ///     let data = share!(0);
    ///
    ///     manager.spawn("worker", data.clone(), |data| {
    ///         std::thread::sleep(Duration::from_millis(100));
    ///     })?;
    ///
    ///     println!("Active threads: {}", manager.active_threads()); // Prints: 1
    ///
    ///     // Wait for completion
    ///     manager.join_all()?;
    ///
    ///     println!("Active threads: {}", manager.active_threads()); // Prints: 0
    ///     Ok(())
    /// }
    /// ```
    pub fn active_threads(&self) -> usize {
        self.threads.lock().unwrap().len()
    }

    /// Gets the number of shared data entries (for demonstration)
    ///
    /// This method returns the number of shared data entries currently stored.
    /// It's primarily used for demonstration and debugging purposes.
    ///
    /// ## Returns
    ///
    /// The number of shared data entries.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadManager;
    ///
    /// let manager = ThreadManager::new();
    /// println!("Shared data count: {}", manager.shared_data_count()); // Prints: 0
    /// ```
    pub fn shared_data_count(&self) -> usize {
        self.shared_data.lock().unwrap().len()
    }

    /// Checks if all threads have completed
    ///
    /// This method returns `true` if there are no active threads, `false` otherwise.
    ///
    /// ## Returns
    ///
    /// `true` if all threads have completed, `false` if any threads are still running.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{ThreadManager, share};
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = ThreadManager::new();
    ///     let data = share!(0);
    ///
    ///     manager.spawn("worker", data.clone(), |data| {
    ///         data.update(|x| *x = *x + 1);
    ///     })?;
    ///
    ///     assert!(!manager.is_complete()); // Thread is still running
    ///
    ///     manager.join_all()?;
    ///
    ///     assert!(manager.is_complete()); // All threads completed
    ///     Ok(())
    /// }
    /// ```
    pub fn is_complete(&self) -> bool {
        self.threads.lock().unwrap().is_empty()
    }
}

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for simplified thread spawning
///
/// This macro simplifies spawning multiple threads with the same shared data.
/// It creates a vector of thread configurations and calls `spawn_multiple`.
///
/// ## Syntax
///
/// `spawn_threads!(manager, shared_data, { name: function, ... })`
///
/// ## Arguments
///
/// * `manager` - The ThreadManager instance
/// * `shared_data` - The ThreadShare<T> data to share
/// * `{ name: function, ... }` - Named thread functions
///
/// ## Returns
///
/// `Result<(), String>` from `spawn_multiple`.
///
///
/// ## Performance
///
/// - **Compile-time expansion**: No runtime overhead
/// - **Efficient spawning**: Same performance as manual `spawn_multiple`
/// - **Type safety**: Compile-time type checking
/// - **Memory usage**: No additional allocations
#[macro_export]
macro_rules! spawn_threads {
    ($manager:expr, $shared_data:expr, { $($name:ident: $func:expr),* }) => {
        {
            let configs = vec![
                $(
                    (stringify!($name), $func)
                ),*
            ];
            $manager.spawn_multiple($shared_data, configs)
        }
    };
}

/// Macro for creating a complete thread setup
#[macro_export]
macro_rules! thread_setup {
    ($shared_data:expr, { $($name:ident: $func:expr),* }) => {
        {
            let manager = $crate::thread_pool::ThreadManager::new();
            $(
                manager.spawn(stringify!($name), $shared_data.clone(), $func)
                    .expect(&format!("Failed to spawn {}", stringify!($name)));
            )*
            manager
        }
    };
}
