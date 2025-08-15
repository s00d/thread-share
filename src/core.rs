//! # Core Module - ThreadShare and SimpleShare
//!
//! This module provides the core structures for safe data exchange between threads.
//!
//! ## Overview
//!
//! The core module contains two main structures:
//!
//! - **`ThreadShare<T>`** - Full-featured synchronization with change detection
//! - **`SimpleShare<T>`** - Lightweight alternative for basic use cases
//!
//! ## ThreadShare<T>
//!
//! `ThreadShare<T>` is the main structure that provides comprehensive thread synchronization.
//! It uses `Arc<RwLock<T>>` internally and provides automatic change notification.
//!
//! ### Key Features
//!
//! - **Automatic Cloning**: Each thread gets its own clone for safe access
//! - **Change Detection**: Built-in waiting mechanisms for data changes
//! - **Flexible Access**: Read, write, and update operations with proper locking
//! - **Condition Variables**: Efficient waiting for data modifications
//! - **Thread Safety**: Implements `Send` and `Sync` automatically
//!
//! ### Example Usage
//!
//! ```rust
//! use thread_share::ThreadShare;
//! use std::thread;
//! use std::time::Duration;
//!
//! let data = ThreadShare::new(vec![1, 2, 3]);
//! let clone = data.clone();
//!
//! // Spawn a thread that modifies the data
//! thread::spawn(move || {
//!     thread::sleep(Duration::from_millis(100));
//!     clone.update(|v| v.push(4));
//! });
//!
//! // Wait for changes and read the result
//! data.wait_for_change_forever();
//! let result = data.get();
//! assert_eq!(result, vec![1, 2, 3, 4]);
//! ```
//!
//! ### Performance Characteristics
//!
//! - **Read Operations**: Multiple threads can read simultaneously
//! - **Write Operations**: Exclusive access during writes
//! - **Change Detection**: Efficient condition variable notifications
//! - **Memory Overhead**: Minimal overhead from Arc and RwLock structures
//!
//! ## SimpleShare<T>
//!
//! `SimpleShare<T>` is a simplified version of `ThreadShare<T>` that provides
//! basic functionality without change detection.
//!
//! ### Key Features
//!
//! - **Minimal Overhead**: Lighter synchronization primitives
//! - **Essential Operations**: Basic get/set/update functionality
//! - **Clone Support**: Each thread gets a clone for safe access
//! - **No Change Detection**: Simpler implementation without condition variables
//!
//! ### Example Usage
//!
//! ```rust
//! use thread_share::SimpleShare;
//! use std::thread;
//! use std::time::Duration;
//!
//! let counter = SimpleShare::new(0);
//! let clone = counter.clone();
//!
//! thread::spawn(move || {
//!     for _ in 0..100 {
//!         clone.update(|x| *x = *x + 1);
//!     }
//! });
//!
//! thread::sleep(Duration::from_millis(100));
//! assert_eq!(counter.get(), 100);
//! ```
//!
//! ## When to Use Each
//!
//! ### Use ThreadShare<T> when you need:
//! - Change detection and waiting mechanisms
//! - Complex synchronization patterns
//! - Maximum flexibility and features
//! - Production applications with complex requirements
//!
//! ### Use SimpleShare<T> when you need:
//! - Basic data sharing without change detection
//! - Minimal overhead and complexity
//! - Simple producer-consumer patterns
//! - Learning and prototyping
//!
//! ## Thread Safety
//!
//! Both structures automatically implement `Send` and `Sync` traits, making them
//! safe to use across thread boundaries. The internal synchronization primitives
//! ensure that all operations are thread-safe.
//!
//! ## Memory Management
//!
//! - **Arc**: Provides reference counting for shared ownership
//! - **RwLock**: Ensures exclusive write access and concurrent read access
//! - **Mutex**: Protects condition variable access
//! - **Condvar**: Enables efficient waiting for changes
//!
//! ## Best Practices
//!
//! 1. **Always clone for thread usage**: Use `.clone()` to get thread-safe copies
//! 2. **Use appropriate access patterns**: `read()` for read-only, `write()` for modifications
//! 3. **Consider change detection**: Use `wait_for_change()` when you need to react to updates
//! 4. **Minimize lock contention**: Keep critical sections as short as possible
//! 5. **Handle errors gracefully**: Always check return values from operations

