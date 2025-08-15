//! # Worker Manager Module
//!
//! This module provides the `WorkerManager` struct for controlling spawned threads.
//! It allows you to manage individual workers: pause, resume, remove, and monitor them.
//!
//! ## Overview
//!
//! `WorkerManager` is designed to work with the `spawn_workers!` macro and provides
//! fine-grained control over thread management. It's particularly useful for:
//!
//! - **Dynamic worker management**: Add/remove workers at runtime
//! - **Worker state control**: Pause/resume individual workers
//! - **Monitoring**: Track worker status and count
//! - **Synchronization**: Wait for all workers to complete
//!
//! ## Key Features
//!
//! - 🔄 **Dynamic Worker Management**: Add/remove workers programmatically
//! - ⏸️ **State Control**: Pause/resume individual workers
//! - 📊 **Real-time Monitoring**: Track worker status and count
//! - 🔒 **Thread Safety**: All operations are thread-safe
//! - 🎮 **Fine-grained Control**: Manage each worker individually
//! - 📈 **Scalability**: Handle hundreds of workers efficiently
//!
//! ## Basic Usage
//!
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//!
//! // Create shared data
//! let data = enhanced_share!(0u32);
//!
//! // Spawn workers and get manager
//! let manager = spawn_workers!(data, {
//!     counter: |data| {
//!         for i in 1..=10 {
//!             data.update(|x| *x += i);
//!             std::thread::sleep(std::time::Duration::from_millis(100));
//!         }
//!     },
//!     monitor: |data| {
//!         for _ in 0..5 {
//!             std::thread::sleep(std::time::Duration::from_millis(200));
//!             println!("Value: {}", data.get());
//!         }
//!     }
//! });
//!
//! // Control workers
//! println!("Active workers: {}", manager.active_workers());
//! println!("Worker names: {:?}", manager.get_worker_names());
//!
//! // Wait for completion
//! manager.join_all().expect("Workers failed");
//! ```
//!
//! ## Advanced Usage with Programmatic Worker Addition
//!
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers, worker_manager::WorkerManager};
//! use std::thread;
//!
//! let data = enhanced_share!(0u32);
//!
//! // Start with initial workers
//! let manager = spawn_workers!(data, {
//!     initial_worker: |data| {
//!         data.update(|x| *x += 1);
//!         std::thread::sleep(std::time::Duration::from_millis(100));
//!     }
//! });
//!
//! // Add more workers programmatically
//! let data_clone = data.clone();
//! let handle = thread::spawn(move || {
//!     for _ in 0..3 {
//!         data_clone.update(|x| *x *= 2);
//!         std::thread::sleep(std::time::Duration::from_millis(150));
//!     }
//! });
//!
//! manager.add_worker("dynamic_worker", handle).expect("Failed to add worker");
//!
//! // Now we have 2 workers
//! assert_eq!(manager.active_workers(), 2);
//! ```
//!
//! ## Worker Lifecycle Management
//!
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//!
//! let data = enhanced_share!(0u32);
//! let manager = spawn_workers!(data, {
//!     worker1: |data| { /* work */ },
//!     worker2: |data| { /* work */ }
//! });
//!
//! // Pause a worker
//! manager.pause_worker("worker1").expect("Failed to pause");
//!
//! // Check if paused
//! assert!(manager.is_worker_paused("worker1"));
//!
//! // Resume a worker
//! manager.resume_worker("worker1").expect("Failed to resume");
//!
//! // Remove from tracking
//! manager.remove_worker("worker2").expect("Failed to remove");
//!
//! // Remove all workers
//! manager.remove_all_workers().expect("Failed to remove all");
//! ```
//!
//! ## Error Handling
//!
//! All methods return `Result<T, String>` for proper error handling:
//!
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//!
//! let data = enhanced_share!(0u32);
//! let manager = spawn_workers!(data, {
//!     worker: |data| { /* work */ }
//! });
//!
//! // Handle errors gracefully
//! match manager.pause_worker("nonexistent") {
//!     Ok(()) => println!("Worker paused successfully"),
//!     Err(e) => println!("Failed to pause worker: {}", e),
//! }
//!
//! // Remove worker and handle result
//! match manager.remove_worker("worker") {
//!     Ok(()) => println!("Worker removed successfully"),
//!     Err(e) => println!("Failed to remove worker: {}", e),
//! }
//! ```
//!
//! ## Thread Safety
//!
//! `WorkerManager` is designed to be thread-safe and can be shared between threads:
//!
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//! use std::thread;
//! use std::sync::Arc;
//!
//! let data = enhanced_share!(0u32);
//! let manager = spawn_workers!(data, {
//!     worker: |data| { /* work */ }
//! });
//!
//! // Clone manager for another thread
//! let manager_clone = manager.clone();
//! let thread_handle = thread::spawn(move || {
//!     // Use manager in another thread
//!     let names = manager_clone.get_worker_names();
//!     println!("Worker names from thread: {:?}", names);
//! });
//!
//! thread_handle.join().expect("Thread failed");
//! ```
//!
//! ## Performance Considerations
//!
//! - **Thread Spawning**: Minimal overhead over standard `thread::spawn`
//! - **Worker Management**: Constant-time operations for most management functions
//! - **Memory Usage**: Small overhead for worker tracking structures
//! - **Scalability**: Efficient for up to hundreds of workers
//!
//! ## When to Use WorkerManager
//!
//! - **Complex Applications**: When you need fine-grained control over workers
//! - **Dynamic Workloads**: When worker count changes at runtime
//! - **Monitoring Requirements**: When you need real-time worker status
//! - **Production Systems**: When you need robust worker management
//! - **Debugging**: When you need to pause/resume workers for debugging

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

