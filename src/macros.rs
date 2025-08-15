//! # Macros Module
//!
//! This module provides convenient macros for creating and managing thread-safe
//! data structures with minimal boilerplate code.
//!
//! ## 🚀 Overview
//!
//! The macros module contains several utility macros that simplify common
//! operations when working with ThreadShare structures:
//!
//! - **`share!`** - Creates `ThreadShare<T>` instances with automatic type inference
//! - **`simple_share!`** - Creates `SimpleShare<T>` instances for basic use cases
//! - **`enhanced_share!`** - Creates `EnhancedThreadShare<T>` instances
//! - **`spawn_workers!`** - Spawns multiple threads with single macro call
//! - **`spawn_threads!`** - Alternative thread spawning macro
//! - **`thread_setup!`** - Sets up thread management with shared data
//!
//! ## Key Benefits
//!
//! ### 🎯 Simplified Creation
//! ```rust
//! use thread_share::{share, ThreadShare};
//!
//! // Without macros
//! let counter = ThreadShare::new(0);
//! let message = ThreadShare::new(String::from("Hello"));
//! let data = ThreadShare::new(vec![1, 2, 3]);
//!
//! // With macros
//! let counter = share!(0);
//! let message = share!(String::from("Hello"));
//! let data = share!(vec![1, 2, 3]);
//! ```
//!
//! ### 🧵 Enhanced Thread Management
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//!
//! let data = enhanced_share!(vec![1, 2, 3]);
//!
//! // Single macro call spawns all threads
//! spawn_workers!(data, {
//!     processor: |data| { data.update(|v| v.sort()); },
//!     validator: |data| { assert!(data.get().is_sorted()); }
//! });
//!
//! data.join_all().expect("Failed to join");
//! ```
//!
//! ## Macro Reference
//!
//! ### share! Macro
//!
//! Creates a `ThreadShare<T>` instance with automatic type inference.
//!
//! **Syntax**: `share!(expression)`
//!
//! **Example**:
//! ```rust
//! use thread_share::share;
//!
//! let counter = share!(0);                    // ThreadShare<i32>
//! let message = share!("Hello");              // ThreadShare<&str>
//! let data = share!(vec![1, 2, 3]);          // ThreadShare<Vec<i32>>
//! // let user = share!(User { id: 1, name: "Alice" }); // ThreadShare<User>
//! ```
//!
//! ### simple_share! Macro
//!
//! Creates a `SimpleShare<T>` instance for basic data sharing without change detection.
//!
//! **Syntax**: `simple_share!(expression)`
//!
//! **Example**:
//! ```rust
//! use thread_share::simple_share;
//!
//! let counter = simple_share!(0);             // SimpleShare<i32>
//! let flag = simple_share!(false);            // SimpleShare<bool>
//! let data = simple_share!(String::new());    // SimpleShare<String>
//! ```
//!
//! ### enhanced_share! Macro
//!
//! Creates an `EnhancedThreadShare<T>` instance with automatic thread management.
//!
//! **Syntax**: `enhanced_share!(expression)`
//!
//! **Example**:
//! ```rust
//! use thread_share::enhanced_share;
//!
//! let data = enhanced_share!(vec![1, 2, 3]);
//!
//! // Now you can use enhanced thread management
//! data.spawn("worker", |data| {
//!     data.update(|v| v.push(4));
//! });
//!
//! data.join_all().expect("Failed to join");
//! ```
//!
//! ### spawn_workers! Macro
//!
//! Spawns multiple threads with a single macro call, each with a descriptive name.
//!
//! **Syntax**: `spawn_workers!(data, { name: closure, ... })`
//!
//! **Example**:
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//!
//! let data = enhanced_share!(0);
//!
//! spawn_workers!(data, {
//!     incrementer: |data| {
//!         for _ in 0..100 {
//!             data.update(|x| *x += 1);
//!         }
//!     },
//!     monitor: |data| {
//!         for _ in 0..10 {
//!             std::thread::sleep(std::time::Duration::from_millis(100));
//!             println!("Value: {}", data.get());
//!         }
//!     }
//! });
//!
//! data.join_all().expect("Failed to join");
//! ```
//!
//! ### spawn_threads! Macro
//!
//! Alternative macro for spawning threads with different syntax.
//!
//! **Syntax**: `spawn_threads!(data, [closure1, closure2, ...])`
//!
//! **Example**:
//! ```rust
//! use thread_share::{enhanced_share, spawn_workers};
//!
//! let data = enhanced_share!(String::from("Hello"));
//!
//! spawn_workers!(data, {
//!     worker1: |data| { data.update(|s| s.push_str(" World")); },
//!     worker2: |data| { data.update(|s| s.push_str("!")); }
//! });
//!
//! data.join_all().expect("Failed to join");
//! ```
//!
//! ### thread_setup! Macro
//!
//! Sets up thread management with shared data and multiple worker functions.
//!
//! **Syntax**: `thread_setup!(shared_data, { name: function, ... })`
//!
//! **Example**:
//! ```rust
//! use thread_share::{share, thread_setup};
//!
//! let shared_data = share!(vec![1, 2, 3]);
//!
//! let manager = thread_setup!(shared_data, {
//!     worker1: |data| { data.update(|v| v.push(4)); },
//!     worker2: |data| { data.update(|v| v.push(5)); }
//! });
//!
//! manager.join_all().expect("Failed to join");
//! ```
//!
//! ## Type Inference
//!
//! All macros provide automatic type inference, so you don't need to specify
//! generic types explicitly:
//!
//! ```rust
//! use thread_share::share;
//!
//! // Type is automatically inferred as ThreadShare<i32>
//! let counter = share!(0);
//!
//! // Type is automatically inferred as ThreadShare<Vec<String>>
//! let data = share!(vec![String::from("a"), String::from("b")]);
//!
//! // Type is automatically inferred as ThreadShare<Option<bool>>
//! let flag = share!(Some(true));
//! ```
//!
//! ## Error Handling
//!
//! Basic error handling with standard threads:
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
//! ## Best Practices
//!
//! 1. **Use descriptive names** for spawned threads to aid debugging
//! 2. **Keep closures focused** on single responsibilities
//! 3. **Handle errors gracefully** from thread spawning and joining
//! 4. **Prefer `spawn_workers!`** over manual thread management
//! 5. **Use `enhanced_share!`** when you need automatic thread management
//!
//! ## Performance Considerations
//!
//! - **Macro expansion**: Happens at compile time, no runtime overhead
//! - **Type inference**: Compiler optimizations apply normally
//! - **Thread spawning**: Same performance as manual `thread::spawn`
//! - **Memory usage**: No additional overhead from macro usage
//!
//! ## Integration with Other Modules
//!
//! ```

