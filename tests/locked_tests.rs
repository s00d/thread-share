use std::thread;
use std::time::Duration;
use thread_share::{share, ArcThreadShareLocked};

#[test]
fn test_arc_thread_share_locked_from_arc() {
    let original = share!(100);
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    assert_eq!(locked_share.get(), 100);
}

#[test]
fn test_arc_thread_share_locked_set_get() {
    let original = share!(42);
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    locked_share.set(100);
    assert_eq!(locked_share.get(), 100);

    locked_share.set(200);
    assert_eq!(locked_share.get(), 200);

    locked_share.set(300);
    assert_eq!(locked_share.get(), 300);
}

#[test]
fn test_arc_thread_share_locked_update() {
    let original = share!(vec![1, 2, 3]);
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    locked_share.update(|v| v.push(4));
    assert_eq!(locked_share.get(), vec![1, 2, 3, 4]);

    locked_share.update(|v| {
        v[0] = 100;
        v.push(5);
    });
    assert_eq!(locked_share.get(), vec![100, 2, 3, 4, 5]);
}

#[test]
fn test_arc_thread_share_locked_read() {
    let original = share!(String::from("hello world"));
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    let length = locked_share.read(|s| s.len());
    assert_eq!(length, 11);

    let contains_hello = locked_share.read(|s| s.contains("hello"));
    assert!(contains_hello);

    let uppercase = locked_share.read(|s| s.to_uppercase());
    assert_eq!(uppercase, "HELLO WORLD");
}

#[test]
fn test_arc_thread_share_locked_write() {
    let original = share!(vec![1, 2, 3]);
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    let doubled = locked_share.write(|v| {
        v.iter_mut().for_each(|x| *x *= 2);
        v.clone()
    });
    assert_eq!(doubled, vec![2, 4, 6]);
    assert_eq!(locked_share.get(), vec![2, 4, 6]);

    let sum = locked_share.write(|v| {
        let sum = v.iter().sum::<i32>();
        v.push(sum);
        sum
    });
    assert_eq!(sum, 12);
    assert_eq!(locked_share.get(), vec![2, 4, 6, 12]);
}

#[test]
fn test_arc_thread_share_locked_thread_safety() {
    let original = share!(0);
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data.clone());
    let mut handles = vec![];

    // Spawn multiple threads that increment the counter
    for _ in 0..5 {
        let share_clone = ArcThreadShareLocked::from_arc(arc_data.clone());
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                share_clone.update(|x| *x += 1);
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
    assert_eq!(locked_share.get(), 500);
}

