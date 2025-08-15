use std::thread;
use std::time::Duration;
use thread_share::{share, ArcThreadShare, ArcThreadShareLocked};

#[derive(Clone, Debug)]
struct GameState {
    score: u32,
    level: u32,
}

impl GameState {
    fn new() -> Self {
        Self { score: 0, level: 1 }
    }

    fn add_score(&mut self, points: u32) {
        self.score += points;
        if self.score >= self.level * 100 {
            self.level += 1;
        }
    }
}

fn main() {
    println!("=== Example WITHOUT cloning ===");

    // Option 1: WITHOUT locks (AtomicPtr) - faster but less safe
    println!("\n--- Option 1: WITHOUT locks (AtomicPtr) ---");
    let game_state = share!(GameState::new());

    // Get Arc<AtomicPtr<T>> and create ArcThreadShare for thread
    let arc_data = game_state.as_arc();
    let thread_share = ArcThreadShare::from_arc(arc_data.clone());

    // Create clone for main thread
    let thread_share_main = ArcThreadShare::from_arc(arc_data);

    // Thread for updating score - WITHOUT locks!
    let score_handle = thread::spawn(move || {
        for _ in 1..=5 {
            thread::sleep(Duration::from_millis(200));
            thread_share.update(|state: &mut GameState| {
                state.add_score(25);
                println!(
                    "Score thread (AtomicPtr): +25 points, current score: {}",
                    state.score
                );
            });
        }
    });

    // Main thread reads values from AtomicPtr version
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(300));
        let current_state = thread_share_main.get();
        println!(
            "Player (AtomicPtr): Level {}, Score: {}",
            current_state.level, current_state.score
        );
    }

    score_handle.join().unwrap();

    // Option 2: WITH locks (RwLock) - slower but safer
    println!("\n--- Option 2: WITH locks (RwLock) ---");
    let game_state_locked = share!(GameState::new());

    // Get Arc<RwLock<T>> and create ArcThreadShareLocked for thread
    let arc_data_locked = game_state_locked.as_arc_locked();
    let thread_share_locked = ArcThreadShareLocked::from_arc(arc_data_locked);

    // Thread for updating score - WITH locks!
    let score_handle_locked = thread::spawn(move || {
        for _ in 1..=5 {
            thread::sleep(Duration::from_millis(200));
            thread_share_locked.update(|state: &mut GameState| {
                state.add_score(25);
                println!(
                    "Score thread (RwLock): +25 points, current score: {}",
                    state.score
                );
            });
        }
    });

    // Main thread reads values
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(300));
        let current_state = game_state_locked.get();
        println!(
            "Player (RwLock): Level {}, Score: {}",
            current_state.level, current_state.score
        );
    }

    score_handle_locked.join().unwrap();

    // Comparison of results
    println!("\n--- Comparison of results ---");
    let final_state_atomic = game_state.get();
    let final_state_locked = game_state_locked.get();

    println!("AtomicPtr (without locks): {:?}", final_state_atomic);
    println!("RwLock (with locks): {:?}", final_state_locked);

    println!("\n✅ Successfully used library WITHOUT cloning!");
    println!("🎯 AtomicPtr - faster but less safe");
    println!("🔒 RwLock - slower but safer");
}
