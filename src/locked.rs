//! # Locked Module - ArcThreadShareLocked<T>
//!
//! This module provides `ArcThreadShareLocked<T>`, a safe zero-copy structure for
//! data sharing between threads using read/write locks.
//!
//! ## 🚀 Overview
//!
//! `ArcThreadShareLocked<T>` is the **recommended alternative** to `ArcThreadShare<T>`
//! when you need zero-copy operations with guaranteed thread safety. It provides
//! the performance benefits of zero-copy while maintaining the safety guarantees
//! of lock-based synchronization.
//!
//! ## Key Features
//!
//! - **Zero-Copy Operations**: No data cloning during access
//! - **Guaranteed Thread Safety**: Uses `RwLock` for safe concurrent access
//! - **High Performance**: Efficient `parking_lot` synchronization primitives
//! - **Memory Efficiency**: Single copy of data shared across threads
//! - **No Lost Updates**: All operations are guaranteed to succeed
//! - **Predictable Behavior**: Consistent performance under all contention levels
//!
//! ## When to Use ArcThreadShareLocked<T>
//!
//! ### ✅ Perfect Use Cases
//! - **Safe zero-copy operations** without the limitations of `ArcThreadShare<T>`
//! - **High-frequency updates** where `ArcThreadShare<T>` would lose operations
//! - **Critical data integrity** requirements
//! - **Predictable performance** needs
//! - **Large data structures** where cloning would be expensive
//! - **Production applications** requiring reliability
//!
//! ### 🔄 Comparison with Other Patterns
//!
//! | Pattern | Zero-Copy | Thread Safety | Performance | Reliability |
//! |---------|-----------|---------------|-------------|-------------|
//! | **ThreadShare<T>** | ❌ | ✅ | Medium | ✅ |
//! | **ArcThreadShare<T>** | ✅ | ⚠️ | High (unreliable) | ❌ |
//! | **ArcThreadShareLocked<T>** | ✅ | ✅ | High | ✅ |
//!
//! ## Example Usage
//!
//! ### Basic Operations
//! ```rust
//! use thread_share::ArcThreadShareLocked;
//!
//! let counter = ArcThreadShareLocked::new(0);
//!
//! // Safe zero-copy operations
//! counter.update(|x| *x += 1);
//! counter.update(|x| *x += 2);
//!
//! assert_eq!(counter.get(), 3);
//! ```
//!
//! ### From ThreadShare
//! ```rust
//! use thread_share::{share, ArcThreadShareLocked};
//!
//! let data = share!(String::from("Hello"));
//! let arc_data = data.as_arc_locked();
//! let locked_share = ArcThreadShareLocked::from_arc(arc_data);
//!
//! // Safe zero-copy with guaranteed thread safety
//! locked_share.update(|s| s.push_str(" World"));
//! ```
//!
//! ### High-Frequency Updates
//! ```rust
//! use thread_share::ArcThreadShareLocked;
//! use std::thread;
//!
//! let counter = ArcThreadShareLocked::new(0);
//! let clone = counter.clone();
//!
//! // Spawn multiple threads for high-frequency updates
//! let handles: Vec<_> = (0..10).map(|_| {
//!     let counter_clone = clone.clone();
//!     thread::spawn(move || {
//!         for _ in 0..10000 {
//!             counter_clone.update(|x| *x += 1);
//!         }
//!     })
//! }).collect();
//!
//! // Wait for all threads
//! for handle in handles {
//!     handle.join().unwrap();
//! }
//!
//! // All updates are guaranteed to succeed
//! assert_eq!(counter.get(), 100000);
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Low Contention**: Excellent performance, minimal overhead
//! - **Medium Contention**: Good performance with some lock contention
//! - **High Contention**: Consistent performance, no lost operations
//! - **Memory Usage**: Minimal overhead from lock structures
//! - **Scalability**: Scales well with thread count
//!
//! ## Thread Safety
//!
//! `ArcThreadShareLocked<T>` automatically implements `Send` and `Sync` traits,
//! making it safe to use across thread boundaries. The internal `RwLock` ensures
//! that all operations are thread-safe and no data races can occur.
//!
//! ## Memory Management
//!
//! - **Arc**: Provides reference counting for shared ownership
//! - **RwLock**: Ensures exclusive write access and concurrent read access
//! - **No Box Allocation**: Unlike `ArcThreadShare<T>`, no per-operation allocations
//! - **Efficient Locking**: Uses `parking_lot` for optimal performance
//!
//! ## Best Practices
//!
//! 1. **Use for zero-copy needs**: When you need to avoid cloning data
//! 2. **Prefer over ArcThreadShare**: For reliable, production applications
//! 3. **Monitor lock contention**: Use `read()` and `write()` methods appropriately
//! 4. **Keep critical sections short**: Minimize time spent holding locks
//! 5. **Use descriptive variable names**: Make it clear this is the locked version
//!
//! ## Migration from ArcThreadShare<T>
//!
//! If you're currently using `ArcThreadShare<T>` and experiencing issues:
//!
//! ```rust
//! // Old: Unreliable atomic operations
//! use thread_share::ArcThreadShare;
//! let arc_share = ArcThreadShare::new(0);
//! arc_share.update(|x| *x += 1); // May fail under contention
//!
//! // New: Reliable locked operations
//! use thread_share::ArcThreadShareLocked;
//! let locked_share = ArcThreadShareLocked::new(0);
//! locked_share.update(|x| *x += 1); // Always succeeds
//! ```
//!
//! ## Error Handling
//!
//! Unlike `ArcThreadShare<T>`, `ArcThreadShareLocked<T>` operations never fail
//! due to contention. All operations are guaranteed to complete successfully,
//! making error handling much simpler.
//!
//! ## Integration with ThreadShare
//!
//! `ArcThreadShareLocked<T>` integrates seamlessly with `ThreadShare<T>`:
//!
//! ```rust
//! use thread_share::{share, ArcThreadShareLocked};
//!
//! let data = share!(vec![1, 2, 3]);
//!
//! // Get locked version for zero-copy operations
//! let arc_data = data.as_arc_locked();
//! let locked_share = ArcThreadShareLocked::from_arc(arc_data);
//!
//! // Use locked version in threads
//! locked_share.update(|v| v.push(4));
//!
//! // Changes are visible in original ThreadShare
//! assert_eq!(data.get(), vec![1, 2, 3, 4]);
//! ```

