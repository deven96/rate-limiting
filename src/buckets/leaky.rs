use crate::utils::{RateLimiter, run_simulation};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

struct LeakyBucket {
    capacity: u64,
    leak_rate: u64,
    stop: Arc<AtomicBool>,
    used: AtomicU64,
}

impl LeakyBucket {
    pub fn new(capacity: u64, leak_rate: u64) -> Arc<Self> {
        let bucket = Arc::new(Self {
            used: AtomicU64::new(capacity),
            capacity,
            leak_rate,
            stop: Arc::new(AtomicBool::new(false)),
        });
        Self::start_leak_thread(Arc::clone(&bucket));
        bucket
    }

    fn start_leak_thread(bucket: Arc<Self>) {
        thread::spawn(move || {
            while !bucket.stop.load(Ordering::SeqCst) {
                bucket
                    .used
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |u| {
                        Some(u.saturating_sub(bucket.leak_rate))
                    })
                    .ok();
                thread::sleep(Duration::from_secs(1));
            }
            println!("Dropping leak thread");
        });
    }
}

impl RateLimiter for LeakyBucket {
    fn is_rate_limited(&self, amount: u64) -> bool {
        let current = self.used.load(Ordering::SeqCst);
        if current + amount <= self.capacity {
            self.used.fetch_add(amount, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    fn cleanup(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

pub(crate) fn test_leaky_bucket(capacity: u64, leak_rate: u64, requests_per_second: &[u64]) {
    let bucket = LeakyBucket::new(capacity, leak_rate);
    let graph_title = format!("leaky_bucket_{capacity}_leak_{leak_rate}.png");
    let max_y = requests_per_second
        .iter()
        .max()
        .unwrap_or(&capacity)
        .max(&capacity);
    run_simulation(bucket.as_ref(), requests_per_second, *max_y, &graph_title);
}
