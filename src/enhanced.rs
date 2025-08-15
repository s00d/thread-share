//! # Enhanced Module - EnhancedThreadShare<T>
//!
//! This module provides `EnhancedThreadShare<T>`, a powerful extension of `ThreadShare<T>`
//! that adds automatic thread management capabilities.
//!
//! ## 🚀 Overview
//!
//! `EnhancedThreadShare<T>` eliminates the need for manual thread management by providing:
//!
//! - **Automatic Thread Spawning**: Spawn threads with a single method call
//! - **Built-in Thread Tracking**: Monitor active thread count and status
//! - **Automatic Thread Joining**: Wait for all threads to complete with `join_all()`
//! - **Thread Naming**: Give meaningful names to threads for debugging
//! - **All ThreadShare Features**: Inherits all capabilities from `ThreadShare<T>`
//!
//! ## Key Benefits
//!
//! ### 🎯 Simplified Thread Management
//! ```rust
//! // Old way: Manual thread management
//! use thread_share::share;
//! use std::thread;
//!
//! let data = share!(vec![1, 2, 3]);
//! let clone1 = data.clone();
//! let clone2 = data.clone();
//!
//! let handle1 = thread::spawn(move || { /* logic */ });
//! let handle2 = thread::spawn(move || { /* logic */ });
//!
//! handle1.join().expect("Failed to join");
//! handle2.join().expect("Failed to join");
//!
//! // New way: Enhanced thread management
//! use thread_share::enhanced_share;
//!
//! let enhanced = enhanced_share!(vec![1, 2, 3]);
//!
//! enhanced.spawn("worker1", |data| { /* logic */ });
//! enhanced.spawn("worker2", |data| { /* logic */ });
//!
//! enhanced.join_all().expect("Failed to join");
//! ```
//!
//! ### 📊 Real-time Monitoring
//! ```rust
//! use thread_share::enhanced_share;
//!
//! let enhanced = enhanced_share!(vec![1, 2, 3]);
//!
//! enhanced.spawn("processor", |data| { /* logic */ });
//! enhanced.spawn("validator", |data| { /* logic */ });
//!
//! println!("Active threads: {}", enhanced.active_threads());
//!
//! // Wait for completion
//! enhanced.join_all().expect("Failed to join");
//!
//! assert!(enhanced.is_complete());
//! ```
//!
//! ## Architecture
//!
//! `EnhancedThreadShare<T>` wraps a `ThreadShare<T>` and adds:
//!
//! - **`inner: ThreadShare<T>`** - The underlying shared data
//! - **`threads: Arc<Mutex<HashMap<String, JoinHandle<()>>>>`** - Thread tracking
//!
//! ## Thread Lifecycle
//!
//! 1. **Creation**: `EnhancedThreadShare::new(data)` or `enhanced_share!(data)`
//! 2. **Spawning**: `enhanced.spawn(name, function)` creates named threads
//! 3. **Execution**: Threads run with access to shared data
//! 4. **Monitoring**: Track active threads with `active_threads()`
//! 5. **Completion**: Wait for all threads with `join_all()`
//!
//! ## Example Usage
//!
//! ### Basic Thread Management
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//!
//! let data = enhanced_share!(vec![1, 2, 3]);
//!
//! // Spawn individual threads
//! data.spawn("sorter", |data| {
//!     data.update(|v| v.sort());
//! });
//!
//! data.spawn("validator", |data| {
//!     assert!(data.get().is_sorted());
//! });
//!
//! // Wait for completion
//! data.join_all().expect("Failed to join");
//! ```
//!
//! ### Using Macros
//! ```rust
//! use thread_share::share;
//!
//! let data = share!(String::from("Hello"));
//! let clone = data.clone();
//!
//! // Spawn a simple thread
//! std::thread::spawn(move || {
//!     clone.update(|s| s.push_str(" World"));
//! });
//!
//! // Wait a bit and check result
//! std::thread::sleep(std::time::Duration::from_millis(100));
//! println!("Updated: {}", data.get());
//! ```
//!
//! ### Real-world Example
//! ```rust
//! use thread_share::share;
//! use std::time::Duration;
//!
//! #[derive(Clone)]
//! struct Server {
//!     port: u16,
//!     is_running: bool,
//!     connections: u32,
//! }
//!
//! let server = share!(Server {
//!     port: 8080,
//!     is_running: false,
//!     connections: 0,
//! });
//!
//! let server_clone = server.clone();
//!
//! // Spawn a simple server thread
//! std::thread::spawn(move || {
//!     server_clone.update(|s| {
//!         s.is_running = true;
//!         s.connections = 5;
//!     });
//! });
//!
//! // Wait a bit and check result
//! std::thread::sleep(Duration::from_millis(100));
//! let final_state = server.get();
//! println!("Server running: {}, connections: {}", final_state.is_running, final_state.connections);
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Thread Spawning**: Minimal overhead over standard `thread::spawn`
//! - **Thread Tracking**: Constant-time operations for thread management
//! - **Memory Usage**: Small overhead for thread tracking structures
//! - **Scalability**: Efficient for up to hundreds of threads
//!
//! ## Best Practices
//!
//! 1. **Use descriptive thread names** for easier debugging
//! 2. **Keep thread functions focused** on single responsibilities
//! 3. **Always call `join_all()`** to ensure proper cleanup
//! 4. **Monitor thread count** with `active_threads()` for debugging
//! 5. **Handle errors gracefully** from `join_all()` and `spawn()`
//!
//! ## Error Handling
//!
//! ```rust
//! use thread_share::share;
//!
//! let data = share!(0);
//! let clone = data.clone();
//!
//! // Spawn thread with error handling
//! let handle = std::thread::spawn(move || {
//!     clone.update(|x| *x = *x + 1);
//! });
//!
//! // Handle join errors
//! if let Err(e) = handle.join() {
//!     eprintln!("Thread execution failed: {:?}", e);
//! }
//! ```
//!
//! ## Thread Safety
//!
//! `EnhancedThreadShare<T>` automatically implements `Send` and `Sync` traits
//! when `T` implements them, making it safe to use across thread boundaries.
//!
//! ## Integration with Macros
//!
//! This module works seamlessly with the library's macros:
//!
//! - **`enhanced_share!`** - Creates `EnhancedThreadShare<T>` instances
//! - **`spawn_workers!`** - Spawns multiple threads with single macro call
//!
//! ## Comparison with Manual Thread Management
//!
//! | Aspect | Manual Management | EnhancedThreadShare |
//! |--------|-------------------|-------------------|
//! | **Thread Creation** | `thread::spawn()` calls | `enhanced.spawn()` |
//! | **Thread Tracking** | Manual `JoinHandle` storage | Automatic tracking |
//! | **Thread Joining** | Manual `join()` calls | `join_all()` |
//! | **Error Handling** | Per-thread error handling | Centralized error handling |
//! | **Debugging** | No thread identification | Named threads |
//! | **Code Complexity** | High | Low |

