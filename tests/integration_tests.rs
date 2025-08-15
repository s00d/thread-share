use std::thread;
use std::time::Duration;
use thread_share::{share, simple_share, ArcThreadShare, ArcThreadShareLocked};

#[test]
fn test_producer_consumer_with_different_patterns() {
    let queue = share!(Vec::<String>::new());
    let counter = simple_share!(0);

    // Producer using ThreadShare
    let producer = thread::spawn({
        let queue_clone = queue.clone();
        move || {
            for i in 0..10 {
                queue_clone.update(|q| {
                    q.push(format!("item {}", i));
                });
                thread::sleep(Duration::from_millis(10));
            }
        }
    });

    // Consumer using SimpleShare
    let consumer = thread::spawn({
        let queue_clone = queue.clone();
        let counter_clone = counter.clone();
        move || {
            let mut processed = 0;
            while processed < 10 {
                let items = queue_clone.get();
                if !items.is_empty() {
                    queue_clone.update(|q| {
                        if let Some(_item) = q.pop() {
                            processed += 1;
                            counter_clone.update(|c| *c += 1);
                        }
                    });
                } else {
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    // Verify results
    assert_eq!(queue.get(), Vec::<String>::new());
    assert_eq!(counter.get(), 10);
}

#[test]
fn test_game_state_simulation() {
    #[derive(Clone, Debug, PartialEq)]
    struct GameState {
        player_health: i32,
        score: u32,
        level: u32,
        inventory: Vec<String>,
        is_game_over: bool,
    }

    let game_state = share!(GameState {
        player_health: 100,
        score: 0,
        level: 1,
        inventory: vec!["sword".to_string()],
        is_game_over: false,
    });

    // Health management thread using ThreadShare
    let health_thread = thread::spawn({
        let state_clone = game_state.clone();
        move || {
            for _ in 0..5 {
                state_clone.update(|state| {
                    state.player_health -= 10;
                    if state.player_health <= 0 {
                        state.is_game_over = true;
                    }
                });
                thread::sleep(Duration::from_millis(50));
            }
        }
    });

    // Score management thread using ArcThreadShare
    let score_thread = thread::spawn({
        let arc_data = game_state.as_arc();
        let arc_share = ArcThreadShare::from_arc(arc_data);
        move || {
            for _ in 0..5 {
                arc_share.update(|state| {
                    state.score += 100;
                    if state.score >= state.level * 500 {
                        state.level += 1;
                    }
                });
                thread::sleep(Duration::from_millis(40));
            }
        }
    });

    // Inventory management thread using ArcThreadShareLocked
    let inventory_thread = thread::spawn({
        let arc_locked_data = game_state.as_arc_locked();
        let locked_share = ArcThreadShareLocked::from_arc(arc_locked_data);
        move || {
            for i in 0..5 {
                locked_share.update(|state| {
                    state.inventory.push(format!("item_{}", i));
                });
                thread::sleep(Duration::from_millis(30));
            }
        }
    });

    // Wait for all threads
    health_thread.join().unwrap();
    score_thread.join().unwrap();
    inventory_thread.join().unwrap();

    // Verify final game state
    let final_state = game_state.get();
    assert!(final_state.player_health <= 50);
    // Note: score and level changes are in independent ArcThreadShare copies
    // so they won't affect the main game_state
    assert_eq!(final_state.score, 0); // Original value
    assert_eq!(final_state.level, 1); // Original value
                                      // inventory_thread uses as_arc_locked, so changes should be visible
    assert_eq!(final_state.inventory.len(), 6); // 1 initial + 5 new
    assert!(!final_state.is_game_over); // Health should not reach 0

    // Check the independent copies
    let arc_data = game_state.as_arc();
    let arc_share = ArcThreadShare::from_arc(arc_data);
    let arc_state = arc_share.get();
    // as_arc creates independent copy, so score and level should be original values
    assert_eq!(arc_state.score, 0);
    assert_eq!(arc_state.level, 1);
}

#[test]
fn test_concurrent_readers_writers() {
    let data = share!(vec![0; 1000]);
    let read_count = simple_share!(0u32);
    let write_count = simple_share!(0u32);

    // Multiple writer threads
    let mut writer_handles = vec![];
    for writer_id in 0..5 {
        let data_clone = data.clone();
        let write_count_clone = write_count.clone();
        let handle = thread::spawn(move || {
            for i in 0..200 {
                let index = (writer_id * 200 + i) % 1000;
                data_clone.update(|v| v[index] = writer_id as i32);
                write_count_clone.update(|c| *c += 1);
                thread::sleep(Duration::from_millis(1));
            }
        });
        writer_handles.push(handle);
    }

    // Multiple reader threads
    let mut reader_handles = vec![];
    for _ in 0..10 {
        let data_clone = data.clone();
        let read_count_clone = read_count.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _sum = data_clone.read(|v| v.iter().sum::<i32>());
                read_count_clone.update(|c| *c += 1);
                thread::sleep(Duration::from_millis(1));
            }
        });
        reader_handles.push(handle);
    }

    // Wait for all threads
    for handle in writer_handles {
        handle.join().unwrap();
    }
    for handle in reader_handles {
        handle.join().unwrap();
    }

    // Verify results
    assert_eq!(write_count.get(), 1000); // 5 writers × 200 writes each
    assert_eq!(read_count.get(), 1000); // 10 readers × 100 reads each

    // Verify data integrity
    let final_data = data.get();
    assert_eq!(final_data.len(), 1000);

    // All values should be between 0 and 4 (writer IDs)
    for &value in &final_data {
        assert!(value >= 0 && value <= 4);
    }
}

#[test]
fn test_memory_pressure_scenario() {
    // Test library behavior under memory pressure
    let large_data = share!(vec![0u8; 10000]);
    let mut handles = vec![];

    // Create many threads that access the data
    for _ in 0..20 {
        let data_clone = large_data.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _value = data_clone.get();
                thread::sleep(Duration::from_micros(100));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify data integrity
    let final_data = large_data.get();
    assert_eq!(final_data.len(), 10000);
    assert_eq!(final_data.iter().sum::<u8>(), 0);
}

#[test]
fn test_error_handling_scenarios() {
    // Test that the library handles edge cases gracefully

    // Test with very large data
    let large_string = "x".repeat(100000);
    let large_share = share!(large_string.clone());
    assert_eq!(large_share.get(), large_string);

    // Test with many small operations
    let counter = share!(0);
    for _ in 0..10000 {
        counter.update(|c| *c += 1);
    }
    assert_eq!(counter.get(), 10000);

    // Test rapid clone operations
    let original = share!(42);
    let mut clones = Vec::new();
    for _ in 0..1000 {
        clones.push(original.clone());
    }

    // Update through original
    original.set(100);

    // Verify all clones see the change
    for clone in &clones {
        assert_eq!(clone.get(), 100);
    }

    // Drop clones to test cleanup
    drop(clones);

    // Original should still work
    assert_eq!(original.get(), 100);
}
