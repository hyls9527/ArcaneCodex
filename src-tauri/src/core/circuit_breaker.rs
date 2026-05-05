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
    failure_threshold: u64,
    reset_timeout_ms: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, reset_timeout_ms: u64) -> Self {
        Self {
            state: AtomicU8::new(CLOSED),
            failure_count: AtomicU64::new(0),
            last_failure_ms: AtomicU64::new(0),
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
                    self.state.store(HALF_OPEN, Ordering::SeqCst);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.state.store(CLOSED, Ordering::SeqCst);
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.last_failure_ms.store(
            Instant::now().elapsed().as_millis() as u64,
            Ordering::SeqCst,
        );
        if count >= self.failure_threshold {
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
        assert_eq!(cb.current_state(), CircuitState::Closed);
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
}
