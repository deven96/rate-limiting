use crate::utils::{RateLimiter, run_simulation};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct SlidingWindow {
    capacity: u64,
    window_size: Duration,
    timestamps: Mutex<VecDeque<Instant>>,
    stop: Arc<AtomicBool>,
}

impl SlidingWindow {
    pub fn new(capacity: u64, window_size: Duration) -> Arc<Self> {
        let limiter = Arc::new(Self {
            capacity,
            window_size,
            timestamps: Mutex::new(VecDeque::new()),
            stop: Arc::new(AtomicBool::new(false)),
        });
        Self::start_cleanup_thread(Arc::clone(&limiter));
        limiter
    }

    fn start_cleanup_thread(limiter: Arc<Self>) {
        let window_size = limiter.window_size;
        thread::spawn(move || {
            while !limiter.stop.load(Ordering::SeqCst) {
                // auto cleanup every half window
                thread::sleep(window_size / 2);
                let mut timestamps = limiter.timestamps.lock().unwrap();
                let cutoff = Instant::now() - window_size;
                while let Some(&time) = timestamps.front() {
                    if time < cutoff {
                        timestamps.pop_front();
                    } else {
                        break;
                    }
                }
            }
            println!("Sliding window cleanup thread stopped.");
        });
    }
}

impl RateLimiter for SlidingWindow {
    fn is_rate_limited(&self, _amount: u64) -> bool {
        let mut timestamps = self.timestamps.lock().unwrap();
        let now = Instant::now();
        let cutoff = now - self.window_size;

        // Remove expired timestamps
        while let Some(&time) = timestamps.front() {
            if time < cutoff {
                timestamps.pop_front();
            } else {
                break;
            }
        }

        // Check if we can allow the request
        if timestamps.len() < self.capacity as usize {
            timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    fn cleanup(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

pub(crate) fn test_sliding_window(
    capacity: u64,
    window_size_secs: u64,
    requests_per_second: &[u64],
) {
    let window_size = Duration::from_secs(window_size_secs);
    let limiter = SlidingWindow::new(capacity, window_size);
    let graph_title = format!("sliding_window_{}_{}.png", capacity, window_size_secs);
    let max_y = requests_per_second
        .iter()
        .max()
        .unwrap_or(&capacity)
        .max(&capacity);
    run_simulation(limiter.as_ref(), requests_per_second, *max_y, &graph_title);
}