#[test]
fn test_arc_thread_share_locked_concurrent_access() {
    let original = share!(vec![0; 100]);
    let arc_data = original.as_arc_locked();

    // Writer thread
    let writer = thread::spawn({
        let share_clone = ArcThreadShareLocked::from_arc(arc_data.clone());
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
        let share_clone = ArcThreadShareLocked::from_arc(arc_data.clone());
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
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);
    let final_vec = locked_share.get();
    let expected_sum: i32 = (0..100).sum();
    assert_eq!(final_sum, expected_sum);
    assert_eq!(final_vec.iter().sum::<i32>(), expected_sum);
}

#[test]
fn test_arc_thread_share_locked_custom_struct() {
    #[derive(Clone, Debug, PartialEq)]
    struct TestStruct {
        value: i32,
        text: String,
        flag: bool,
    }

    let original = share!(TestStruct {
        value: 10,
        text: "test".to_string(),
        flag: false,
    });

    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    // Test initial state
    let initial = locked_share.get();
    assert_eq!(initial.value, 10);
    assert_eq!(initial.text, "test");
    assert_eq!(initial.flag, false);

    // Test update
    locked_share.update(|data| {
        data.value = 20;
        data.text = "updated".to_string();
        data.flag = true;
    });

    // Test updated state
    let updated = locked_share.get();
    assert_eq!(updated.value, 20);
    assert_eq!(updated.text, "updated");
    assert_eq!(updated.flag, true);
}

#[test]
fn test_arc_thread_share_locked_string_operations() {
    let original = share!(String::from("hello"));
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    // Test string operations
    let length = locked_share.read(|s| s.len());
    assert_eq!(length, 5);

    locked_share.update(|s| s.push_str(" world"));
    assert_eq!(locked_share.get(), "hello world");

    let uppercase = locked_share.read(|s| s.to_uppercase());
    assert_eq!(uppercase, "HELLO WORLD");

    locked_share.update(|s| s.truncate(5));
    assert_eq!(locked_share.get(), "hello");
}

#[test]
fn test_arc_thread_share_locked_numeric_operations() {
    let original = share!(0u64);
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    // Test numeric operations
    locked_share.update(|n| *n += 100);
    assert_eq!(locked_share.get(), 100);

    locked_share.update(|n| *n *= 2);
    assert_eq!(locked_share.get(), 200);

    locked_share.update(|n| *n -= 50);
    assert_eq!(locked_share.get(), 150);

    let is_even = locked_share.read(|n| n % 2 == 0);
    assert!(is_even);
}

#[test]
fn test_arc_thread_share_locked_vector_operations() {
    let original = share!(Vec::<i32>::new());
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    // Test empty vector
    let length = locked_share.read(|v| v.len());
    assert_eq!(length, 0);

    // Test push operations
    locked_share.update(|v| v.push(1));
    assert_eq!(locked_share.get(), vec![1]);

    locked_share.update(|v| v.push(2));
    assert_eq!(locked_share.get(), vec![1, 2]);

    // Test clear operation
    locked_share.update(|v| v.clear());
    assert_eq!(locked_share.get(), vec![] as Vec<i32>);
}

#[test]
fn test_arc_thread_share_locked_multiple_clones() {
    let original = share!(42);
    let arc_data = original.as_arc_locked();

    // Create multiple clones
    let clone1 = ArcThreadShareLocked::from_arc(arc_data.clone());
    let clone2 = ArcThreadShareLocked::from_arc(arc_data.clone());

    // Change through one clone
    clone1.set(100);

    // All should see the change
    assert_eq!(clone1.get(), 100);
    assert_eq!(clone2.get(), 100);

    // Change through another clone
    clone2.update(|x| *x += 50);

    // All should see the change
    assert_eq!(clone1.get(), 150);
    assert_eq!(clone2.get(), 150);
}

#[test]
fn test_arc_thread_share_locked_large_data() {
    let large_vec: Vec<i32> = (0..1000).collect();
    let original = share!(large_vec.clone());
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    // Verify initial data
    assert_eq!(locked_share.get(), large_vec);

    // Test update with large data
    locked_share.update(|v| {
        v.iter_mut().for_each(|x| *x *= 2);
    });

    let doubled: Vec<i32> = large_vec.iter().map(|x| x * 2).collect();
    assert_eq!(locked_share.get(), doubled);
}

#[test]
fn test_arc_thread_share_locked_memory_management() {
    let original = share!(String::from("test"));
    let arc_data = original.as_arc_locked();

    // Create many clones to test memory management
    let mut clones = Vec::new();
    for _ in 0..100 {
        clones.push(ArcThreadShareLocked::from_arc(arc_data.clone()));
    }

    // Update through one clone
    clones[0].set(String::from("updated"));

    // Verify all clones see the change
    for clone in &clones {
        assert_eq!(clone.get(), "updated");
    }

    // Drop clones to test cleanup
    drop(clones);

    // Original should still work
    assert_eq!(original.get(), "updated");
}

#[test]
fn test_arc_thread_share_locked_edge_cases() {
    // Test with zero-sized type
    let original = share!(());
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    locked_share.set(());
    assert_eq!(locked_share.get(), ());

    // Test with unit type
    locked_share.update(|_| {});
    assert_eq!(locked_share.get(), ());
}

#[test]
fn test_arc_thread_share_locked_performance_pattern() {
    let original = share!(0);
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data.clone());
    let share_clone = ArcThreadShareLocked::from_arc(arc_data);

    // Simulate high-frequency updates
    let handle = thread::spawn(move || {
        for _ in 0..1000 {
            share_clone.update(|x| *x += 1);
        }
    });

    // Simulate high-frequency reads
    let mut total = 0;
    for _ in 0..1000 {
        total += locked_share.read(|x| *x);
        thread::sleep(Duration::from_micros(1));
    }

    handle.join().unwrap();

    // Verify final state
    let final_value = locked_share.get();
    assert_eq!(final_value, 1000);
    assert!(total > 0); // Should have read some values
}

#[test]
fn test_arc_thread_share_locked_sync_with_original() {
    let original = share!(42);
    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    // Change through original
    original.set(100);

    // Locked share should see the change
    assert_eq!(locked_share.get(), 100);

    // Change through locked share
    locked_share.set(200);

    // Original should see the change
    assert_eq!(original.get(), 200);

    // Both should be in sync
    assert_eq!(original.get(), locked_share.get());
}

#[test]
fn test_arc_thread_share_locked_complex_nested_operations() {
    #[derive(Clone, Debug, PartialEq)]
    struct ComplexStruct {
        numbers: Vec<i32>,
        metadata: String,
        flags: Vec<bool>,
    }

    let original = share!(ComplexStruct {
        numbers: vec![1, 2, 3],
        metadata: "initial".to_string(),
        flags: vec![false, false, false],
    });

    let arc_data = original.as_arc_locked();
    let locked_share = ArcThreadShareLocked::from_arc(arc_data);

    // Complex nested update
    locked_share.update(|data| {
        // Update numbers
        data.numbers.iter_mut().for_each(|n| *n *= 2);
        data.numbers.push(8);

        // Update metadata
        data.metadata.push_str("_updated");

        // Update flags
        data.flags.iter_mut().for_each(|f| *f = true);
        data.flags.push(false);
    });

    // Verify complex update
    let result = locked_share.get();
    assert_eq!(result.numbers, vec![2, 4, 6, 8]);
    assert_eq!(result.metadata, "initial_updated");
    assert_eq!(result.flags, vec![true, true, true, false]);
}
