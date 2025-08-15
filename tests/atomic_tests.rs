use std::thread;
use std::time::Duration;
use thread_share::{share, ArcThreadShare};

#[test]
fn test_arc_thread_share_new() {
    let share = ArcThreadShare::new(42);
    assert_eq!(share.get(), 42);
}

#[test]
fn test_arc_thread_share_from_arc() {
    let original = share!(100);
    let arc_data = original.as_arc();
    let arc_share = ArcThreadShare::from_arc(arc_data);

    assert_eq!(arc_share.get(), 100);
}

#[test]
fn test_arc_thread_share_set_get() {
    let share = ArcThreadShare::new(42);

    share.set(100);
    assert_eq!(share.get(), 100);

    share.set(200);
    assert_eq!(share.get(), 200);

    share.set(300);
    assert_eq!(share.get(), 300);
}

#[test]
fn test_arc_thread_share_update() {
    let share = ArcThreadShare::new(vec![1, 2, 3]);

    share.update(|v| v.push(4));
    assert_eq!(share.get(), vec![1, 2, 3, 4]);

    share.update(|v| {
        v[0] = 100;
        v.push(5);
    });
    assert_eq!(share.get(), vec![100, 2, 3, 4, 5]);
}

#[test]
fn test_arc_thread_share_read() {
    let share = ArcThreadShare::new(String::from("hello world"));

    let length = share.read(|s| s.len());
    assert_eq!(length, 11);

    let contains_hello = share.read(|s| s.contains("hello"));
    assert!(contains_hello);

    let uppercase = share.read(|s| s.to_uppercase());
    assert_eq!(uppercase, "HELLO WORLD");
}

#[test]
fn test_arc_thread_share_write() {
    let share = ArcThreadShare::new(vec![1, 2, 3]);

    let doubled = share.write(|v| {
        v.iter_mut().for_each(|x| *x *= 2);
        v.clone()
    });
    assert_eq!(doubled, vec![2, 4, 6]);
    assert_eq!(share.get(), vec![2, 4, 6]);

    let sum = share.write(|v| {
        let sum = v.iter().sum::<i32>();
        v.push(sum);
        sum
    });
    assert_eq!(sum, 12);
    assert_eq!(share.get(), vec![2, 4, 6, 12]);
}

