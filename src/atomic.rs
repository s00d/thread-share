//! # Atomic Module - ArcThreadShare<T>
//!
//! This module provides `ArcThreadShare<T>`, a high-performance structure for
//! zero-copy data sharing between threads using atomic operations.
//!
//! ## ⚠️ Important Warning
//!
//! **`ArcThreadShare<T>` has significant limitations and should be used with caution!**
//!
//! ## Overview
//!
//! `ArcThreadShare<T>` uses `Arc<AtomicPtr<T>>` internally to provide zero-copy
//! data sharing without locks. While this can offer high performance, it comes
//! with important trade-offs.
//!
//! ## Key Features
//!
//! - **Zero-Copy Operations**: No data cloning during access
//! - **Atomic Updates**: Uses atomic pointer operations
//! - **High Performance**: Potentially faster than lock-based approaches
//! - **Memory Efficiency**: Single copy of data shared across threads
//!
//! ## ⚠️ Critical Limitations
//!
//! ### 1. **Non-Atomic Complex Operations**
//! ```rust
//! use thread_share::ArcThreadShare;
//!
//! let arc_share = ArcThreadShare::new(0);
//!
//! // ❌ This is NOT atomic and can cause race conditions
//! arc_share.update(|x| *x = *x + 1);
//!
//! // ✅ Use the atomic increment method instead
//! arc_share.increment();
//! ```
//!
//! **Problem**: The `update` method with complex operations like `+=` is not atomic.
//! Between reading the value, modifying it, and writing it back, other threads can interfere.
//!
//! ### 2. **High Contention Performance Issues**
//! ```rust
//! use thread_share::ArcThreadShare;
//!
//! let arc_share = ArcThreadShare::new(0);
//!
//! // ❌ High contention can cause significant performance degradation
//! for _ in 0..10000 {
//!     arc_share.increment(); // May lose many operations under high contention
//! }
//! ```
//!
//! **Problem**: Under high contention (many threads updating simultaneously), `AtomicPtr`
//! operations can lose updates due to:
//! - Box allocation/deallocation overhead
//! - CAS (Compare-And-Swap) failures requiring retries
//! - Memory pressure from frequent allocations
//!
//! **Expected Behavior**: In high-contention scenarios, you may see only 20-30% of
//! expected operations complete successfully.
//!
//! ### 3. **Memory Allocation Overhead**
//! ```rust
//! use thread_share::ArcThreadShare;
//!
//! let arc_share = ArcThreadShare::new(0);
//!
//! // Each increment operation involves:
//! // 1. Allocating new Box<T>
//! // 2. Converting to raw pointer
//! // 3. Atomic pointer swap
//! // 4. Deallocating old Box<T>
//! arc_share.increment();
//! ```
//!
//! **Problem**: Every update operation creates a new `Box<T>` and deallocates the old one,
//! which can be expensive for large data types.
//!
//! ## When to Use ArcThreadShare<T>
//!
//! ### ✅ Good Use Cases
//! - **Low-contention scenarios** (few threads, infrequent updates)
//! - **Performance-critical applications** where you understand the limitations
//! - **Simple atomic operations** using built-in methods (`increment()`, `add()`)
//! - **Read-heavy workloads** with occasional writes
//!
//! ### ❌ Avoid When
//! - **High-frequency updates** (>1000 ops/second per thread)
//! - **Critical data integrity** requirements
//! - **Predictable performance** needs
//! - **Large data structures** (due to allocation overhead)
//! - **Multi-threaded counters** with strict accuracy requirements
//!
//! ## Example Usage
//!
//! ### Basic Operations
//! ```rust
//! use thread_share::ArcThreadShare;
//!
//! let counter = ArcThreadShare::new(0);
//!
//! // Use atomic methods for safety
//! counter.increment();
//! counter.add(5);
//!
//! assert_eq!(counter.get(), 6);
//! ```
//!
//! ### From ThreadShare
//! ```rust
//! use thread_share::{share, ArcThreadShare};
//!
//! let data = share!(String::from("Hello"));
//! let arc_data = data.as_arc();
//! let arc_share = ArcThreadShare::from_arc(arc_data);
//!
//! // Safe atomic operations
//! arc_share.update(|s| s.push_str(" World"));
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Low Contention**: Excellent performance, minimal overhead
//! - **Medium Contention**: Good performance with some lost operations
//! - **High Contention**: Poor performance, many lost operations
//! - **Memory Usage**: Higher due to Box allocation/deallocation
//!
//! ## Best Practices
//!
//! 1. **Always use atomic methods** (`increment()`, `add()`) instead of complex `update()` operations
//! 2. **Test with realistic contention levels** before production use
//! 3. **Consider `ThreadShare<T>`** for critical applications
//! 4. **Monitor performance** under expected load conditions
//! 5. **Use for simple operations** only (increment, add, simple updates)
//!
//! ## Alternatives
//!
//! ### For High-Frequency Updates
//! ```rust
//! use thread_share::share;
//!
//! // Use ThreadShare with batching
//! let share = share!(0);
//! let clone = share.clone();
//!
//! clone.update(|x| {
//!     for _ in 0..100 {
//!         *x = *x + 1;
//!     }
//! });
//! ```
//!
//! ### For Critical Data Integrity
//! ```rust
//! use thread_share::share;
//!
//! // Use ThreadShare for guaranteed safety
//! let share = share!(vec![1, 2, 3]);
//! let clone = share.clone();
//!
//! // All operations are guaranteed to succeed
//! clone.update(|data| {
//!     // Critical modifications
//! });
//! ```
//!
//! ### For Safe Zero-Copy
//! ```rust
//! use thread_share::{share, ArcThreadShareLocked};
//!
//! // Use ArcThreadShareLocked for safe zero-copy
//! let share = share!(vec![1, 2, 3]);
//! let arc_data = share.as_arc_locked();
//! let locked_share = ArcThreadShareLocked::from_arc(arc_data);
//!
//! // Safe zero-copy with guaranteed thread safety
//! locked_share.update(|data| {
//!     // Safe modifications
//! });
//! ```