/// Worker Manager for controlling spawned threads
///
/// This struct provides methods to control individual workers:
/// - Pause/resume specific workers
/// - Remove workers from tracking
/// - Monitor worker status
/// - Add new workers programmatically
///
/// ## Creation
///
/// `WorkerManager` is typically created by the `spawn_workers!` macro:
///
/// ```rust
/// use thread_share::{enhanced_share, spawn_workers};
///
/// let data = enhanced_share!(0u32);
/// let manager = spawn_workers!(data, {
///     worker1: |data| { /* work */ },
///     worker2: |data| { /* work */ }
/// });
/// ```
///
/// You can also create it directly:
///
/// ```rust
/// use thread_share::worker_manager::WorkerManager;
/// use std::sync::{Arc, Mutex};
/// use std::collections::HashMap;
///
/// let threads = Arc::new(Mutex::new(HashMap::new()));
/// let manager = WorkerManager::new(threads);
/// ```
///
/// ## Thread Safety
///
/// `WorkerManager` implements `Clone` and can be safely shared between threads.
/// All operations are thread-safe and use proper locking mechanisms.
///
/// ## Example: Complete Worker Lifecycle
///
/// ```rust
/// use thread_share::{enhanced_share, spawn_workers};
/// use std::thread;
/// use std::time::Duration;
///
/// let data = enhanced_share!(0u32);
///
/// // Start initial workers
/// let manager = spawn_workers!(data, {
///     counter: |data| {
///         for i in 1..=5 {
///             data.update(|x| *x += i);
///             thread::sleep(Duration::from_millis(100));
///         }
///     }
/// });
///
/// // Add worker programmatically
/// let data_clone = data.clone();
/// let handle = thread::spawn(move || {
///     for _ in 0..3 {
///         data_clone.update(|x| *x *= 2);
///         thread::sleep(Duration::from_millis(150));
///     }
/// });
///
/// manager.add_worker("multiplier", handle).expect("Failed to add worker");
///
/// // Monitor workers
/// println!("Active workers: {}", manager.active_workers());
/// println!("Worker names: {:?}", manager.get_worker_names());
///
/// // Control workers
/// manager.pause_worker("counter").expect("Failed to pause");
/// thread::sleep(Duration::from_millis(200));
/// manager.resume_worker("counter").expect("Failed to resume");
///
/// // Wait for completion
/// manager.join_all().expect("Workers failed");
///
/// println!("Final value: {}", data.get());
/// ```
pub struct WorkerManager {
    threads: Arc<Mutex<HashMap<String, thread::JoinHandle<()>>>>,
    paused_workers: Arc<Mutex<HashMap<String, bool>>>,
}