use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(feature = "serialize")]
use serde::{de::DeserializeOwned};

/// Helper structure for working with Arc<RwLock<T>> directly (with locks)
///
/// `ArcThreadShareLocked<T>` is the **recommended alternative** to `ArcThreadShare<T>`
/// when you need zero-copy operations with guaranteed thread safety. It provides
/// the performance benefits of zero-copy while maintaining the safety guarantees
/// of lock-based synchronization.
///
/// ## Key Features
///
/// - **Zero-Copy Operations**: No data cloning during access
/// - **Guaranteed Thread Safety**: Uses `RwLock` for safe concurrent access
/// - **High Performance**: Efficient `parking_lot` synchronization primitives
/// - **Memory Efficiency**: Single copy of data shared across threads
/// - **No Lost Updates**: All operations are guaranteed to succeed
/// - **Predictable Behavior**: Consistent performance under all contention levels
///
/// ## When to Use
///
/// - **Safe zero-copy operations** without the limitations of `ArcThreadShare<T>`
/// - **High-frequency updates** where `ArcThreadShare<T>` would lose operations
/// - **Critical data integrity** requirements
/// - **Predictable performance** needs
/// - **Production applications** requiring reliability
///
/// ## Example
///
/// ```rust
/// use thread_share::ArcThreadShareLocked;
///
/// let counter = ArcThreadShareLocked::new(0);
///
/// // Safe zero-copy operations
/// counter.update(|x| *x += 1);
/// counter.update(|x| *x += 2);
///
/// assert_eq!(counter.get(), 3);
/// ```
///
/// ## Performance
///
/// - **Low Contention**: Excellent performance, minimal overhead
/// - **Medium Contention**: Good performance with some lock contention
/// - **High Contention**: Consistent performance, no lost operations
/// - **Memory Usage**: Minimal overhead from lock structures
/// - **Scalability**: Scales well with thread count
pub struct ArcThreadShareLocked<T> {
    pub data: Arc<RwLock<T>>,
}