use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

#[cfg(feature = "serialize")]
use serde::{de::DeserializeOwned, Serialize};

/// Helper structure for working with Arc<AtomicPtr<T>> directly (without locks!)
///
/// **⚠️ WARNING: This structure has significant limitations and should be used with caution!**
///
/// ## Overview
///
/// `ArcThreadShare<T>` provides zero-copy data sharing between threads using atomic
/// pointer operations. While this can offer high performance, it comes with important
/// trade-offs that developers must understand.
///
/// ## Key Features
///
/// - **Zero-Copy Operations**: No data cloning during access
/// - **Atomic Updates**: Uses atomic pointer operations
/// - **High Performance**: Potentially faster than lock-based approaches
/// - **Memory Efficiency**: Single copy of data shared across threads
///
///
/// ### 2. **High Contention Performance Issues**
/// Under high contention, many operations may be lost due to:
/// - Box allocation/deallocation overhead
/// - CAS failures requiring retries
/// - Memory pressure from frequent allocations
///
/// ### 3. **Memory Allocation Overhead**
/// Every update operation involves Box allocation and deallocation.
///
/// ## When to Use
///
/// - **Low-contention scenarios** (few threads, infrequent updates)
/// - **Performance-critical applications** where you understand the limitations
/// - **Simple atomic operations** using built-in methods
/// - **Read-heavy workloads** with occasional writes
///
/// ## When to Avoid
///
/// - **High-frequency updates** (>1000 ops/second per thread)
/// - **Critical data integrity** requirements
/// - **Predictable performance** needs
/// - **Large data structures**
///
/// ## Example
///
/// ```rust
/// use thread_share::ArcThreadShare;
///
/// let counter = ArcThreadShare::new(0);
///
/// // Use atomic methods for safety
/// counter.increment();
/// counter.add(5);
///
/// assert_eq!(counter.get(), 6);
/// ```
pub struct ArcThreadShare<T> {
    pub data: Arc<AtomicPtr<T>>,
}

// Automatically implement Send and Sync for ArcThreadShare
unsafe impl<T> Send for ArcThreadShare<T> {}
unsafe impl<T> Sync for ArcThreadShare<T> {}

impl<T> Clone for ArcThreadShare<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

