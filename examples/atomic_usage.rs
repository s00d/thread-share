use std::thread;
use std::time::Duration;
use thread_share::ArcThreadShare;

#[derive(Clone, Debug)]
struct Counter {
    value: u32,
}

impl Counter {
    fn new() -> Self {
        Self { value: 0 }
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn get_value(&self) -> u32 {
        self.value
    }
}

fn main() {
    println!("=== Example WITHOUT locks (AtomicPtr) ===");

    // Create ArcThreadShare directly (without RwLock!)
    let counter = ArcThreadShare::new(Counter::new());

    // Create several threads that work with one AtomicPtr
    let mut handles = vec![];

    for thread_id in 0..5 {
        let counter_clone = ArcThreadShare::from_arc(counter.data.clone());

        let handle = thread::spawn(move || {
            for _ in 0..10 {
                counter_clone.update(|c: &mut Counter| {
                    c.increment();
                    println!(
                        "Thread {}: increased counter to {}",
                        thread_id,
                        c.get_value()
                    );
                });
                thread::sleep(Duration::from_millis(50));
            }
        });

        handles.push(handle);
    }

    // Main thread reads values
    for _ in 0..20 {
        let value = counter.get();
        println!("Main thread read: {}", value.get_value());
        thread::sleep(Duration::from_millis(100));
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let final_value = counter.get();
    println!("\nFinal value: {}", final_value.get_value());
    println!("✅ Successfully used AtomicPtr WITHOUT locks!");
}