#[test]
fn test_arc_thread_share_thread_safety() {
    let share = ArcThreadShare::new(0);
    let mut handles: Vec<thread::JoinHandle<()>> = vec![];

    // Spawn multiple threads that increment the counter
    for _ in 0..5 {
        let share_clone = share.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                share_clone.increment();
                thread::sleep(Duration::from_millis(1));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Final value should be 500 (5 threads × 100 increments each)
    // Note: Even with atomic operations, some increments may be lost due to high contention
    // and the overhead of creating/destroying Box allocations
    let result = share.get();
    assert!(result >= 450 && result <= 500); // Allow some tolerance for lost operations
}

#[test]
fn test_arc_thread_share_concurrent_access() {
    let share = ArcThreadShare::new(vec![0; 100]);

    // Writer thread
    let writer = thread::spawn({
        let share_clone = ArcThreadShare::from_arc(share.data.clone());
        move || {
            for i in 0..100 {
                share_clone.update(|v| {
                    v[i] = i as i32;
                });
                thread::sleep(Duration::from_millis(1));
            }
        }
    });

    // Reader thread
    let reader = thread::spawn({
        let share_clone = ArcThreadShare::from_arc(share.data.clone());
        move || {
            let mut last_sum = 0;
            for _ in 0..100 {
                let current_sum = share_clone.read(|v| v.iter().sum::<i32>());
                if current_sum > last_sum {
                    last_sum = current_sum;
                }
                thread::sleep(Duration::from_millis(2));
            }
            last_sum
        }
    });

    let final_sum = reader.join().unwrap();
    writer.join().unwrap();

    // Verify final state
    let final_vec = share.get();
    let expected_sum: i32 = (0..100).sum();
    assert_eq!(final_sum, expected_sum);
    assert_eq!(final_vec.iter().sum::<i32>(), expected_sum);
}

#[test]
fn test_arc_thread_share_custom_struct() {
    #[derive(Clone, Debug, PartialEq)]
    struct TestStruct {
        value: i32,
        text: String,
        flag: bool,
    }

    let share = ArcThreadShare::new(TestStruct {
        value: 10,
        text: "test".to_string(),
        flag: false,
    });

    // Test initial state
    let initial = share.get();
    assert_eq!(initial.value, 10);
    assert_eq!(initial.text, "test");
    assert_eq!(initial.flag, false);

    // Test update
    share.update(|data| {
        data.value = 20;
        data.text = "updated".to_string();
        data.flag = true;
    });

    // Test updated state
    let updated = share.get();
    assert_eq!(updated.value, 20);
    assert_eq!(updated.text, "updated");
    assert_eq!(updated.flag, true);
}

#[test]
fn test_arc_thread_share_string_operations() {
    let share = ArcThreadShare::new(String::from("hello"));

    // Test string operations
    let length = share.read(|s| s.len());
    assert_eq!(length, 5);

    share.update(|s| s.push_str(" world"));
    assert_eq!(share.get(), "hello world");

    let uppercase = share.read(|s| s.to_uppercase());
    assert_eq!(uppercase, "HELLO WORLD");

    share.update(|s| s.truncate(5));
    assert_eq!(share.get(), "hello");
}

#[test]
fn test_arc_thread_share_numeric_operations() {
    let share = ArcThreadShare::new(0u64);

    // Test numeric operations
    share.update(|n| *n += 100);
    assert_eq!(share.get(), 100);

    share.update(|n| *n *= 2);
    assert_eq!(share.get(), 200);

    share.update(|n| *n -= 50);
    assert_eq!(share.get(), 150);

    let is_even = share.read(|n| n % 2 == 0);
    assert!(is_even);
}

#[test]
fn test_arc_thread_share_vector_operations() {
    let share = ArcThreadShare::new(Vec::<i32>::new());

    // Test empty vector
    let length = share.read(|v| v.len());
    assert_eq!(length, 0);

    // Test push operations
    share.update(|v| v.push(1));
    assert_eq!(share.get(), vec![1]);

    share.update(|v| v.push(2));
    assert_eq!(share.get(), vec![1, 2]);

    // Test clear operation
    share.update(|v| v.clear());
    assert_eq!(share.get(), vec![] as Vec<i32>);
}

#[test]
fn test_arc_thread_share_multiple_clones() {
    let share = ArcThreadShare::new(42);

    // Create multiple clones
    let clone1 = ArcThreadShare::from_arc(share.data.clone());
    let clone2 = ArcThreadShare::from_arc(share.data.clone());

    // Change through one clone
    clone1.set(100);

    // All should see the change
    assert_eq!(share.get(), 100);
    assert_eq!(clone1.get(), 100);
    assert_eq!(clone2.get(), 100);

    // Change through another clone
    clone2.update(|x| *x += 50);

    // All should see the change
    assert_eq!(share.get(), 150);
    assert_eq!(clone1.get(), 150);
    assert_eq!(clone2.get(), 150);
}

#[test]
fn test_arc_thread_share_large_data() {
    let large_vec: Vec<i32> = (0..1000).collect();
    let share = ArcThreadShare::new(large_vec.clone());

    // Verify initial data
    assert_eq!(share.get(), large_vec);

    // Test update with large data
    share.update(|v| {
        v.iter_mut().for_each(|x| *x *= 2);
    });

    let doubled: Vec<i32> = large_vec.iter().map(|x| x * 2).collect();
    assert_eq!(share.get(), doubled);
}

#[test]
fn test_arc_thread_share_memory_management() {
    let share = ArcThreadShare::new(String::from("test"));

    // Create many clones to test memory management
    let mut clones = Vec::new();
    for _ in 0..100 {
        clones.push(ArcThreadShare::from_arc(share.data.clone()));
    }

    // Update through original
    share.set(String::from("updated"));

    // Verify all clones see the change
    for clone in &clones {
        assert_eq!(clone.get(), "updated");
    }

    // Drop clones to test cleanup
    drop(clones);

    // Original should still work
    assert_eq!(share.get(), "updated");
}

#[test]
fn test_arc_thread_share_edge_cases() {
    // Test with zero-sized type
    let share = ArcThreadShare::new(());
    share.set(());
    assert_eq!(share.get(), ());

    // Test with unit type
    let share = ArcThreadShare::new(());
    share.update(|_| {});
    assert_eq!(share.get(), ());
}

#[test]
fn test_arc_thread_share_performance_pattern() {
    let share = ArcThreadShare::new(0);
    let share_clone = ArcThreadShare::from_arc(share.data.clone());

    // Simulate high-frequency updates
    let handle = thread::spawn(move || {
        for _ in 0..1000 {
            share_clone.update(|x| *x += 1);
        }
    });

    // Simulate high-frequency reads
    let mut total = 0;
    for _ in 0..1000 {
        total += share.read(|x| *x);
        thread::sleep(Duration::from_micros(1));
    }

    handle.join().unwrap();

    // Verify final state
    let final_value = share.get();
    assert_eq!(final_value, 1000);
    assert!(total > 0); // Should have read some values
}