// Automatically implement Send and Sync for ArcThreadShareLocked
unsafe impl<T> Send for ArcThreadShareLocked<T> {}
unsafe impl<T> Sync for ArcThreadShareLocked<T> {}

impl<T> Clone for ArcThreadShareLocked<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

impl<T> ArcThreadShareLocked<T> {
    /// Creates a new ArcThreadShareLocked with data
    ///
    /// This method creates a new `ArcThreadShareLocked<T>` instance with the provided data.
    /// The data is wrapped in an `Arc<RwLock<T>>` for thread-safe sharing.
    ///
    /// ## Arguments
    ///
    /// * `data` - The initial data to share between threads
    ///
    /// ## Returns
    ///
    /// A new `ArcThreadShareLocked<T>` instance containing the data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let counter = ArcThreadShareLocked::new(0);
    /// let message = ArcThreadShareLocked::new(String::from("Hello"));
    /// let data = ArcThreadShareLocked::new(vec![1, 2, 3]);
    /// ```
    pub fn new(data: T) -> Self {
        let arc = Arc::new(RwLock::new(data));
        Self { data: arc }
    }

    /// Creates from Arc<RwLock<T>>
    ///
    /// This method creates an `ArcThreadShareLocked<T>` from an existing `Arc<RwLock<T>>`.
    /// Useful when you already have locked data from other sources, such as
    /// from `ThreadShare<T>::as_arc_locked()`.
    ///
    /// ## Arguments
    ///
    /// * `arc` - An `Arc<RwLock<T>>` containing the data to share
    ///
    /// ## Returns
    ///
    /// A new `ArcThreadShareLocked<T>` instance sharing the same data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{share, ArcThreadShareLocked};
    ///
    /// let data = share!(vec![1, 2, 3]);
    /// let arc_data = data.as_arc_locked();
    /// let locked_share = ArcThreadShareLocked::from_arc(arc_data);
    ///
    /// // Now you can use safe zero-copy operations
    /// locked_share.update(|v| v.push(4));
    /// ```
    pub fn from_arc(arc: Arc<RwLock<T>>) -> Self {
        Self { data: arc }
    }

