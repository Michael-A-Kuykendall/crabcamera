//! Invariant PPT Testing Framework
//!
//! This module provides runtime invariant checking and contract test support
//! for Predictive Property-Based Testing (PPT).
//!
//! # Philosophy
//!
//! "Invariant Superhighways": State flows through the system and is checked at
//! deterministic toll booths. These checks form a verifiable contract that the
//! system architecture enforces at runtime (debug) and verifies in tests.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::thread_local;

/// Categories of invariants for granular analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantType {
    /// Correctness of data structures and algorithms
    Correctness,
    /// Memory safety, boundary checks, valid pointers
    Safety,
    /// Performance envelopes (latency, throughput)
    Performance,
    /// API Contracts and State consistency
    State,
}

impl Default for InvariantType {
    fn default() -> Self {
        Self::Correctness
    }
}

/// A record of a checked invariant
#[derive(Debug, Clone)]
pub struct InvariantRecord {
    /// Description of the invariant check.
    pub message: String,
    /// Category of the invariant.
    pub invariant_type: InvariantType,
    /// Context where the check occurred (e.g. module name).
    pub context: String,
    /// Whether the check passed.
    pub passed: bool,
}

thread_local! {
    // Ring buffer history of recent invariant checks (for crash dumps)
    static INVARIANT_HISTORY: RefCell<VecDeque<InvariantRecord>> = RefCell::new(VecDeque::with_capacity(100));
}

/// Assert an invariant and log it for contract testing.
///
/// # Arguments
/// * `condition` - The invariant condition (must be true)
/// * `message` - Description of the invariant
/// * `context` - Optional context (module/function name)
///
/// # Panics
/// Panics if the condition is false.
#[macro_export]
macro_rules! assert_invariant {
    ($condition:expr, $message:expr) => {
        $crate::invariant_ppt::__assert_invariant_impl($condition, $message, None, $crate::invariant_ppt::InvariantType::Correctness)
    };
    ($condition:expr, $message:expr, $context:expr) => {
        $crate::invariant_ppt::__assert_invariant_impl($condition, $message, Some($context), $crate::invariant_ppt::InvariantType::Correctness)
    };
}

/// Internal implementation - do not call directly
#[doc(hidden)]
pub fn __assert_invariant_impl(
    condition: bool,
    message: &str,
    context: Option<&str>,
    type_: InvariantType
) {
    let ctx = context.unwrap_or("unknown");
    
    let record = InvariantRecord {
        message: message.to_string(),
        invariant_type: type_,
        context: ctx.to_string(),
        passed: condition,
    };

    // Log to history
    INVARIANT_HISTORY.with(|history| {
        let mut h = history.borrow_mut();
        if h.len() >= 100 {
            h.pop_front();
        }
        h.push_back(record);
    });

    // In the future: Dump history here
    assert!(condition, "INVARIANT VIOLATION [{}]: {}", ctx, message);
}

/// Check that specific invariants were verified during test execution.
///
/// # Arguments
/// * `test_name` - Name of the contract test
/// * `required_invariants` - List of invariant messages that must have been checked
///
/// # Panics
/// Panics if any required invariant was not checked.
pub fn contract_test(test_name: &str, required_invariants: &[&str]) {
    let history = INVARIANT_HISTORY.with(|h| h.borrow().clone());

    let mut missing: Vec<&str> = Vec::new();
    for invariant in required_invariants {
        let found = history.iter().any(|r| r.message == *invariant);
        if !found {
            missing.push(invariant);
        }
    }

    assert!(
        missing.is_empty(),
        "CONTRACT FAILURE [{}]: The following invariants were not checked:\n  - {}",
        test_name,
        missing.join("\n  - ")
    );
}

/// Clear the invariant log (call between test runs if needed)
pub fn clear_invariant_log() {
    INVARIANT_HISTORY.with(|h| {
        h.borrow_mut().clear();
    });
}

// ==============================================================================================
//  Performance Invariants
// ==============================================================================================

/// Performance metrics snapshot for invariant analysis
#[derive(Debug, Clone, PartialEq)]
pub struct PerfSnapshot {
    /// Label identifying the performance checkpoint.
    pub label: String,
    /// Latency in milliseconds.
    pub latency_ms: f64,
    /// Throughput in operations per second.
    pub throughput_ops: f64,
    /// Memory usage change in kilobytes.
    pub memory_delta_kb: i64,
}

/// Assert that performance meets a baseline predictive model
///
/// This follows the methodology of "Predictive Property Testing":
/// We assert that the system behaves within a predicted envelope.
pub fn assert_performance_invariant(
    snapshot: &PerfSnapshot,
    baseline_latency: f64,
    tolerance_factor: f64
) {
    let max_latency = baseline_latency * (1.0 + tolerance_factor);
    
    // Log the check so contract tests know we validated performance
    __assert_invariant_impl(
        snapshot.latency_ms <= max_latency, 
        &format!("PERF: {} latency within predicted envelope", snapshot.label),
        Some("performance"),
        InvariantType::Performance
    );
}
