use std::thread;
use std::time::Duration;
use thread_share::{share, simple_share};

#[test]
fn test_basic_operations() {
    let share = share!(42);
    assert_eq!(share.get(), 42);

    share.set(100);
    assert_eq!(share.get(), 100);

    share.update(|x| *x += 50);
    assert_eq!(share.get(), 150);
}

#[test]
fn test_thread_safety() {
    let share = share!(0);
    let share_clone = share.clone();

    let handle = thread::spawn(move || {
        for i in 1..=100 {
            share_clone.set(i);
            thread::sleep(Duration::from_millis(1));
        }
    });

    let mut last_value = 0;
    for _ in 0..100 {
        let current = share.get();
        if current > last_value {
            last_value = current;
        }
        thread::sleep(Duration::from_millis(1));
    }

    handle.join().unwrap();
    // Wait a bit to ensure the last value is set
    thread::sleep(Duration::from_millis(10));
    let final_value = share.get();
    assert_eq!(final_value, 100);
}

#[test]
fn test_simple_share() {
    let share = simple_share!("hello");
    assert_eq!(share.get(), "hello");

    share.set("world");
    assert_eq!(share.get(), "world");

    share.update(|s| *s = "rust");
    assert_eq!(share.get(), "rust");
}

#[test]
fn test_read_write_operations() {
    let share = share!(vec![1, 2, 3]);

    // Test read operation
    let sum = share.read(|v| v.iter().sum::<i32>());
    assert_eq!(sum, 6);

    // Test write operation
    let doubled = share.write(|v| {
        v.iter_mut().for_each(|x| *x *= 2);
        v.clone()
    });
    assert_eq!(doubled, vec![2, 4, 6]);
    assert_eq!(share.get(), vec![2, 4, 6]);
}

#[test]
fn test_wait_for_change() {
    let data = share!(false);
    let data_clone = data.clone();

    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        data_clone.set(true);
    });

    // Wait for change with timeout
    let timeout_occurred = data.wait_for_change(Duration::from_millis(50));
    assert!(timeout_occurred); // Should timeout

    // Wait for real change
    let timeout_occurred = data.wait_for_change(Duration::from_millis(200));
    assert!(!timeout_occurred); // Should not timeout

    handle.join().unwrap();
}

#[test]
fn test_multiple_clones() {
    let original = share!(0);
    let clone1 = original.clone();
    let clone2 = original.clone();

    // Change through one clone
    clone1.set(42);

    // Check that all clones see changes
    assert_eq!(original.get(), 42);
    assert_eq!(clone1.get(), 42);
    assert_eq!(clone2.get(), 42);

    // Change through another clone
    clone2.update(|x| *x += 8);

    // Check that all clones see changes
    assert_eq!(original.get(), 50);
    assert_eq!(clone1.get(), 50);
    assert_eq!(clone2.get(), 50);
}

#[test]
fn test_custom_struct() {
    #[derive(Clone, Debug, PartialEq)]
    struct TestStruct {
        value: i32,
        text: String,
    }

    let test_data = share!(TestStruct {
        value: 10,
        text: "test".to_string(),
    });

    let test_data_clone = test_data.clone();

    // Change in thread
    let handle = thread::spawn(move || {
        test_data_clone.update(|data| {
            data.value = 20;
            data.text = "updated".to_string();
        });
    });

    // Check original state
    let original = test_data.get();
    assert_eq!(original.value, 10);
    assert_eq!(original.text, "test");

    handle.join().unwrap();

    // Check updated state
    let updated = test_data.get();
    assert_eq!(updated.value, 20);
    assert_eq!(updated.text, "updated");
}
