use std::thread;
use std::time::Duration;
use thread_share::{ArcThreadShare, ArcThreadShareLocked, SimpleShare, ThreadShare};

#[derive(Clone, Debug)]
struct User {
    name: String,
    age: u32,
    is_active: bool,
}

impl User {
    fn new(name: String, age: u32) -> Self {
        Self {
            name,
            age,
            is_active: true,
        }
    }

    fn birthday(&mut self) {
        self.age += 1;
        println!(
            "🎉 {} celebrated birthday! Now he is {} years old",
            self.name, self.age
        );
    }

    fn deactivate(&mut self) {
        self.is_active = false;
        println!("❌ {} deactivated", self.name);
    }
}

fn main() {
    println!("=== Example of using constructors ===");

    // 1. ThreadShare through constructor
    println!("\n--- 1. ThreadShare through constructor ---");
    let user = ThreadShare::new(User::new("Alice".to_string(), 25));
    let user_clone = user.clone();

    let handle1 = thread::spawn(move || {
        for _ in 0..3 {
            thread::sleep(Duration::from_millis(200));
            user_clone.update(|u| u.birthday());
        }
        user_clone.update(|u| u.deactivate());
    });

    // Main thread reads values
    for _ in 0..4 {
        thread::sleep(Duration::from_millis(300));
        let current_user = user.get();
        println!(
            "Main thread: {} - {} years old, active: {}",
            current_user.name, current_user.age, current_user.is_active
        );
    }

    handle1.join().unwrap();

    // 2. SimpleShare through constructor
    println!("\n--- 2. SimpleShare through constructor ---");
    let counter = SimpleShare::new(0);
    let counter_clone = counter.clone();

    let handle2 = thread::spawn(move || {
        for i in 1..=5 {
            thread::sleep(Duration::from_millis(100));
            counter_clone.set(i * 10);
            println!("Thread set counter: {}", i * 10);
        }
    });

    // Main thread reads values
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(150));
        let value = counter.get();
        println!("Main thread read: {}", value);
    }

    handle2.join().unwrap();

    // 3. ArcThreadShare through constructor (without locks)
    println!("\n--- 3. ArcThreadShare through constructor (without locks) ---");
    let atomic_counter = ArcThreadShare::new(0);
    let atomic_counter_clone = ArcThreadShare::from_arc(atomic_counter.data.clone());

    let handle3 = thread::spawn(move || {
        for _ in 1..=5 {
            thread::sleep(Duration::from_millis(100));
            atomic_counter_clone.update(|x| *x += 5);
            println!("Thread increased atomic counter by 5");
        }
    });

    // Main thread reads values
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(150));
        let value = atomic_counter.get();
        println!("Main thread read atomic counter: {}", value);
    }

    handle3.join().unwrap();

    // 4. ArcThreadShareLocked through constructor (with locks)
    println!("\n--- 4. ArcThreadShareLocked through constructor (with locks) ---");
    let locked_counter = ArcThreadShareLocked::new(0);
    let locked_counter_clone = ArcThreadShareLocked::from_arc(locked_counter.data.clone());

    let handle4 = thread::spawn(move || {
        for _ in 1..=5 {
            thread::sleep(Duration::from_millis(100));
            locked_counter_clone.update(|x| *x += 5);
            println!("Thread increased locked counter by 5");
        }
    });

    // Main thread reads values
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(150));
        let value = locked_counter.get();
        println!("Main thread read locked counter: {}", value);
    }

    handle4.join().unwrap();

    // 5. Comparison of all methods
    println!("\n--- 5. Comparison of all methods ---");
    println!("ThreadShare (with locks): {:?}", user.get());
    println!("SimpleShare (with locks): {}", counter.get());
    println!("ArcThreadShare (without locks): {}", atomic_counter.get());
    println!(
        "ArcThreadShareLocked (with locks): {}",
        locked_counter.get()
    );

    println!("\n✅ Successfully used all constructors!");
    println!("🎯 ThreadShare::new() - main constructor with locks");
    println!("🔒 SimpleShare::new() - simple constructor with locks");
    println!("⚡ ArcThreadShare::new() - constructor without locks");
    println!("🔐 ArcThreadShareLocked::new() - constructor with locks");
}
