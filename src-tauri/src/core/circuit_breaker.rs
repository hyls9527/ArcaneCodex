use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::time::Instant;

const CLOSED: u8 = 0;
const OPEN: u8 = 1;
const HALF_OPEN: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU64,
    last_failure_ms: AtomicU64,
    half_open_permits: AtomicU64,
    half_open_success_threshold: u64,
    half_open_success_count: AtomicU64,
    failure_threshold: u64,
    reset_timeout_ms: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, reset_timeout_ms: u64) -> Self {
        Self {
            state: AtomicU8::new(CLOSED),
            failure_count: AtomicU64::new(0),
            last_failure_ms: AtomicU64::new(0),
            half_open_permits: AtomicU64::new(1),
            half_open_success_threshold: 2,
            half_open_success_count: AtomicU64::new(0),
            failure_threshold,
            reset_timeout_ms,
        }
    }

    pub fn allow_request(&self) -> bool {
        match self.current_state() {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let last = self.last_failure_ms.load(Ordering::SeqCst);
                let now = Instant::now().elapsed().as_millis() as u64;
                if now.saturating_sub(last) >= self.reset_timeout_ms {
                    let prev = self.half_open_permits.fetch_sub(1, Ordering::SeqCst);
                    if prev > 0 {
                        self.state.store(HALF_OPEN, Ordering::SeqCst);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                let prev = self.half_open_permits.fetch_sub(1, Ordering::SeqCst);
                prev > 0
            }
        }
    }

    pub fn record_success(&self) {
        match self.current_state() {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let count = self.half_open_success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.half_open_success_threshold {
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.half_open_success_count.store(0, Ordering::SeqCst);
                    self.half_open_permits.store(1, Ordering::SeqCst);
                    self.state.store(CLOSED, Ordering::SeqCst);
                }
            }
            CircuitState::Open => {
                self.state.store(HALF_OPEN, Ordering::SeqCst);
            }
        }
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.last_failure_ms.store(
            Instant::now().elapsed().as_millis() as u64,
            Ordering::SeqCst,
        );
        if count >= self.failure_threshold {
            self.half_open_permits.store(1, Ordering::SeqCst);
            self.state.store(OPEN, Ordering::SeqCst);
        }
    }

    pub fn current_state(&self) -> CircuitState {
        match self.state.load(Ordering::SeqCst) {
            CLOSED => CircuitState::Closed,
            OPEN => CircuitState::Open,
            _ => CircuitState::HalfOpen,
        }
    }

    #[allow(dead_code)]
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_closed() {
        let cb = CircuitBreaker::new(5, 30000);
        assert_eq!(cb.current_state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_transitions_to_open_after_threshold() {
        let cb = CircuitBreaker::new(3, 30000);
        cb.record_failure();
        assert_eq!(cb.current_state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.current_state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.current_state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_success_resets_to_closed() {
        let cb = CircuitBreaker::new(2, 30000);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.current_state(), CircuitState::Open);
        cb.record_success();
        assert_eq!(cb.current_state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
    }

    #[test]
    fn test_half_open_allows_request() {
        let cb = CircuitBreaker::new(1, 0);
        cb.record_failure();
        assert_eq!(cb.current_state(), CircuitState::Open);
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(cb.allow_request());
        assert_eq!(cb.current_state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_success_closes() {
        let cb = CircuitBreaker::new(1, 0);
        cb.record_failure();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let _ = cb.allow_request();
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.current_state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_single_success_not_enough() {
        let cb = CircuitBreaker::new(1, 0);
        cb.record_failure();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let _ = cb.allow_request();
        cb.record_success();
        assert_eq!(cb.current_state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_failure_reopens() {
        let cb = CircuitBreaker::new(1, 0);
        cb.record_failure();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let _ = cb.allow_request();
        cb.record_failure();
        assert_eq!(cb.current_state(), CircuitState::Open);
    }

    #[test]
    fn test_half_open_only_one_request_allowed() {
        let cb = CircuitBreaker::new(1, 0);
        cb.record_failure();
        std::thread::sleep(std::time::Duration::from_millis(1));

        let allowed1 = cb.allow_request();
        let allowed2 = cb.allow_request();

        assert!(allowed1);
        assert!(!allowed2);
        assert_eq!(cb.current_state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_concurrent_half_open_race_condition() {
        use std::sync::Arc;
        use std::thread;

        let cb = Arc::new(CircuitBreaker::new(1, 0));
        cb.record_failure();
        std::thread::sleep(std::time::Duration::from_millis(1));

        let mut handles = vec![];
        for _ in 0..100 {
            let cb_clone = Arc::clone(&cb);
            handles.push(thread::spawn(move || {
                if cb_clone.allow_request() {
                    1u64
                } else {
                    0u64
                }
            }));
        }

        let total_allowed: u64 = handles.into_iter().map(|h| h.join().unwrap()).sum();

        assert_eq!(
            total_allowed, 1,
            "Only one request should be allowed through HalfOpen state, got {}",
            total_allowed
        );
        assert_eq!(cb.current_state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_success_resets_permits() {
        let cb = CircuitBreaker::new(1, 0);
        cb.record_failure();
        std::thread::sleep(std::time::Duration::from_millis(1));

        let _ = cb.allow_request();
        cb.record_success();
        cb.record_success();

        assert_eq!(cb.current_state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_half_open_failure_resets_permits() {
        let cb = CircuitBreaker::new(1, 0);
        cb.record_failure();
        std::thread::sleep(std::time::Duration::from_millis(1));

        let _ = cb.allow_request();
        cb.record_failure();

        assert_eq!(cb.current_state(), CircuitState::Open);

        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(cb.allow_request());
        assert_eq!(cb.current_state(), CircuitState::HalfOpen);
    }
}