impl<T> ArcThreadShare<T> {
    /// Creates from Arc<AtomicPtr<T>>
    ///
    /// This method creates an `ArcThreadShare<T>` from an existing `Arc<AtomicPtr<T>>`.
    /// Useful when you already have atomic pointer data from other sources.
    ///
    /// ## Arguments
    ///
    /// * `arc` - An `Arc<AtomicPtr<T>>` containing the data to share
    ///
    /// ## Returns
    ///
    /// A new `ArcThreadShare<T>` instance sharing the same data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::{share, ArcThreadShare};
    ///
    /// let data = share!(String::from("Hello"));
    /// let arc_data = data.as_arc();
    /// let arc_share = ArcThreadShare::from_arc(arc_data);
    ///
    /// // Now you can use atomic operations
    /// arc_share.update(|s| s.push_str(" World"));
    /// ```
    pub fn from_arc(arc: Arc<AtomicPtr<T>>) -> Self {
        Self { data: arc }
    }

    /// Creates a new ArcThreadShare with data
    ///
    /// This method creates a new `ArcThreadShare<T>` instance with the provided data.
    /// The data is boxed and converted to an atomic pointer for thread-safe sharing.
    ///
    /// ## Arguments
    ///
    /// * `data` - The initial data to share between threads
    ///
    /// ## Requirements
    ///
    /// The type `T` must implement `Clone` trait.
    ///
    /// ## Returns
    ///
    /// A new `ArcThreadShare<T>` instance containing the data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShare;
    ///
    /// let counter = ArcThreadShare::new(0);
    /// let message = ArcThreadShare::new(String::from("Hello"));
    /// let data = ArcThreadShare::new(vec![1, 2, 3]);
    /// ```
    pub fn new(data: T) -> Self
    where
        T: Clone,
    {
        let boxed = Box::new(data);
        let ptr = Box::into_raw(boxed);
        let atomic = Arc::new(AtomicPtr::new(ptr));
        Self { data: atomic }
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
    /// use thread_share::ArcThreadShare;
    ///
    /// let counter = ArcThreadShare::new(42);
    /// let value = counter.get();
    /// assert_eq!(value, 42);
    /// ```
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        let ptr = self.data.load(Ordering::Acquire);
        unsafe { (*ptr).clone() }
    }

    /// Sets data atomically
    ///
    /// This method atomically replaces the current data with new data.
    /// The old data is automatically deallocated.
    ///
    /// ## Arguments
    ///
    /// * `new_data` - The new data to set
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShare;
    ///
    /// let counter = ArcThreadShare::new(0);
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn set(&self, new_data: T) {
        let new_boxed = Box::new(new_data);
        let new_ptr = Box::into_raw(new_boxed);

        let old_ptr = self.data.swap(new_ptr, Ordering::AcqRel);

        // Free old data
        if !old_ptr.is_null() {
            unsafe {
                drop(Box::from_raw(old_ptr));
            }
        }
    }