    /// Gets a copy of data
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
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let counter = ArcThreadShareLocked::new(42);
    /// let value = counter.get();
    /// assert_eq!(value, 42);
    /// ```
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.data.read().clone()
    }

    /// Gets a reference to data (no cloning!)
    ///
    /// This method provides read-only access to the data without cloning.
    /// The returned guard holds the read lock until it's dropped.
    ///
    /// ## Returns
    ///
    /// A read guard that provides access to the data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let data = ArcThreadShareLocked::new(vec![1, 2, 3]);
    ///
    /// // Get reference without cloning
    /// {
    ///     let guard = data.get_ref();
    ///     assert_eq!(guard.len(), 3);
    ///     assert_eq!(guard[0], 1);
    ///     // Guard is automatically dropped here, releasing the lock
    /// }
    /// ```
    ///
    /// ## Note
    ///
    /// This method will block until the read lock can be acquired.
    /// Multiple threads can read simultaneously.
    /// For non-blocking behavior, use `try_get_ref()`.
    pub fn get_ref(&self) -> parking_lot::RwLockReadGuard<'_, T> {
        self.data.read()
    }

    /// Tries to get a reference to data without blocking
    ///
    /// This method attempts to acquire a read lock without blocking.
    /// Returns `None` if the lock cannot be acquired immediately.
    ///
    /// ## Returns
    ///
    /// `Some(guard)` if the lock was acquired, `None` if it couldn't be acquired.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let data = ArcThreadShareLocked::new(vec![1, 2, 3]);
    ///
    /// // Try to get reference without blocking
    /// if let Some(guard) = data.try_get_ref() {
    ///     assert_eq!(guard.len(), 3);
    ///     assert_eq!(guard[0], 1);
    ///     // Guard is automatically dropped here
    /// } else {
    ///     // Lock was not available
    /// }
    ///
    /// // Ensure data is still accessible
    /// assert_eq!(data.get(), vec![1, 2, 3]);
    /// ```
    pub fn try_get_ref(&self) -> Option<parking_lot::RwLockReadGuard<'_, T>> {
        self.data.try_read()
    }

    /// Gets a mutable reference to data (no cloning!)
    ///
    /// This method provides mutable access to the data without cloning.
    /// The returned guard holds the write lock until it's dropped.
    ///
    /// ## Returns
    ///
    /// A write guard that provides mutable access to the data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let data = ArcThreadShareLocked::new(vec![1, 2, 3]);
    ///
    /// // Get mutable reference without cloning
    /// {
    ///     let mut guard = data.get_mut();
    ///     guard.push(4);
    ///     // Guard is automatically dropped here, releasing the lock
    /// }
    ///
    /// assert_eq!(data.get(), vec![1, 2, 3, 4]);
    /// ```
    ///
    /// ## Warning
    ///
    /// This method will block until the write lock can be acquired.
    /// In high-contention scenarios, this can cause delays.
    /// For non-blocking behavior, use `try_get_mut()`.
    ///
    /// ## Best Practices
    ///
    /// - Keep critical sections short to minimize lock contention
    /// - Always drop the guard explicitly in complex scenarios
    /// - Consider using `try_get_mut()` for non-blocking operations
    pub fn get_mut(&self) -> parking_lot::RwLockWriteGuard<'_, T> {
        self.data.write()
    }

    /// Tries to get a mutable reference to data without blocking
    ///
    /// This method attempts to acquire a write lock without blocking.
    /// Returns `None` if the lock cannot be acquired immediately.
    ///
    /// ## Returns
    ///
    /// `Some(guard)` if the lock was acquired, `None` if it couldn't be acquired.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let data = ArcThreadShareLocked::new(vec![1, 2, 3]);
    ///
    /// // Try to get mutable reference without blocking
    /// if let Some(mut guard) = data.try_get_mut() {
    ///     guard.push(4);
    ///     // Guard is automatically dropped here
    /// }
    ///
    /// // Ensure data is still accessible
    /// assert_eq!(data.get(), vec![1, 2, 3, 4]);
    /// ```
    pub fn try_get_mut(&self) -> Option<parking_lot::RwLockWriteGuard<'_, T>> {
        self.data.try_write()
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
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let counter = ArcThreadShareLocked::new(0);
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn set(&self, new_data: T) {
        let mut data = self.data.write();
        *data = new_data;
    }

    /// Updates data using a function
    ///
    /// This method allows you to modify the data through a closure.
    /// The operation is guaranteed to succeed and is thread-safe.
    ///
    /// ## Arguments
    ///
    /// * `f` - Closure that receives a mutable reference to the data
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let counter = ArcThreadShareLocked::new(0);
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
    }

    /// Reads data through a function
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
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let data = ArcThreadShareLocked::new(vec![1, 2, 3]);
    ///
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

    /// Writes data through a function
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
    /// use thread_share::ArcThreadShareLocked;
    ///
    /// let data = ArcThreadShareLocked::new(vec![1, 2, 3]);
    ///
    /// let length = data.write(|v| {
    ///     v.push(4);
    ///     v.len()
    /// });
    ///
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

    #[cfg(feature = "serialize")]
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    where
        T: serde::Serialize + Clone,
    {
        serde_json::to_string(&self.get())
    }

    #[cfg(feature = "serialize")]
    pub fn from_json<D: DeserializeOwned>(&self, json: &str) -> Result<D, serde_json::Error> {
        serde_json::from_str(json)
    }
}