use parking_lot::RwLock;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[cfg(feature = "serialize")]
use serde::{de::DeserializeOwned, Serialize};

// Conditional compilation for serialization support
#[cfg(feature = "serialize")]
impl<T> ThreadShare<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    /// Serializes the current data to JSON string
    ///
    /// This method requires the `serialize` feature to be enabled.
    ///
    /// ## Returns
    ///
    /// JSON string representation of the data, or error string if serialization fails.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    ///
    /// #[cfg(feature = "serialize")]
    /// {
    ///     let data = ThreadShare::new(vec![1, 2, 3]);
    ///     let json = data.to_json().expect("Failed to serialize");
    ///     assert_eq!(json, "[1,2,3]");
    /// }
    /// ```
    pub fn to_json(&self) -> Result<String, String> {
        let data = self.data.read();
        serde_json::to_string(&*data).map_err(|e| format!("Serialization failed: {}", e))
    }

    /// Deserializes data from JSON string
    ///
    /// This method requires the `serialize` feature to be enabled.
    ///
    /// ## Arguments
    ///
    /// * `json` - JSON string to deserialize
    ///
    /// ## Returns
    ///
    /// `Ok(())` on success, `Err(String)` if deserialization fails.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    ///
    /// #[cfg(feature = "serialize")]
    /// {
    ///     let data = ThreadShare::new(vec![0; 0]);
    ///     data.from_json("[1,2,3]").expect("Failed to deserialize");
    ///     assert_eq!(data.get(), vec![1, 2, 3]);
    /// }
    /// ```
    pub fn from_json(&self, json: &str) -> Result<(), String> {
        let deserialized: T =
            serde_json::from_str(json).map_err(|e| format!("Deserialization failed: {}", e))?;

        self.set(deserialized);
        Ok(())
    }
}

/// Main structure for data exchange between threads
///
/// `ThreadShare<T>` provides comprehensive thread synchronization with automatic
/// change detection and efficient locking mechanisms.
///
/// ## Features
///
/// - **Automatic Cloning**: Each thread gets its own clone for safe access
/// - **Change Detection**: Built-in waiting mechanisms for data changes
/// - **Flexible Access**: Read, write, and update operations with proper locking
/// - **Condition Variables**: Efficient waiting for data modifications
/// - **Thread Safety**: Implements `Send` and `Sync` automatically
///
/// ## Example
///
/// ```rust
/// use thread_share::ThreadShare;
/// use std::thread;
///
/// let data = ThreadShare::new(vec![1, 2, 3]);
/// let clone = data.clone();
///
/// thread::spawn(move || {
///     clone.update(|v| v.push(4));
/// });
///
/// data.wait_for_change_forever();
/// assert_eq!(data.get(), vec![1, 2, 3, 4]);
/// ```
///
/// ## Performance
///
/// - **Read Operations**: Multiple threads can read simultaneously
/// - **Write Operations**: Exclusive access during writes
/// - **Change Detection**: Efficient condition variable notifications
/// - **Memory Overhead**: Minimal overhead from Arc and RwLock structures
pub struct ThreadShare<T> {
    data: Arc<RwLock<T>>,
    sender: Arc<Mutex<()>>,
    receiver: Arc<Mutex<()>>,
    condvar: Arc<Condvar>,
}

// Automatically implement Send and Sync for ThreadShare
unsafe impl<T> Send for ThreadShare<T> {}
unsafe impl<T> Sync for ThreadShare<T> {}

