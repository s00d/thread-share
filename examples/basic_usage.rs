use std::thread;
use std::time::Duration;
use thread_share::{share, simple_share};

fn main() {
    println!("=== Basic ThreadShare Example ===");

    // Create a shared counter
    let counter = share!(0);
    let counter_clone = counter.clone();

    // Start a thread that increments the counter
    let handle = thread::spawn(move || {
        for i in 1..=10 {
            counter_clone.set(i);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Main thread reads values
    for _ in 0..10 {
        let value = counter.get();
        println!("Main thread read: {}", value);
        thread::sleep(Duration::from_millis(150));
    }

    handle.join().unwrap();

    println!("\n=== SimpleShare Example ===");

    let message = simple_share!("Hello!");
    let message_clone = message.clone();

    let handle2 = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        message_clone.set("Hello from thread!");
        thread::sleep(Duration::from_millis(100));
        message_clone.set("Goodbye!");
    });

    for _ in 0..3 {
        println!("Message: {}", message.get());
        thread::sleep(Duration::from_millis(100));
    }

    handle2.join().unwrap();

    println!("\n=== Custom Type Example ===");

    #[derive(Clone, Debug)]
    struct Person {
        name: String,
        age: u32,
    }

    let person = share!(Person {
        name: "Alice".to_string(),
        age: 25,
    });

    let person_clone = person.clone();

    // Thread updates person
    let handle3 = thread::spawn(move || {
        person_clone.update(|p| {
            p.age += 1;
            p.name = "Bob".to_string();
        });
        println!("Thread updated data");
    });

    // Main thread reads
    println!("Before: {:?}", person.get());
    handle3.join().unwrap();
    println!("After: {:?}", person.get());
}