/// Macro for creating ThreadShare with automatic type inference
///
/// This macro creates a `ThreadShare<T>` instance with automatic type inference.
/// It's the most commonly used macro for creating thread-safe shared data.
///
/// ## Syntax
///
/// `share!(expression)`
///
/// ## Arguments
///
/// * `expression` - The data to wrap in ThreadShare
///
/// ## Returns
///
/// A new `ThreadShare<T>` instance where `T` is inferred from the expression.
///
/// ## Example
///
/// ```rust
/// use thread_share::share;
///
/// // Basic types
/// let counter = share!(0);                    // ThreadShare<i32>
/// let message = share!("Hello");              // ThreadShare<&str>
/// let flag = share!(true);                    // ThreadShare<bool>
///
/// // Complex types
/// let data = share!(vec![1, 2, 3]);          // ThreadShare<Vec<i32>>
/// // let user = share!(User { id: 1, name: "Alice" }); // ThreadShare<User>
///
/// // Expressions
/// let result = share!(10 + 20);               // ThreadShare<i32>
/// let computed = share!(vec![1, 2, 3].len()); // ThreadShare<usize>
/// ```
///
/// ## Type Inference
///
/// The macro automatically infers the generic type `T` from the expression:
///
/// ```rust
/// use thread_share::share;
///
/// // No need to specify types explicitly
/// let counter: thread_share::ThreadShare<i32> = share!(0);
/// let data: thread_share::ThreadShare<Vec<String>> = share!(vec![String::new()]);
/// ```
#[macro_export]
macro_rules! share {
    ($data:expr) => {
        $crate::ThreadShare::new($data)
    };
}

/// Macro for creating SimpleShare
///
/// This macro creates a `SimpleShare<T>` instance for basic data sharing
/// without change detection capabilities.
///
/// ## Syntax
///
/// `simple_share!(expression)`
///
/// ## Arguments
///
/// * `expression` - The data to wrap in SimpleShare
///
/// ## Returns
///
/// A new `SimpleShare<T>` instance where `T` is inferred from the expression.
///
/// ## Example
///
/// ```rust
/// use thread_share::simple_share;
///
/// // Basic types
/// let counter = simple_share!(0);             // SimpleShare<i32>
/// let message = simple_share!("Hello");       // SimpleShare<&str>
/// let flag = simple_share!(false);            // SimpleShare<bool>
///
/// // Complex types
/// let data = simple_share!(vec![1, 2, 3]);   // SimpleShare<Vec<i32>>
/// // let user = simple_share!(User { id: 1, name: "Bob" }); // SimpleShare<User>
/// ```
///
/// ## When to Use
///
/// Use `simple_share!` when you need:
/// - Basic data sharing without change detection
/// - Minimal overhead and complexity
/// - Simple producer-consumer patterns
/// - Learning and prototyping
///
/// Use `share!` when you need:
/// - Change detection and waiting mechanisms
/// - Complex synchronization patterns
/// - Maximum flexibility and features
#[macro_export]
macro_rules! simple_share {
    ($data:expr) => {
        $crate::SimpleShare::new($data)
    };
}
