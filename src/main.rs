use buckets::{leaky::test_leaky_bucket, token::test_token_bucket};

mod buckets;
mod utils;

fn main() {
    // Low refill rate, spiky requests
    test_token_bucket(10, 5, &[2, 3, 7, 3, 3, 5, 1, 12, 3, 3]);
    // Medium refill rate, mostly balanced requests
    test_token_bucket(100, 50, &[40, 40, 40, 40, 40, 120, 40, 40, 40, 40]);
    // Slow refill rate, aggressive requests
    test_token_bucket(20, 2, &[5, 5, 5, 5, 5, 5, 5, 5, 5, 5]);
    // Equal refill rate, aggressive requests
    test_token_bucket(50, 50, &[60, 60, 60, 60, 60, 60, 60, 60, 60, 60]);

    // Low leak rate, spiky requests
    test_leaky_bucket(10, 5, &[2, 3, 7, 3, 3, 5, 1, 12, 3, 3]);
    // High capacity, mostly balanced requests
    test_leaky_bucket(100, 50, &[40, 40, 40, 40, 40, 120, 40, 40, 40, 40]);
    // Slow leak rate, aggressive requests
    test_leaky_bucket(20, 2, &[5, 5, 5, 5, 5, 5, 5, 5, 5, 5]);
    // Equal leak rate, aggressive requests
    test_leaky_bucket(50, 50, &[60, 60, 60, 60, 60, 60, 60, 60, 60, 60]);
}
