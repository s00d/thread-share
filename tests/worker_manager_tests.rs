use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thread_share::worker_manager::WorkerManager;

#[test]
fn test_worker_manager_new() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    assert_eq!(manager.active_workers(), 0);
    assert!(manager.get_worker_names().is_empty());
}

#[test]
fn test_add_worker() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager =  WorkerManager::new_with_threads(threads);
    
    // Add a worker
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
    });
    
    let result = manager.add_worker("test_worker", handle);
    assert!(result.is_ok());
    
    assert_eq!(manager.active_workers(), 1);
    assert_eq!(manager.get_worker_names(), vec!["test_worker"]);
}

#[test]
fn test_add_duplicate_worker() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Add first worker
    let handle1 = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
    });
    
    let result1 = manager.add_worker("test_worker", handle1);
    assert!(result1.is_ok());
    
    // Try to add worker with same name
    let handle2 = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
    });
    
    let result2 = manager.add_worker("test_worker", handle2);
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), "Worker 'test_worker' already exists");
    
    assert_eq!(manager.active_workers(), 1);
}

#[test]
fn test_remove_worker() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Add a worker
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
    });
    
    manager.add_worker("test_worker", handle).unwrap();
    assert_eq!(manager.active_workers(), 1);
    
    // Remove the worker
    let result = manager.remove_worker("test_worker");
    assert!(result.is_ok());
    
    assert_eq!(manager.active_workers(), 0);
    assert!(manager.get_worker_names().is_empty());
}

#[test]
fn test_remove_nonexistent_worker() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    let result = manager.remove_worker("nonexistent");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Worker 'nonexistent' not found");
}

#[test]
fn test_remove_all_workers() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Add multiple workers
    for i in 0..3 {
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
        });
        
        manager.add_worker(&format!("worker_{}", i), handle).unwrap();
    }
    
    assert_eq!(manager.active_workers(), 3);
    
    // Remove all workers
    let result = manager.remove_all_workers();
    assert!(result.is_ok());
    
    assert_eq!(manager.active_workers(), 0);
    assert!(manager.get_worker_names().is_empty());
}

#[test]
fn test_pause_and_resume_worker() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Add a worker
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
    });
    
    manager.add_worker("test_worker", handle).unwrap();
    
    // Initially not paused
    assert!(!manager.is_worker_paused("test_worker"));
    
    // Pause the worker
    let result = manager.pause_worker("test_worker");
    assert!(result.is_ok());
    assert!(manager.is_worker_paused("test_worker"));
    
    // Resume the worker
    let result = manager.resume_worker("test_worker");
    assert!(result.is_ok());
    assert!(!manager.is_worker_paused("test_worker"));
}

#[test]
fn test_worker_names() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Add workers with specific names
    let names = vec!["worker_a", "worker_b", "worker_c"];
    
    for name in &names {
        let handle = thread::spawn(|| {
            thread::sleep(Duration::from_millis(10));
        });
        
        manager.add_worker(name, handle).unwrap();
    }
    
    let worker_names = manager.get_worker_names();
    assert_eq!(worker_names.len(), 3);
    
    // Check that all names are present (order might vary)
    for name in &names {
        assert!(worker_names.contains(&name.to_string()));
    }
}

#[test]
fn test_active_workers_count() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    assert_eq!(manager.active_workers(), 0);
    
    // Add workers one by one
    for i in 0..5 {
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
        });
        
        manager.add_worker(&format!("worker_{}", i), handle).unwrap();
        assert_eq!(manager.active_workers(), i + 1);
    }
    
    // Remove some workers
    manager.remove_worker("worker_0").unwrap();
    assert_eq!(manager.active_workers(), 4);
    
    manager.remove_worker("worker_2").unwrap();
    assert_eq!(manager.active_workers(), 3);
    
    // Remove all remaining
    manager.remove_all_workers().unwrap();
    assert_eq!(manager.active_workers(), 0);
}

#[test]
fn test_join_all_workers() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Add workers that complete quickly
    for i in 0..3 {
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            println!("Worker {} completed", i);
        });
        
        manager.add_worker(&format!("worker_{}", i), handle).unwrap();
    }
    
    // Wait for all workers to complete
    let result = manager.join_all();
    assert!(result.is_ok());
    
    // All workers should be completed
    assert_eq!(manager.active_workers(), 0);
}

#[test]
fn test_clone_worker_manager() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Add a worker
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
    });
    
    manager.add_worker("test_worker", handle).unwrap();
    
    // Clone the manager
    let cloned_manager = manager.clone();
    
    // Both should have the same worker
    assert_eq!(manager.active_workers(), 1);
    assert_eq!(cloned_manager.active_workers(), 1);
    
    assert_eq!(manager.get_worker_names(), vec!["test_worker"]);
    assert_eq!(cloned_manager.get_worker_names(), vec!["test_worker"]);
}

#[test]
fn test_worker_manager_with_long_running_workers() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Add a long-running worker
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(100));
    });
    
    manager.add_worker("long_worker", handle).unwrap();
    
    // Check that worker is active
    assert_eq!(manager.active_workers(), 1);
    assert!(manager.get_worker_names().contains(&"long_worker".to_string()));
    
    // Wait for completion
    let result = manager.join_all();
    assert!(result.is_ok());
    
    // Worker should be completed
    assert_eq!(manager.active_workers(), 0);
}

#[test]
fn test_worker_manager_edge_cases() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = WorkerManager::new_with_threads(threads);
    
    // Test with empty name
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
    });
    
    let result = manager.add_worker("", handle);
    assert!(result.is_ok());
    
    // Test pause/resume with empty name
    assert!(manager.pause_worker("").is_ok());
    assert!(manager.is_worker_paused(""));
    assert!(manager.resume_worker("").is_ok());
    assert!(!manager.is_worker_paused(""));
    
    // Test remove with empty name
    assert!(manager.remove_worker("").is_ok());
    assert_eq!(manager.active_workers(), 0);
}

#[test]
fn test_worker_manager_concurrent_access() {
    let threads = Arc::new(Mutex::new(HashMap::new()));
    let manager = Arc::new(WorkerManager::new_with_threads(threads));
    
    // Spawn multiple threads that add workers concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let manager_clone = manager.clone();
        let handle = thread::spawn(move || {
            let worker_handle = thread::spawn(|| {
                thread::sleep(Duration::from_millis(10));
            });
            
            manager_clone.add_worker(&format!("concurrent_worker_{}", i), worker_handle)
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
    
    // All workers should be added
    assert_eq!(manager.active_workers(), 10);
    
    // Clean up
    manager.remove_all_workers().unwrap();
    assert_eq!(manager.active_workers(), 0);
}
