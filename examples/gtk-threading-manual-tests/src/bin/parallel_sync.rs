//! Several threads open sync dialogs at the same time, all should be shown at once
use threading_tests::*;

fn main() {
    init();
    let handles: Vec<_> = (0..3)
        .map(|i| spawn(&format!("worker-{i}"), move || message(&format!("worker {i}"))))
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
