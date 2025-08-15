use std::time::Duration;
use std::thread;
use thread_share::{enhanced_share, spawn_workers, worker_manager::WorkerManager};

/// Example demonstrating WorkerManager functionality
/// 
/// This example shows how to use the WorkerManager returned by spawn_workers!
/// to control individual workers: pause, resume, stop, and monitor them.
/// Also demonstrates creating WorkerManager directly and adding workers programmatically.
fn main() {
    println!("=== Worker Management Example ===");
    println!("Demonstrating WorkerManager functionality...\n");

    // Create shared data
    let data = enhanced_share!(0u32);

    println!("🚀 Starting workers with spawn_workers!...");
    
    // Start initial workers and get WorkerManager
    let manager = spawn_workers!(data, {
        counter: |data: thread_share::ThreadShare<u32>| {
            println!("🔢 Counter worker started");
            for i in 1..=10 {
                data.update(|x| *x += i);
                println!("  Counter: {}", data.get());
                std::thread::sleep(Duration::from_millis(500));
            }
            println!("🔢 Counter worker finished");
        },

        monitor: |data: thread_share::ThreadShare<u32>| {
            println!("📊 Monitor worker started");
            for _ in 0..5 {
                std::thread::sleep(Duration::from_millis(1000));
                println!("  📊 Current value: {}", data.get());
            }
            println!("📊 Monitor worker finished");
        }
    });

    println!("✅ Initial workers started!");
    println!("📋 Worker names: {:?}", manager.get_worker_names());
    println!("🔢 Active workers: {}", manager.active_workers());

    // Demonstrate creating WorkerManager directly
    println!("\n🔨 Creating additional WorkerManager directly...");
    let direct_manager = WorkerManager::new_with_threads(data.get_threads());
    
    // Add processor worker programmatically
    let data_clone1 = data.clone();
    let processor_handle = thread::spawn(move || {
        println!("⚙️ Processor worker started (added programmatically)");
        for _ in 0..3 {
            std::thread::sleep(Duration::from_millis(800));
            let current = data_clone1.get();
            data_clone1.update(|x| *x *= 2);
            println!("  ⚙️ Processed: {} -> {}", current, data_clone1.get());
        }
        println!("⚙️ Processor worker finished");
    });

    // Add to direct manager
    if let Err(e) = direct_manager.add_worker("processor", processor_handle) {
        println!("❌ Failed to add processor: {}", e);
    } else {
        println!("✅ Processor worker added to direct manager");
    }

    // Add multiplier worker programmatically to main manager
    let data_clone2 = data.clone();
    let multiplier_handle = thread::spawn(move || {
        println!("✖️ Multiplier worker started (added programmatically)");
        for i in 1..=4 {
            std::thread::sleep(Duration::from_millis(600));
            let current = data_clone2.get();
            data_clone2.update(|x| *x *= i);
            println!("  ✖️ Multiplied by {}: {} -> {}", i, current, data_clone2.get());
        }
        println!("✖️ Multiplier worker finished");
    });

    // Add to main manager
    if let Err(e) = manager.add_worker("multiplier", multiplier_handle) {
        println!("❌ Failed to add multiplier: {}", e);
    } else {
        println!("✅ Multiplier worker added to main manager");
    }

    println!("📋 Updated worker names: {:?}", manager.get_worker_names());
    println!("🔢 Active workers in main manager: {}", manager.active_workers());
    println!("🔢 Active workers in direct manager: {}", direct_manager.active_workers());

    // Demonstrate worker management
    println!("\n🎮 Demonstrating worker management...");

    // Pause a worker
    println!("\n⏸️ Pausing counter worker...");
    if let Err(e) = manager.pause_worker("counter") {
        println!("❌ Failed to pause counter: {}", e);
    }

    // Check if worker is paused
    if manager.is_worker_paused("counter") {
        println!("✅ Counter worker is paused");
    }

    // Resume a worker
    println!("\n▶️ Resuming counter worker...");
    if let Err(e) = manager.resume_worker("counter") {
        println!("❌ Failed to resume counter: {}", e);
    }

    // Programmatically stop a worker by removing it
    println!("\n🛑 Stopping multiplier worker by removing it...");
    if let Err(e) = manager.remove_worker("multiplier") {
        println!("❌ Failed to remove multiplier: {}", e);
    } else {
        println!("✅ Multiplier worker stopped and removed");
    }

    println!("🔢 Active workers after stopping multiplier: {}", manager.active_workers());

    // Add a new worker after stopping another
    let data_clone3 = data.clone();
    let replacement_handle = thread::spawn(move || {
        println!("🔄 Replacement worker started (replacing stopped worker)");
        for _ in 0..2 {
            std::thread::sleep(Duration::from_millis(700));
            let current = data_clone3.get();
            data_clone3.update(|x| *x += 100);
            println!("  🔄 Added 100: {} -> {}", current, data_clone3.get());
        }
        println!("🔄 Replacement worker finished");
    });

    if let Err(e) = manager.add_worker("replacement", replacement_handle) {
        println!("❌ Failed to add replacement worker: {}", e);
    } else {
        println!("✅ Replacement worker added successfully");
    }

    // Remove a worker from tracking
    println!("\n🗑️ Removing processor worker from direct manager...");
    if let Err(e) = direct_manager.remove_worker("processor") {
        println!("❌ Failed to remove processor: {}", e);
    }

    println!("🔢 Active workers in direct manager after removal: {}", direct_manager.active_workers());

    // Wait for remaining workers to complete
    println!("\n⏳ Waiting for remaining workers to complete...");
    if let Err(e) = manager.join_all() {
        println!("❌ Some workers failed: {}", e);
    } else {
        println!("✅ All workers in main manager completed successfully!");
    }

    // Wait for direct manager workers
    if let Err(e) = direct_manager.join_all() {
        println!("❌ Some workers in direct manager failed: {}", e);
    } else {
        println!("✅ All workers in direct manager completed successfully!");
    }

    println!("🔢 Final active workers in main manager: {}", manager.active_workers());
    println!("🔢 Final active workers in direct manager: {}", direct_manager.active_workers());
    println!("📊 Final data value: {}", data.get());
    println!("\n🎉 Worker management example completed!");
}