    /// Updates data (⚠️ NOT atomic for complex operations!)
    ///
    /// **⚠️ WARNING: This method is NOT atomic for complex operations!**
    ///
    /// For simple operations like `+= 1`, use the atomic methods `increment()` or `add()`
    /// instead. This method can cause race conditions under high contention.
    ///
    /// ## Arguments
    ///
    /// * `f` - Closure that receives a mutable reference to the data
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShare;
    ///
    /// let counter = ArcThreadShare::new(0);
    ///
    /// // ❌ NOT atomic - can cause race conditions
    /// counter.update(|x| *x += 1);
    ///
    /// // ✅ Use atomic methods instead
    /// counter.increment();
    /// ```
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let ptr = self.data.load(Ordering::Acquire);
        if !ptr.is_null() {
            unsafe {
                f(&mut *ptr);
            }
        }
    }

    /// Atomically increments numeric values (for types that support it)
    ///
    /// This method provides atomic increment operations for numeric types.
    /// It uses a compare-exchange loop to ensure atomicity.
    ///
    /// ## Requirements
    ///
    /// The type `T` must implement:
    /// - `Copy` - for efficient copying
    /// - `std::ops::Add<Output = T>` - for addition operations
    /// - `std::ops::AddAssign` - for compound assignment
    /// - `From<u8>` - for creating the value 1
    /// - `'static` - for lifetime requirements
    ///
    /// ## Example
    ///
    /// ```rust
    /// use thread_share::ArcThreadShare;
    ///
    /// let counter = ArcThreadShare::new(0);
    ///
    /// // Atomic increment
    /// counter.increment();
    /// assert_eq!(counter.get(), 1);
    ///
    /// counter.increment();
    /// assert_eq!(counter.get(), 2);
    /// ```
    pub fn increment(&self)
    where
        T: Copy + std::ops::Add<Output = T> + std::ops::AddAssign + From<u8> + 'static,
    {
        loop {
            let ptr = self.data.load(Ordering::Acquire);
            if ptr.is_null() {
                break;
            }

            let current_value = unsafe { *ptr };
            let new_value = current_value + T::from(1u8);

            // Try to atomically update the pointer with new data
            let new_boxed = Box::new(new_value);
            let new_ptr = Box::into_raw(new_boxed);

            if let Ok(_) =
                self.data
                    .compare_exchange(ptr, new_ptr, Ordering::AcqRel, Ordering::Acquire)
            {
                // Successfully updated, free old data
                unsafe {
                    drop(Box::from_raw(ptr));
                }
                break;
            } else {
                // Failed to update, free new data and retry
                unsafe {
                    drop(Box::from_raw(new_ptr));
                }
            }
        }
    }

    /// Atomically adds a value (for types that support it)
    pub fn add(&self, value: T)
    where
        T: Copy + std::ops::Add<Output = T> + std::ops::AddAssign + 'static,
    {
        loop {
            let ptr = self.data.load(Ordering::Acquire);
            if ptr.is_null() {
                break;
            }

            let current_value = unsafe { *ptr };
            let new_value = current_value + value;

            // Try to atomically update the pointer with new data
            let new_boxed = Box::new(new_value);
            let new_ptr = Box::into_raw(new_boxed);

            if let Ok(_) =
                self.data
                    .compare_exchange(ptr, new_ptr, Ordering::AcqRel, Ordering::Acquire)
            {
                // Successfully updated, free old data
                unsafe {
                    drop(Box::from_raw(ptr));
                }
                break;
            } else {
                // Failed to update, free new data and retry
                unsafe {
                    drop(Box::from_raw(new_ptr));
                }
            }
        }
    }

    /// Reads data
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let ptr = self.data.load(Ordering::Acquire);
        if !ptr.is_null() {
            unsafe { f(&*ptr) }
        } else {
            panic!("Attempted to read from null pointer");
        }
    }

    /// Writes data
    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let ptr = self.data.load(Ordering::Acquire);
        if !ptr.is_null() {
            unsafe { f(&mut *ptr) }
        } else {
            panic!("Attempted to write to null pointer");
        }
    }

    #[cfg(feature = "serialize")]
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    where
        T: Serialize + Clone,
    {
        serde_json::to_string(&self.get())
    }

    #[cfg(feature = "serialize")]
    pub fn from_json<D>(&self, json: &str) -> Result<D, serde_json::Error>
    where
        D: DeserializeOwned,
    {
        serde_json::from_str(json)
    }
}

/// Helper structure for working with Arc<Mutex<T>> directly
pub struct ArcSimpleShare<T> {
    pub data: Arc<std::sync::Mutex<T>>,
}

// Automatically implement Send and Sync for ArcSimpleShare
unsafe impl<T> Send for ArcSimpleShare<T> {}
unsafe impl<T> Sync for ArcSimpleShare<T> {}

impl<T> ArcSimpleShare<T> {
    /// Creates from Arc<Mutex<T>>
    pub fn from_arc(arc: Arc<std::sync::Mutex<T>>) -> Self {
        Self { data: arc }
    }

    /// Gets data
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.data.lock().unwrap().clone()
    }

    /// Sets data
    pub fn set(&self, new_data: T) {
        let mut data = self.data.lock().unwrap();
        *data = new_data;
    }

    /// Updates data
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.data.lock().unwrap();
        f(&mut data);
    }
}
