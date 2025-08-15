use std::thread;
use std::time::Duration;
use thread_share::{share, simple_share, ArcThreadShare, SimpleShare, ThreadShare};

#[test]
fn test_thread_share_new() {
    let share = ThreadShare::new(42);
    assert_eq!(share.get(), 42);
}

#[test]
fn test_thread_share_set_get() {
    let share = share!(100);
    assert_eq!(share.get(), 100);

    share.set(200);
    assert_eq!(share.get(), 200);

    share.set(300);
    assert_eq!(share.get(), 300);
}

#[test]
fn test_thread_share_update() {
    let share = share!(vec![1, 2, 3]);

    share.update(|v| v.push(4));
    assert_eq!(share.get(), vec![1, 2, 3, 4]);

    share.update(|v| {
        v[0] = 100;
        v.push(5);
    });
    assert_eq!(share.get(), vec![100, 2, 3, 4, 5]);
}

#[test]
fn test_thread_share_read() {
    let share = share!(String::from("hello world"));

    let length = share.read(|s| s.len());
    assert_eq!(length, 11);

    let contains_hello = share.read(|s| s.contains("hello"));
    assert!(contains_hello);

    let uppercase = share.read(|s| s.to_uppercase());
    assert_eq!(uppercase, "HELLO WORLD");
}

#[test]
fn test_thread_share_write() {
    let share = share!(vec![1, 2, 3]);

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
fn test_thread_share_wait_for_change() {
    let data = share!(false);
    let data_clone = data.clone();

    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        data_clone.set(true);
    });

    // Wait for change with timeout - should timeout
    let timeout_occurred = data.wait_for_change(Duration::from_millis(50));
    assert!(timeout_occurred);

    // Wait for real change - should not timeout
    let timeout_occurred = data.wait_for_change(Duration::from_millis(200));
    assert!(!timeout_occurred);

    handle.join().unwrap();
}

#[test]
fn test_thread_share_wait_for_change_forever() {
    let data = share!(0);
    let data_clone = data.clone();

    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        data_clone.set(42);
    });

    // This should not block indefinitely
    data.wait_for_change_forever();
    assert_eq!(data.get(), 42);

    handle.join().unwrap();
}

#[test]
fn test_thread_share_clone() {
    let original = share!(String::from("original"));
    let clone1 = original.clone();
    let clone2 = original.clone();

    // Change through one clone
    clone1.set(String::from("changed"));

    // All clones should see the change
    assert_eq!(original.get(), "changed");
    assert_eq!(clone1.get(), "changed");
    assert_eq!(clone2.get(), "changed");

    // Change through another clone
    clone2.update(|s| s.push_str(" again"));

    // All clones should see the change
    assert_eq!(original.get(), "changed again");
    assert_eq!(clone1.get(), "changed again");
    assert_eq!(clone2.get(), "changed again");
}

#[test]
fn test_thread_share_as_arc() {
    let share = share!(42);
    let arc_data = share.as_arc();

    // Verify we can still use the original
    assert_eq!(share.get(), 42);

    // Verify the Arc contains the same initial data using ArcThreadShare
    let thread_share = ArcThreadShare::from_arc(arc_data.clone());
    let value = thread_share.get();
    assert_eq!(value, 42);

    // Update through original
    share.set(100);

    // Verify Arc still has the old value (as_arc creates independent copy)
    let value = thread_share.get();
    assert_eq!(value, 42);

    // Update through ArcThreadShare
    thread_share.set(200);
    let value = thread_share.get();
    assert_eq!(value, 200);

    // Original share is unchanged
    assert_eq!(share.get(), 100);
}

#[test]
fn test_simple_share_new() {
    let share = SimpleShare::new("hello");
    assert_eq!(share.get(), "hello");
}

#[test]
fn test_simple_share_set_get() {
    let share = simple_share!("initial");
    assert_eq!(share.get(), "initial");

    share.set("updated");
    assert_eq!(share.get(), "updated");

    share.set("final");
    assert_eq!(share.get(), "final");
}

#[test]
fn test_simple_share_update() {
    let share = simple_share!(vec![1, 2, 3]);

    share.update(|v| v.push(4));
    assert_eq!(share.get(), vec![1, 2, 3, 4]);

    share.update(|v| {
        v[0] = 100;
        v.push(5);
    });
    assert_eq!(share.get(), vec![100, 2, 3, 4, 5]);
}

#[test]
fn test_simple_share_clone() {
    let original = simple_share!(42);
    let clone1 = original.clone();
    let clone2 = original.clone();

    // Change through one clone
    clone1.set(100);

    // All clones should see the change
    assert_eq!(original.get(), 100);
    assert_eq!(clone1.get(), 100);
    assert_eq!(clone2.get(), 100);
}

#[test]
fn test_custom_struct() {
    #[derive(Clone, Debug, PartialEq)]
    struct TestStruct {
        value: i32,
        text: String,
        flag: bool,
    }

    let test_data = share!(TestStruct {
        value: 10,
        text: "test".to_string(),
        flag: false,
    });

    let test_data_clone = test_data.clone();

    // Change in thread
    let handle = thread::spawn(move || {
        test_data_clone.update(|data| {
            data.value = 20;
            data.text = "updated".to_string();
            data.flag = true;
        });
    });

    // Check original state
    let original = test_data.get();
    assert_eq!(original.value, 10);
    assert_eq!(original.text, "test");
    assert_eq!(original.flag, false);

    handle.join().unwrap();

    // Check updated state
    let updated = test_data.get();
    assert_eq!(updated.value, 20);
    assert_eq!(updated.text, "updated");
    assert_eq!(updated.flag, true);
}

#[test]
fn test_thread_safety_multiple_threads() {
    let share = share!(0);
    let mut handles = vec![];

    // Spawn multiple threads that increment the counter
    for _ in 0..5 {
        let share_clone = share.clone();
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
    assert_eq!(share.get(), 500);
}

#[test]
fn test_concurrent_read_write() {
    let share = share!(vec![0; 100]);
    let share_clone = share.clone();

    // Writer thread
    let writer = thread::spawn(move || {
        for i in 0..100 {
            share_clone.update(|v| {
                v[i] = i as i32;
            });
            thread::sleep(Duration::from_millis(1));
        }
    });

    // Reader thread
    let reader = thread::spawn({
        let share_clone = share.clone();
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
fn test_macro_share() {
    let share = share!(42);
    assert_eq!(share.get(), 42);

    let share = share!("hello");
    assert_eq!(share.get(), "hello");

    let share = share!(vec![1, 2, 3]);
    assert_eq!(share.get(), vec![1, 2, 3]);
}

#[test]
fn test_macro_simple_share() {
    let share = simple_share!(42);
    assert_eq!(share.get(), 42);

    let share = simple_share!("hello");
    assert_eq!(share.get(), "hello");

    let share = simple_share!(vec![1, 2, 3]);
    assert_eq!(share.get(), vec![1, 2, 3]);
}

#[test]
fn test_empty_vector_operations() {
    let share = share!(Vec::<i32>::new());

    // Test operations on empty vector
    let length = share.read(|v| v.len());
    assert_eq!(length, 0);

    share.update(|v| v.push(1));
    assert_eq!(share.get(), vec![1]);

    share.update(|v| v.clear());
    assert_eq!(share.get(), vec![] as Vec<i32>);
}

#[test]
fn test_string_operations() {
    let share = share!(String::from("hello"));

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
fn test_numeric_operations() {
    let share = share!(0u64);

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