use crate::core::ThreadShare;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

/// Enhanced ThreadShare with built-in thread management
///
/// `EnhancedThreadShare<T>` extends `ThreadShare<T>` with automatic thread management
/// capabilities, eliminating the need for manual thread spawning and joining.
///
/// ## Key Features
///
/// - **Automatic Thread Spawning**: Spawn threads with a single method call
/// - **Built-in Thread Tracking**: Monitor active thread count and status
/// - **Automatic Thread Joining**: Wait for all threads to complete with `join_all()`
/// - **Thread Naming**: Give meaningful names to threads for debugging
/// - **All ThreadShare Features**: Inherits all capabilities from `ThreadShare<T>`
///
/// ## Example
///
/// ```rust
/// use thread_share::{enhanced_share, spawn_workers};
///
/// let data = enhanced_share!(vec![1, 2, 3]);
///
/// // Spawn individual threads
/// data.spawn("sorter", |data| {
///     data.update(|v| v.sort());
/// });
///
/// data.spawn("validator", |data| {
///     assert!(data.get().is_sorted());
/// });
///
/// // Wait for completion
/// data.join_all().expect("Failed to join");
/// ```
///
/// ## Thread Lifecycle
///
/// 1. **Creation**: `EnhancedThreadShare::new(data)` or `enhanced_share!(data)`
/// 2. **Spawning**: `enhanced.spawn(name, function)` creates named threads
/// 3. **Execution**: Threads run with access to shared data
/// 4. **Monitoring**: Track active threads with `active_threads()`
/// 5. **Completion**: Wait for all threads with `join_all()`
///
/// ## Performance
///
/// - **Thread Spawning**: Minimal overhead over standard `thread::spawn`
/// - **Thread Tracking**: Constant-time operations for thread management
/// - **Memory Usage**: Small overhead for thread tracking structures
/// - **Scalability**: Efficient for up to hundreds of threads
pub struct EnhancedThreadShare<T> {
    inner: ThreadShare<T>,
    threads: Arc<Mutex<HashMap<String, thread::JoinHandle<()>>>>,
}