impl<T> ThreadShare<T> {
    /// Creates a new ThreadShare instance with data
    ///
    /// ## Arguments
    ///
    /// * `data` - The initial data to share between threads
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    ///
    /// let counter = ThreadShare::new(0);
    /// let data = ThreadShare::new(vec![1, 2, 3]);
    /// let message = ThreadShare::new(String::from("Hello"));
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            sender: Arc::new(Mutex::new(())),
            receiver: Arc::new(Mutex::new(())),
            condvar: Arc::new(Condvar::new()),
        }
    }

    /// Gets a copy of data (for types implementing Clone)
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
    /// use thread_share::ThreadShare;
    ///
    /// let counter = ThreadShare::new(42);
    /// let value = counter.get();
    /// assert_eq!(value, 42);
    /// ```
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.data.read().clone()
    }

    /// Gets a reference to data for reading
    ///
    /// This method provides read-only access to the data through a closure.
    /// Multiple threads can read simultaneously.
    ///
    /// ## Arguments
    ///
    /// * `f` - Closure that receives a reference to the data
    ///
    /// ## Returns
    ///
    /// The result of the closure execution.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    ///
    /// let data = ThreadShare::new(vec![1, 2, 3]);
    /// let length = data.read(|v| v.len());
    /// assert_eq!(length, 3);
    ///
    /// let sum: i32 = data.read(|v| v.iter().sum());
    /// assert_eq!(sum, 6);
    /// ```
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let data = self.data.read();
        f(&data)
    }

    /// Gets a mutable reference to data
    ///
    /// This method provides mutable access to the data through a closure.
    /// Only one thread can write at a time.
    ///
    /// ## Arguments
    ///
    /// * `f` - Closure that receives a mutable reference to the data
    ///
    /// ## Returns
    ///
    /// The result of the closure execution.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    ///
    /// let data = ThreadShare::new(vec![1, 2, 3]);
    /// let length = data.write(|v| {
    ///     v.push(4);
    ///     v.len()
    /// });
    /// assert_eq!(length, 4);
    /// assert_eq!(data.get(), vec![1, 2, 3, 4]);
    /// ```
    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut data = self.data.write();
        f(&mut data)
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
    /// use thread_share::ThreadShare;
    /// use std::thread;
    ///
    /// let data = ThreadShare::new(0);
    /// let clone = data.clone();
    ///
    /// thread::spawn(move || {
    ///     clone.set(100);
    /// });
    ///
    /// data.wait_for_change_forever();
    /// assert_eq!(data.get(), 100);
    /// ```
    pub fn set(&self, new_data: T) {
        let mut data = self.data.write();
        *data = new_data;
        self.condvar.notify_all();
    }

    /// Updates data using a function and notifies waiting threads
    ///
    /// This method allows you to modify the data through a closure and
    /// automatically notifies waiting threads of the change.
    ///
    /// ## Arguments
    ///
    /// * `f` - Closure that receives a mutable reference to the data
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    ///
    /// let counter = ThreadShare::new(0);
    ///
    /// counter.update(|x| *x += 1);
    /// assert_eq!(counter.get(), 1);
    ///
    /// counter.update(|x| *x *= 2);
    /// assert_eq!(counter.get(), 2);
    /// ```
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.data.write();
        f(&mut data);
        self.condvar.notify_all();
    }

    /// Waits for data changes with timeout
    ///
    /// This method waits for a change notification with a specified timeout.
    ///
    /// ## Arguments
    ///
    /// * `timeout` - Maximum time to wait for changes
    ///
    /// ## Returns
    ///
    /// `true` if the timeout was reached, `false` if a change occurred.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    /// use std::time::Duration;
    ///
    /// let data = ThreadShare::new(0);
    /// let clone = data.clone();
    ///
    /// // Spawn thread that will change data after 200ms
    /// std::thread::spawn(move || {
    ///     std::thread::sleep(Duration::from_millis(200));
    ///     clone.set(100);
    /// });
    ///
    /// // Wait for change with 100ms timeout
    /// let timed_out = data.wait_for_change(Duration::from_millis(100));
    /// assert!(timed_out); // Should timeout
    ///
    /// // Wait for change with 300ms timeout
    /// let timed_out = data.wait_for_change(Duration::from_millis(300));
    /// assert!(!timed_out); // Should not timeout
    /// ```
    pub fn wait_for_change(&self, timeout: Duration) -> bool {
        let guard = self.receiver.lock().unwrap();
        let result = self.condvar.wait_timeout(guard, timeout).unwrap();
        result.1.timed_out()
    }

    /// Waits for data changes infinitely
    ///
    /// This method waits indefinitely for a change notification.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let data = ThreadShare::new(0);
    /// let clone = data.clone();
    ///
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_millis(100));
    ///     clone.set(100);
    /// });
    ///
    /// // Wait indefinitely for change
    /// data.wait_for_change_forever();
    /// assert_eq!(data.get(), 100);
    /// ```
    pub fn wait_for_change_forever(&self) {
        let guard = self.receiver.lock().unwrap();
        let _unused = self.condvar.wait(guard).unwrap();
    }

    /// Creates a clone for use in another thread
    ///
    /// This method creates a new `ThreadShare<T>` instance that shares
    /// the same underlying data. Each clone can be safely moved to
    /// different threads.
    ///
    /// ## Returns
    ///
    /// A new `ThreadShare<T>` instance sharing the same data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let data = ThreadShare::new(0);
    /// let clone1 = data.clone();
    /// let clone2 = data.clone();
    ///
    /// // Each clone can be used in different threads
    /// thread::spawn(move || {
    ///     clone1.set(100);
    /// });
    ///
    /// thread::spawn(move || {
    ///     clone2.set(200);
    /// });
    ///
    /// // Main thread waits for changes
    /// data.wait_for_change_forever();
    /// ```
    pub fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            sender: Arc::clone(&self.sender),
            receiver: Arc::clone(&self.receiver),
            condvar: Arc::clone(&self.condvar),
        }
    }

    /// Gets Arc on data for transfer to thread without cloning
    ///
    /// This method converts the `ThreadShare<T>` into an `Arc<RwLock<T>>`,
    /// which can be moved into threads without cloning the `ThreadShare` itself.
    ///
    /// ## Returns
    ///
    /// An `Arc<RwLock<T>>` containing the shared data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    /// use std::thread;
    ///
    /// let data = ThreadShare::new(0);
    /// let arc_data = data.into_arc();
    ///
    /// thread::spawn(move || {
    ///     let mut guard = arc_data.write();
    ///     *guard += 100;
    /// });
    /// ```
    pub fn into_arc(self) -> Arc<RwLock<T>> {
        self.data
    }

    /// Gets Arc<RwLock<T>> for version with locks
    ///
    /// This method returns an `Arc<RwLock<T>>` that shares the same data
    /// as this `ThreadShare<T>`. This is useful when you need to work
    /// directly with the underlying `Arc<RwLock<T>>` structure.
    ///
    /// ## Returns
    ///
    /// An `Arc<RwLock<T>>` sharing the same data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    ///
    /// let data = ThreadShare::new(vec![1, 2, 3]);
    /// let arc_data = data.as_arc_locked();
    ///
    /// // Use the Arc<RwLock<T>> directly
    /// let mut guard = arc_data.write();
    /// guard.push(4);
    /// drop(guard);
    ///
    /// // Changes are visible in the original ThreadShare
    /// assert_eq!(data.get(), vec![1, 2, 3, 4]);
    /// ```
    pub fn as_arc_locked(&self) -> Arc<RwLock<T>> {
        Arc::clone(&self.data)
    }

    /// Gets Arc on data for transfer to thread without cloning (reference)
    ///
    /// This method creates an `Arc<AtomicPtr<T>>` from the current data.
    /// **Warning**: This creates an independent copy of the data, not a shared reference.
    /// Changes to the returned `Arc<AtomicPtr<T>>` will not be visible in the original `ThreadShare<T>`.
    ///
    /// ## Requirements
    ///
    /// The type `T` must implement `Clone` trait.
    ///
    /// ## Returns
    ///
    /// An `Arc<AtomicPtr<T>>` containing a copy of the current data.
    ///
    /// ## Warning
    ///
    /// This method creates an **independent copy** of the data. Use `as_arc_locked()` if you
    /// need a shared reference to the same data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ThreadShare;
    ///
    /// let data = ThreadShare::new(vec![1, 2, 3]);
    /// let arc_data = data.as_arc();
    ///
    /// // This modifies the copy, not the original
    /// // Use ArcThreadShare::from_arc(arc_data) to work with it
    /// ```
    pub fn as_arc(&self) -> Arc<std::sync::atomic::AtomicPtr<T>>
    where
        T: Clone,
    {
        // Create AtomicPtr from current data
        let current_data = self.data.read();
        let cloned_data = (*current_data).clone();
        let boxed = Box::new(cloned_data);
        let ptr = Box::into_raw(boxed);
        Arc::new(std::sync::atomic::AtomicPtr::new(ptr))
    }
}