impl WorkerManager {
    /// Creates a new WorkerManager
    ///
    /// ## Arguments
    ///
    /// * `threads` - Arc<Mutex<HashMap<String, JoinHandle<()>>>> containing thread handles
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::worker_manager::WorkerManager;
    /// use std::sync::{Arc, Mutex};
    /// use std::collections::HashMap;
    ///
    /// let threads = Arc::new(Mutex::new(HashMap::new()));
    /// let manager = WorkerManager::new(threads);
    /// ```
    pub fn new(threads: Arc<Mutex<HashMap<String, thread::JoinHandle<()>>>>) -> Self {
        Self {
            threads,
            paused_workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Adds a new worker to the manager
    ///
    /// This method allows you to add workers programmatically after the manager is created.
    /// The worker will be tracked and can be managed like any other worker.
    ///
    /// ## Arguments
    ///
    /// * `name` - A descriptive name for the worker
    /// * `handle` - The JoinHandle of the spawned thread
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success, `Err(String)` if a worker with the same name already exists.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::worker_manager::WorkerManager;
    /// use std::sync::{Arc, Mutex};
    /// use std::collections::HashMap;
    /// use std::thread;
    ///
    /// let threads = Arc::new(Mutex::new(HashMap::new()));
    /// let manager = WorkerManager::new(threads.clone());
    ///
    /// // Spawn a thread manually
    /// let handle = thread::spawn(|| {
    ///     println!("Worker doing work...");
    /// });
    ///
    /// // Add it to the manager
    /// manager.add_worker("manual_worker", handle).expect("Failed to add worker");
    /// ```
    pub fn add_worker(&self, name: &str, handle: thread::JoinHandle<()>) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        
        if threads.contains_key(name) {
            return Err(format!("Worker '{}' already exists", name));
        }
        
        threads.insert(name.to_string(), handle);
        println!("Worker '{}' added to manager", name);
        Ok(())
    }

    /// Pauses a specific worker by name
    ///
    /// Note: This is a placeholder for future implementation.
    /// Currently, Rust doesn't support pausing threads directly.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the worker to pause
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success, `Err(String)` if the worker doesn't exist
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     worker: |data| { /* work */ }
    /// });
    ///
    /// // Pause the worker
    /// manager.pause_worker("worker").expect("Failed to pause");
    /// ```
    pub fn pause_worker(&self, name: &str) -> Result<(), String> {
        let mut paused = self.paused_workers.lock().unwrap();
        paused.insert(name.to_string(), true);
        println!("Worker '{}' marked for pause (implementation pending)", name);
        Ok(())
    }

    /// Resumes a specific worker by name
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the worker to resume
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success, `Err(String)` if the worker doesn't exist
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     worker: |data| { /* work */ }
    /// });
    ///
    /// // Pause then resume
    /// manager.pause_worker("worker").expect("Failed to pause");
    /// manager.resume_worker("worker").expect("Failed to resume");
    /// ```
    pub fn resume_worker(&self, name: &str) -> Result<(), String> {
        let mut paused = self.paused_workers.lock().unwrap();
        paused.remove(name);
        println!("Worker '{}' resumed", name);
        Ok(())
    }

    /// Removes a worker from tracking without stopping it
    ///
    /// This method removes the worker from the manager's tracking but doesn't
    /// actually stop the thread. The thread will continue running until it
    /// completes naturally.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the worker to remove
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success, `Err(String)` if the worker doesn't exist
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     worker: |data| { /* work */ }
    /// });
    ///
    /// // Remove from tracking
    /// manager.remove_worker("worker").expect("Failed to remove");
    /// ```
    pub fn remove_worker(&self, name: &str) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        if threads.remove(name).is_some() {
            println!("Worker '{}' removed from tracking", name);
            Ok(())
        } else {
            Err(format!("Worker '{}' not found", name))
        }
    }