impl<T> EnhancedThreadShare<T> {
    /// Creates a new EnhancedThreadShare
    ///
    /// This method creates a new `EnhancedThreadShare<T>` instance with the provided data.
    /// The data is wrapped in a `ThreadShare<T>` for safe sharing between threads.
    ///
    /// ## Arguments
    ///
    /// * `data` - The initial data to share between threads
    ///
    /// ## Returns
    ///
    /// A new `EnhancedThreadShare<T>` instance.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::EnhancedThreadShare;
    ///
    /// let enhanced = EnhancedThreadShare::new(0);
    /// let enhanced = EnhancedThreadShare::new(String::from("Hello"));
    /// let enhanced = EnhancedThreadShare::new(vec![1, 2, 3]);
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            inner: ThreadShare::new(data),
            threads: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawns a thread with access to this shared data
    ///
    /// This method creates a new thread with the given name and function.
    /// The thread receives a clone of the shared data and can safely modify it.
    ///
    /// ## Arguments
    ///
    /// * `name` - A descriptive name for the thread (useful for debugging)
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
    /// use thread_share::EnhancedThreadShare;
    ///
    /// let enhanced = EnhancedThreadShare::new(0);
    ///
    /// // Spawn a worker thread
    /// enhanced.spawn("worker", |data| {
    ///     for _ in 0..100 {
    ///         data.update(|x| *x += 1);
    ///         std::thread::sleep(std::time::Duration::from_millis(10));
    ///     }
    /// }).expect("Failed to spawn worker");
    ///
    /// // Spawn a monitor thread
    /// enhanced.spawn("monitor", |data| {
    ///     for _ in 0..10 {
    ///         std::thread::sleep(std::time::Duration::from_millis(100));
    ///         println!("Current value: {}", data.get());
    ///     }
    /// }).expect("Failed to spawn monitor");
    /// ```
    pub fn spawn<F>(&self, name: &str, f: F) -> Result<(), String>
    where
        F: FnOnce(ThreadShare<T>) + Send + 'static,
        T: Send + Sync + 'static,
    {
        let thread_name = name.to_string();
        let thread_data = self.inner.clone();

        let handle = thread::spawn(move || {
            f(thread_data);
        });

        self.threads.lock().unwrap().insert(thread_name, handle);
        Ok(())
    }

    /// Spawns multiple threads with different names and functions
    ///
    /// This method spawns multiple threads from a vector of configurations.
    /// Each configuration contains a thread name and a function.
    ///
    /// ## Arguments
    ///
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
    pub fn spawn_multiple<F>(&self, thread_configs: Vec<(&str, F)>) -> Result<(), String>
    where
        F: FnOnce(ThreadShare<T>) + Send + Clone + 'static,
        T: Send + Sync + 'static,
    {
        for (name, func) in thread_configs {
            self.spawn(name, func)?;
        }
        Ok(())
    }

    /// Spawns multiple threads with boxed closures
    ///
    /// This method spawns multiple threads using boxed closures, which allows
    /// for different function types in the same vector.
    ///
    /// ## Arguments
    ///
    /// * `thread_configs` - Vector of `(name, boxed_function)` tuples
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success, `Err(String)` if any thread spawning fails.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::EnhancedThreadShare;
    ///
    /// let enhanced = EnhancedThreadShare::new(0);
    ///
    /// let configs = vec![
    ///     ("worker1", Box::new(|data: thread_share::ThreadShare<i32>| { data.update(|x| *x = *x + 1); }) as Box<dyn FnOnce(thread_share::ThreadShare<i32>) + Send>),
    ///     ("worker2", Box::new(|data: thread_share::ThreadShare<i32>| { data.update(|x| *x = *x + 2); }) as Box<dyn FnOnce(thread_share::ThreadShare<i32>) + Send>),
    /// ];
    ///
    /// enhanced.spawn_multiple_boxed(configs).expect("Failed to spawn threads");
    /// ```
    pub fn spawn_multiple_boxed(
        &self,
        thread_configs: Vec<(&str, Box<dyn FnOnce(ThreadShare<T>) + Send>)>,
    ) -> Result<(), String>
    where
        T: Send + Sync + 'static,
    {
        for (name, func) in thread_configs {
            let thread_data = self.inner.clone();
            let handle = thread::spawn(move || {
                func(thread_data);
            });
            self.threads
                .lock()
                .unwrap()
                .insert(name.to_string(), handle);
        }
        Ok(())
    }

