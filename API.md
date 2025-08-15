# Thread-Share API

## ThreadShare<T>

Main structure for data exchange between threads with change notifications support.

### Methods

#### Creation
```rust
pub fn new(data: T) -> Self
```
Creates new instance with data.

#### Reading data
```rust
pub fn get(&self) -> T
where T: Clone
```
Returns data copy. Requires `Clone` for type `T`.

```rust
pub fn read<F, R>(&self, f: F) -> R
where F: FnOnce(&T) -> R
```
Executes reading function on data and returns result.

#### Writing data
```rust
pub fn set(&self, new_data: T)
```
Replaces data with new and notifies all waiting threads.

```rust
pub fn update<F>(&self, f: F)
where F: FnOnce(&mut T)
```
Updates data through function and notifies all waiting threads.

```rust
pub fn write<F, R>(&self, f: F) -> R
where F: FnOnce(&mut T) -> R
```
Executes writing function on data and returns result.

#### Waiting for changes
```rust
pub fn wait_for_change(&self, timeout: Duration) -> bool
```
Waits for data changes with timeout. Returns `true` if timeout occurred.

```rust
pub fn wait_for_change_forever(&self)
```
Waits for data changes infinitely.

#### Cloning
```rust
pub fn clone(&self) -> Self
```
Creates clone for use in another thread.

## SimpleShare<T>

Simplified version without change notifications.

### Methods

#### Creation
```rust
pub fn new(data: T) -> Self
```

#### Reading
```rust
pub fn get(&self) -> T
where T: Clone
```

#### Writing
```rust
pub fn set(&self, new_data: T)
pub fn update<F>(&self, f: F)
where F: FnOnce(&mut T)
```

#### Cloning
```rust
pub fn clone(&self) -> Self
```

## Macros

### share!
```rust
let data = share!(42);
let data = share!(String::from("hello"));
let data = share!(MyStruct { field: 123 });
```

### simple_share!
```rust
let data = simple_share!(42);
let data = simple_share!("hello");
```

## Usage examples

### Basic scenario
```rust
use thread_share::share;

let counter = share!(0);
let counter_clone = counter.clone();

// In thread
std::thread::spawn(move || {
    counter_clone.set(100);
});

// In main thread
let value = counter.get();
```

### With notifications
```rust
let data = share!(false);
let data_clone = data.clone();

std::thread::spawn(move || {
    std::thread::sleep(Duration::from_secs(1));
    data_clone.set(true);
});

// Wait for changes
data.wait_for_change_forever();
println!("Data changed!");
```

### With custom types
```rust
#[derive(Clone)]
struct Config {
    port: u16,
    host: String,
}

let config = share!(Config {
    port: 8080,
    host: "localhost".to_string(),
});

config.update(|c| {
    c.port = 9000;
});
```
