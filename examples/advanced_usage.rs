use std::thread;
use std::time::Duration;
use thread_share::share;

#[derive(Clone, Debug)]
struct GameState {
    score: u32,
    level: u32,
    is_game_over: bool,
}

impl GameState {
    fn new() -> Self {
        Self {
            score: 0,
            level: 1,
            is_game_over: false,
        }
    }

    fn add_score(&mut self, points: u32) {
        self.score += points;
        if self.score >= self.level * 100 {
            self.level += 1;
        }
    }

    fn game_over(&mut self) {
        self.is_game_over = true;
    }
}

fn main() {
    println!("=== Advanced Game Example with ThreadShare ===");

    let game_state = share!(GameState::new());
    let game_state_clone = game_state.clone();

    // Thread for updating score
    let score_handle = thread::spawn(move || {
        for _ in 1..=10 {
            thread::sleep(Duration::from_millis(200));
            game_state_clone.update(|state| {
                state.add_score(25);
                println!("Score thread: +25 points, current score: {}", state.score);
            });
        }
    });

    // Thread for monitoring state
    let monitor_handle = {
        let game_state_clone = game_state.clone();
        thread::spawn(move || {
            let mut last_level = 1;
            loop {
                let current_state = game_state_clone.get();
                if current_state.is_game_over {
                    break;
                }

                if current_state.level > last_level {
                    println!("🎉 Level increased to {}!", current_state.level);
                    last_level = current_state.level;
                }

                thread::sleep(Duration::from_millis(100));
            }
        })
    };

    // Main thread - player
    for i in 1..=10 {
        thread::sleep(Duration::from_millis(300));

        let current_state = game_state.get();
        println!(
            "Player: Level {}, Score: {}",
            current_state.level, current_state.score
        );

        if i == 10 {
            game_state.update(|state| {
                state.game_over();
                println!("🏁 Game Over! Final Score: {}", state.score);
            });
        }
    }

    score_handle.join().unwrap();
    monitor_handle.join().unwrap();

    let final_state = game_state.get();
    println!("\nFinal state: {:?}", final_state);
}
