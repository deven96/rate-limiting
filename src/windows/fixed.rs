use crate::utils::{RateLimiter, run_simulation};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct FixedWindow {
    capacity: u64,
    window_size: Duration,
    current_count: Mutex<u64>,
    stop: Arc<AtomicBool>,
}

impl FixedWindow {
    pub fn new(capacity: u64, window_size: Duration) -> Arc<Self> {
        let limiter = Arc::new(Self {
            capacity,
            window_size,
            current_count: Mutex::new(0),
            stop: Arc::new(AtomicBool::new(false)),
        });
        Self::start_reset_thread(Arc::clone(&limiter));
        limiter
    }

    fn start_reset_thread(limiter: Arc<Self>) {
        thread::spawn(move || {
            while !limiter.stop.load(Ordering::SeqCst) {
                thread::sleep(limiter.window_size);
                *limiter.current_count.lock().unwrap() = 0;
            }
            println!("Fixed window reset thread stopped.");
        });
    }
}

impl RateLimiter for FixedWindow {
    fn is_rate_limited(&self, amount: u64) -> bool {
        let mut count = self.current_count.lock().unwrap();
        if *count + amount <= self.capacity {
            *count += amount;
            true
        } else {
            false
        }
    }

    fn cleanup(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

pub(crate) fn test_fixed_window(capacity: u64, window_size_secs: u64, requests_per_second: &[u64]) {
    let window_size = Duration::from_secs(window_size_secs);
    let limiter = FixedWindow::new(capacity, window_size);
    let graph_title = format!("fixed_window_{}_{}.png", capacity, window_size_secs);
    let max_y = requests_per_second
        .iter()
        .max()
        .unwrap_or(&capacity)
        .max(&capacity);
    run_simulation(limiter.as_ref(), requests_per_second, *max_y, &graph_title);
}
