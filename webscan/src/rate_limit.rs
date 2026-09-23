use std::thread::sleep;
use std::time::Duration;

/// Applies the configured throttle delay before an outbound HTTP request.
pub fn throttle(delay_ms: u64) {
    if delay_ms > 0 {
        sleep(Duration::from_millis(delay_ms));
    }
}