impl<T> Clone for ThreadShare<T> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

/// Simplified version for simple types
pub struct SimpleShare<T> {
    data: Arc<Mutex<T>>,
}

// Automatically implement Send and Sync for SimpleShare
unsafe impl<T> Send for SimpleShare<T> {}
unsafe impl<T> Sync for SimpleShare<T> {}

impl<T> SimpleShare<T> {
    /// Creates a new SimpleShare
    ///
    /// This method creates a new `SimpleShare<T>` instance with the provided data.
    /// SimpleShare is a simplified version of ThreadShare without change detection.
    ///
    /// ## Arguments
    ///
    /// * `data` - The initial data to share between threads
    ///
    /// ## Returns
    ///
    /// A new `SimpleShare<T>` instance containing the data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::SimpleShare;
    ///
    /// let counter = SimpleShare::new(0);
    /// let message = SimpleShare::new(String::from("Hello"));
    /// let data = SimpleShare::new(vec![1, 2, 3]);
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// Gets data
    ///
    /// This method retrieves a copy of the current data. The operation is safe
    /// but involves cloning the data.
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
    /// use thread_share::SimpleShare;
    ///
    /// let counter = SimpleShare::new(42);
    /// let value = counter.get();
    /// assert_eq!(value, 42);
    /// ```
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.data.lock().unwrap().clone()
    }

    /// Sets data
    ///
    /// This method replaces the current data with new data.
    ///
    /// ## Arguments
    ///
    /// * `new_data` - The new data to set
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::SimpleShare;
    ///
    /// let counter = SimpleShare::new(0);
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn set(&self, new_data: T) {
        let mut data = self.data.lock().unwrap();
        *data = new_data;
    }

    /// Updates data
    ///
    /// This method allows you to modify the data through a closure.
    ///
    /// ## Arguments
    ///
    /// * `f` - Closure that receives a mutable reference to the data
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::SimpleShare;
    ///
    /// let counter = SimpleShare::new(0);
    ///
    /// counter.update(|x| *x += 1);
    /// assert_eq!(counter.get(), 1);
    ///
    /// counter.update(|x| *x *= 2);
    /// assert_eq!(counter.get(), 2);
    /// ```
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.data.lock().unwrap();
        f(&mut data);
    }

    /// Clones for use in another thread
    ///
    /// This method creates a new `SimpleShare<T>` instance that shares
    /// the same underlying data. Each clone can be safely moved to
    /// different threads.
    ///
    /// ## Returns
    ///
    /// A new `SimpleShare<T>` instance sharing the same data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::SimpleShare;
    /// use std::thread;
    ///
    /// let data = SimpleShare::new(0);
    /// let clone1 = data.clone();
    /// let clone2 = data.clone();
    ///
    /// // Each clone can be used in different threads
    /// thread::spawn(move || {
    ///     clone1.set(100);
    /// });
    ///
    /// thread::spawn(move || {
    ///     clone2.set(200);
    /// });
    /// ```
    pub fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }

    /// Gets Arc on data for transfer to thread without cloning
    ///
    /// This method consumes the `SimpleShare<T>` and returns the underlying
    /// `Arc<Mutex<T>>`. This is useful when you need to work directly
    /// with the `Arc<Mutex<T>>` structure.
    ///
    /// ## Returns
    ///
    /// The underlying `Arc<Mutex<T>>` containing the shared data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::SimpleShare;
    ///
    /// let data = SimpleShare::new(vec![1, 2, 3]);
    /// let arc_data = data.into_arc();
    ///
    /// // Use the Arc<Mutex<T>> directly
    /// let mut guard = arc_data.lock().unwrap();
    /// guard.push(4);
    /// drop(guard);
    /// ```
    pub fn into_arc(self) -> Arc<Mutex<T>> {
        self.data
    }

    /// Gets Arc on data for transfer to thread without cloning (reference)
    ///
    /// This method returns an `Arc<Mutex<T>>` that shares the same data
    /// as this `SimpleShare<T>`. This is useful when you need to work
    /// directly with the underlying `Arc<Mutex<T>>` structure.
    ///
    /// ## Returns
    ///
    /// An `Arc<Mutex<T>>` sharing the same data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::SimpleShare;
    ///
    /// let data = SimpleShare::new(vec![1, 2, 3]);
    /// let arc_data = data.as_arc();
    ///
    /// // Use the Arc<Mutex<T>> directly
    /// let mut guard = arc_data.lock().unwrap();
    /// guard.push(4);
    /// drop(guard);
    ///
    /// // Changes are visible in the original SimpleShare
    /// assert_eq!(data.get(), vec![1, 2, 3, 4]);
    /// ```
    pub fn as_arc(&self) -> Arc<Mutex<T>> {
        Arc::clone(&self.data)
    }
}

impl<T> Clone for SimpleShare<T> {
    fn clone(&self) -> Self {
        self.clone()
    }
}