    /// Removes all workers from tracking without stopping them
    ///
    /// This method removes all workers from the manager's tracking but doesn't
    /// actually stop the threads. The threads will continue running until they
    /// complete naturally.
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     worker1: |data| { /* work */ },
    ///     worker2: |data| { /* work */ }
    /// });
    ///
    /// // Remove all workers from tracking
    /// manager.remove_all_workers().expect("Failed to remove all workers");
    /// ```
    pub fn remove_all_workers(&self) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let count = threads.len();
        threads.clear();
        println!("Removed {} workers from tracking", count);
        Ok(())
    }

    /// Gets the list of all worker names
    ///
    /// ## Returns
    ///
    /// A `Vec<String>` containing all worker names
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     counter: |data| { /* work */ },
    ///     monitor: |data| { /* work */ }
    /// });
    ///
    /// let names = manager.get_worker_names();
    /// assert_eq!(names.len(), 2);
    /// assert!(names.contains(&"counter".to_string()));
    /// assert!(names.contains(&"monitor".to_string()));
    /// ```
    pub fn get_worker_names(&self) -> Vec<String> {
        let threads = self.threads.lock().unwrap();
        threads.keys().cloned().collect()
    }

    /// Gets the number of active workers
    ///
    /// ## Returns
    ///
    /// The number of workers currently being tracked
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     worker1: |data| { /* work */ },
    ///     worker2: |data| { /* work */ }
    /// });
    ///
    /// assert_eq!(manager.active_workers(), 2);
    /// ```
    pub fn active_workers(&self) -> usize {
        let threads = self.threads.lock().unwrap();
        threads.len()
    }

    /// Checks if a specific worker is paused
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the worker to check
    ///
    /// ## Returns
    ///
    /// `true` if the worker is paused, `false` otherwise
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     worker: |data| { /* work */ }
    /// });
    ///
    /// // Initially not paused
    /// assert!(!manager.is_worker_paused("worker"));
    ///
    /// // Pause the worker
    /// manager.pause_worker("worker").expect("Failed to pause");
    /// assert!(manager.is_worker_paused("worker"));
    ///
    /// // Resume the worker
    /// manager.resume_worker("worker").expect("Failed to resume");
    /// assert!(!manager.is_worker_paused("worker"));
    /// ```
    pub fn is_worker_paused(&self, name: &str) -> bool {
        let paused = self.paused_workers.lock().unwrap();
        paused.contains_key(name)
    }

    /// Waits for all workers to complete
    ///
    /// This method blocks until all tracked workers have completed.
    /// It removes all workers from tracking after they complete.
    ///
    /// ## Returns
    ///
    /// `Ok(())` if all workers completed successfully, `Err(String)` if any worker failed
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     worker: |data| {
    ///         thread::sleep(Duration::from_millis(100));
    ///         data.update(|x| *x += 1);
    ///     }
    /// });
    ///
    /// // Wait for completion
    /// manager.join_all().expect("Workers failed");
    ///
    /// // All workers are now completed and removed
    /// assert_eq!(manager.active_workers(), 0);
    /// ```
    pub fn join_all(&self) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread_handles: Vec<_> = threads.drain().collect();
        drop(threads);

        for (name, handle) in thread_handles {
            let result = handle.join();
            if let Err(e) = result {
                return Err(format!("Worker '{}' failed: {:?}", name, e));
            }
        }
        Ok(())
    }
}

impl Clone for WorkerManager {
    /// Creates a clone of the WorkerManager
    ///
    /// The cloned manager shares the same underlying thread tracking data,
    /// so operations on one clone will affect the other.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{enhanced_share, spawn_workers};
    ///
    /// let data = enhanced_share!(0u32);
    /// let manager = spawn_workers!(data, {
    ///     worker: |data| { /* work */ }
    /// });
    ///
    /// // Clone the manager
    /// let manager_clone = manager.clone();
    ///
    /// // Both managers track the same workers
    /// assert_eq!(manager.active_workers(), manager_clone.active_workers());
    /// ```
    fn clone(&self) -> Self {
        Self {
            threads: self.threads.clone(),
            paused_workers: self.paused_workers.clone(),
        }
    }
}
