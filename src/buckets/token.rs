use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use crate::utils::{RateLimiter, run_simulation};

struct TokenBucket {
    tokens: AtomicU64,
    capacity: u64,
    refill_rate: u64,
    stop: Arc<AtomicBool>,
}

impl TokenBucket {
    pub fn new(capacity: u64, refill_rate: u64) -> Arc<Self> {
        let bucket = Arc::new(Self {
            tokens: AtomicU64::new(capacity),
            capacity,
            refill_rate,
            stop: Arc::new(AtomicBool::new(false)),
        });
        Self::start_refill_thread(Arc::clone(&bucket));
        bucket
    }

    fn start_refill_thread(bucket: Arc<Self>) {
        thread::spawn(move || {
            while !bucket.stop.load(Ordering::SeqCst) {
                let current = bucket.tokens.load(Ordering::SeqCst);
                if current < bucket.capacity {
                    let refill_amount = (bucket.capacity - current).min(bucket.refill_rate);
                    bucket.tokens.fetch_add(refill_amount, Ordering::SeqCst);
                }
                thread::sleep(Duration::from_secs(1));
            }
            println!("Dropping refill thread");
        });
    }
}

impl RateLimiter for TokenBucket {
    fn is_rate_limited(&self, amount: u64) -> bool {
        let current = self.tokens.load(Ordering::SeqCst);
        if current >= amount {
            self.tokens.fetch_sub(amount, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    fn cleanup(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

pub(crate) fn test_token_bucket(capacity: u64, refill_rate: u64, requests_per_second: &[u64]) {
    let bucket = TokenBucket::new(capacity, refill_rate);
    let graph_title = format!("token_bucket_capacity_{capacity}_refill_{refill_rate}.png");
    let max_y = requests_per_second
        .iter()
        .max()
        .unwrap_or(&capacity)
        .max(&capacity);
    run_simulation(bucket.as_ref(), requests_per_second, *max_y, &graph_title);
}