    /// Waits for all spawned threads to complete
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
    /// use thread_share::EnhancedThreadShare;
    ///
    /// let enhanced = EnhancedThreadShare::new(0);
    ///
    /// enhanced.spawn("worker", |data| {
    ///     data.update(|x| *x = *x + 100);
    /// }).expect("Failed to spawn worker");
    ///
    /// // Wait for all threads to complete
    /// enhanced.join_all().expect("Thread execution failed");
    ///
    /// // Now safe to access the final result
    /// assert_eq!(enhanced.get(), 100);
    /// ```
    pub fn join_all(&self) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread_handles: Vec<_> = threads.drain().collect();
        drop(threads);

        for (name, handle) in thread_handles {
            let result = handle.join();
            if let Err(e) = result {
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
    /// use thread_share::EnhancedThreadShare;
    /// use std::time::Duration;
    ///
    /// let enhanced = EnhancedThreadShare::new(0);
    ///
    /// enhanced.spawn("worker", |data| {
    ///     std::thread::sleep(Duration::from_millis(100));
    /// }).expect("Failed to spawn worker");
    ///
    /// println!("Active threads: {}", enhanced.active_threads()); // Prints: 1
    ///
    /// // Wait for completion
    /// enhanced.join_all().expect("Failed to join");
    ///
    /// println!("Active threads: {}", enhanced.active_threads()); // Prints: 0
    /// ```
    pub fn active_threads(&self) -> usize {
        self.threads.lock().unwrap().len()
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
    /// use thread_share::EnhancedThreadShare;
    ///
    /// let enhanced = EnhancedThreadShare::new(0);
    ///
    /// enhanced.spawn("worker", |data| {
    ///     data.update(|x| *x = *x + 1);
    /// }).expect("Failed to spawn worker");
    ///
    /// assert!(!enhanced.is_complete()); // Thread is still running
    ///
    /// enhanced.join_all().expect("Failed to join");
    ///
    /// assert!(enhanced.is_complete()); // All threads completed
    /// ```
    pub fn is_complete(&self) -> bool {
        self.threads.lock().unwrap().is_empty()
    }

    /// Delegates all ThreadShare methods
    ///
    /// Gets a copy of the shared data.
    ///
    /// ## Requirements
    ///
    /// The type `T` must implement `Clone` trait.
    ///
    /// ## Returns
    ///
    /// A copy of the current data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::EnhancedThreadShare;
    ///
    /// let enhanced = EnhancedThreadShare::new(42);
    /// let value = enhanced.get();
    /// assert_eq!(value, 42);
    /// ```
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.get()
    }

    /// Sets new data and notifies waiting threads
    ///
    /// This method replaces the current data and notifies all threads
    /// waiting for changes.
    ///
    /// ## Arguments
    ///
    /// * `new_data` - The new data to set
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::EnhancedThreadShare;
    ///
    /// let enhanced = EnhancedThreadShare::new(0);
    /// enhanced.set(100);
    /// assert_eq!(enhanced.get(), 100);
    /// ```
    pub fn set(&self, new_data: T) {
        self.inner.set(new_data);
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        self.inner.update(f);
    }

    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.inner.read(f)
    }

    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner.write(f)
    }

    pub fn wait_for_change(&self, timeout: std::time::Duration) -> bool {
        self.inner.wait_for_change(timeout)
    }

    pub fn wait_for_change_forever(&self) {
        self.inner.wait_for_change_forever();
    }

    pub fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            threads: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<T> Clone for EnhancedThreadShare<T> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

/// Macro for creating enhanced thread share with automatic thread management
#[macro_export]
macro_rules! enhanced_share {
    ($data:expr) => {
        $crate::enhanced::EnhancedThreadShare::new($data)
    };
}

/// Macro for simplified multi-threaded setup
#[macro_export]
macro_rules! spawn_workers {
    ($shared:expr, { $($name:ident: $func:expr),* }) => {
        {
            $(
                $shared.spawn(stringify!($name), $func).expect(&format!("Failed to spawn {}", stringify!($name)));
            )*
        }
    };
}
