use crate::quality::{QualityReport, QualityScore, QualityValidator};
use crate::types::CameraFrame;
use crate::constants::{TRIGGER_MIN_QUALITY, TRIGGER_STABILITY_MS, TRIGGER_TIMEOUT_SECS, TRIGGER_CONSECUTIVE_FRAMES, TRIGGER_HISTORY_SIZE};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Configuration for smart capture triggers
#[derive(Debug, Clone)]
pub struct TriggerConfig {
    /// Minimum overall quality score required (0.0 - 1.0)
    pub min_quality_score: f32,
    /// Minimum stability duration (how long quality must be high)
    pub min_stability_duration: Duration,
    /// Maximum time to wait for a good shot before forcing capture
    pub timeout: Option<Duration>,
    /// Number of consecutive frames that must meet criteria
    pub required_consecutive_good_frames: usize,
    /// Once `Ready` is reached, lock the trigger (return `Captured`) until
    /// `reset()`, preventing rapid re-triggering on subsequent good frames.
    pub lock_after_ready: bool,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            min_quality_score: TRIGGER_MIN_QUALITY,
            min_stability_duration: Duration::from_millis(TRIGGER_STABILITY_MS),
            timeout: Some(Duration::from_secs(TRIGGER_TIMEOUT_SECS)),
            required_consecutive_good_frames: TRIGGER_CONSECUTIVE_FRAMES,
            lock_after_ready: true,
        }
    }
}

/// Status of the smart trigger monitor
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerStatus {
    /// Waiting for better conditions
    Thinking(String),
    /// Ready to capture (conditions met)
    Ready,
    /// Timeout reached, capturing best available
    Timeout,
    /// Processing complete
    Captured,
}

/// Intelligent capture coordinator that uses Invariant Superhighway quality metrics
/// to automate the "perfect shot" timing.
pub struct SmartTrigger {
    validator: QualityValidator,
    config: TriggerConfig,
    
    // State tracking
    start_time: Instant,
    good_frame_streak: usize,
    last_good_frame_time: Option<Instant>,
    best_frame_so_far: Option<(CameraFrame, QualityScore)>,
    locked: bool,
    
    // Analysis history for smoothing
    score_history: VecDeque<f32>,
}

impl SmartTrigger {
    /// Create a new smart trigger with the given configuration.
    pub fn new(config: TriggerConfig) -> Self {
        Self {
            validator: QualityValidator::default(),
            config,
            start_time: Instant::now(),
            good_frame_streak: 0,
            last_good_frame_time: None,
            best_frame_so_far: None,
            locked: false,
            score_history: VecDeque::with_capacity(TRIGGER_HISTORY_SIZE),
        }
    }

    /// Set a custom validator for quality scoring.
    pub fn with_validator(mut self, validator: QualityValidator) -> Self {
        self.validator = validator;
        self
    }

    /// Reset trigger state for a new capture sequence
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.good_frame_streak = 0;
        self.last_good_frame_time = None;
        self.best_frame_so_far = None;
        self.locked = false;
        self.score_history.clear();
    }

    /// Process a new frame and determine if capture should trigger
    pub fn process_frame(&mut self, frame: &CameraFrame) -> (TriggerStatus, QualityReport) {
        // Enforce Performance Invariant for analysis speed
        #[cfg(debug_assertions)]
        let _perf_guard = {
            use crate::invariant_ppt::{PerfSnapshot, assert_performance_invariant};
            struct Guard(Instant);
            impl Drop for Guard {
                 fn drop(&mut self) {
                     let elapsed = self.0.elapsed().as_secs_f64() * 1000.0;
                     assert_performance_invariant(
                         &PerfSnapshot {
                             label: "smart_trigger_analysis".into(),
                             latency_ms: elapsed,
                             throughput_ops: 0.0,
                             memory_delta_kb: 0,
                         },
                         500.0, // Generous budget for debug mode analysis
                         1.0   
                     );
                 }
            }
            Guard(Instant::now())
        };

        let report = self.validator.validate_frame(frame);
        let score = report.score.overall;

        // Update history
        if self.score_history.len() >= TRIGGER_HISTORY_SIZE {
            self.score_history.pop_front();
        }
        self.score_history.push_back(score);

        // Track best frame
        if let Some((_, best_score)) = &self.best_frame_so_far {
            if score > best_score.overall {
                self.best_frame_so_far = Some((frame.clone(), report.score.clone()));
            }
        } else {
            self.best_frame_so_far = Some((frame.clone(), report.score.clone()));
        }

        // Once a shot has been triggered, hold locked until reset() to avoid
        // rapid re-triggering on subsequent good frames.
        if self.locked {
            return (TriggerStatus::Captured, report);
        }

        // Check timeout
        if let Some(timeout) = self.config.timeout {
            if self.start_time.elapsed() > timeout {
                return (TriggerStatus::Timeout, report);
            }
        }

        // Check quality threshold
        if score >= self.config.min_quality_score {
            self.good_frame_streak += 1;
            
            if self.last_good_frame_time.is_none() {
                self.last_good_frame_time = Some(Instant::now());
            }

            let stability_duration = self.last_good_frame_time
                .map_or(Duration::ZERO, |t| t.elapsed());

            if self.good_frame_streak >= self.config.required_consecutive_good_frames 
               && stability_duration >= self.config.min_stability_duration 
            {
                if self.config.lock_after_ready {
                    self.locked = true;
                }
                return (TriggerStatus::Ready, report);
            }
        } else {
            self.good_frame_streak = 0;
            self.last_good_frame_time = None;
        }

        // Generate status message
        let status_msg = if score < self.config.min_quality_score {
            "Improving quality...".to_string()
        } else {
            format!("Stabilizing ({}/{})", 
                self.good_frame_streak, 
                self.config.required_consecutive_good_frames)
        };

        (TriggerStatus::Thinking(status_msg), report)
    }

    /// Retrieve the best frame seen so far (useful for timeout scenarios)
    pub fn get_best_frame(&self) -> Option<CameraFrame> {
        self.best_frame_so_far.as_ref().map(|(f, _)| f.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_frame(brightness: u8) -> CameraFrame {
        let width = 640;
        let height = 480;
        let data = vec![brightness; (width * height * 3) as usize];
        CameraFrame::new(data, width, height, "test".into())
    }

    #[test]
    fn test_smart_trigger_flow() {
        crate::invariant_ppt::clear_invariant_log();

        let config = TriggerConfig {
            min_quality_score: 0.5,
            min_stability_duration: Duration::ZERO,
            required_consecutive_good_frames: 2,
            timeout: None,
            lock_after_ready: true,
        };
        
        let mut trigger = SmartTrigger::new(config);

        // Frame 1: Low Quality (Black frame triggers underexposed/noise often)
        let frame_bad = create_test_frame(0);
        let (status, _) = trigger.process_frame(&frame_bad);
        match status {
            TriggerStatus::Thinking(msg) => assert!(msg.contains("Improving")),
            _ => panic!("Should be thinking on bad frame"),
        }

        // Frame 2: Good Quality (Mid gray)
        let frame_good = create_test_frame(128);
        
        // First good frame - starts streak
        let (status, _) = trigger.process_frame(&frame_good);
        match status {
            TriggerStatus::Thinking(msg) => assert!(msg.contains("Stabilizing")),
            _ => panic!("Should be stabilizing on first good frame"),
        }

        // Second good frame - completes streak
        let (status, _) = trigger.process_frame(&frame_good);
        assert_eq!(status, TriggerStatus::Ready);
    }

    #[test]
    fn test_smart_trigger_timeout() {
        crate::invariant_ppt::clear_invariant_log();

        let config = TriggerConfig {
            min_quality_score: 0.99, // Impossible score
            min_stability_duration: Duration::ZERO,
            required_consecutive_good_frames: 1,
            timeout: Some(Duration::from_millis(1)), // Immediate timeout
            lock_after_ready: true,
        };
        
        let mut trigger = SmartTrigger::new(config);
        
        // Wait a small amount to ensure timeout elapses
        std::thread::sleep(Duration::from_millis(10));

        let frame = create_test_frame(128);
        let (status, _) = trigger.process_frame(&frame);
        
        assert_eq!(status, TriggerStatus::Timeout);
        assert!(trigger.get_best_frame().is_some());
    }

    #[test]
    fn test_smart_trigger_lock_prevents_duplicate() {
        crate::invariant_ppt::clear_invariant_log();

        let config = TriggerConfig {
            min_quality_score: 0.5,
            min_stability_duration: Duration::ZERO,
            required_consecutive_good_frames: 1,
            timeout: None,
            lock_after_ready: true,
        };
        let mut trigger = SmartTrigger::new(config);

        let frame_good = create_test_frame(128);

        // First good frame triggers Ready and locks the trigger.
        let (s1, _) = trigger.process_frame(&frame_good);
        assert_eq!(s1, TriggerStatus::Ready);

        // A second good frame must NOT re-trigger; it reports Captured (locked).
        let (s2, _) = trigger.process_frame(&frame_good);
        assert_eq!(s2, TriggerStatus::Captured);

        // After reset, the trigger re-arms and can fire again.
        trigger.reset();
        let (s3, _) = trigger.process_frame(&frame_good);
        assert_eq!(s3, TriggerStatus::Ready);
    }

    #[test]
    fn test_smart_trigger_no_lock_allows_retrigger() {
        crate::invariant_ppt::clear_invariant_log();

        let config = TriggerConfig {
            min_quality_score: 0.5,
            min_stability_duration: Duration::ZERO,
            required_consecutive_good_frames: 1,
            timeout: None,
            lock_after_ready: false,
        };
        let mut trigger = SmartTrigger::new(config);

        let frame_good = create_test_frame(128);
        let (s1, _) = trigger.process_frame(&frame_good);
        assert_eq!(s1, TriggerStatus::Ready);

        // Without locking, a subsequent good frame re-triggers Ready.
        let (s2, _) = trigger.process_frame(&frame_good);
        assert_eq!(s2, TriggerStatus::Ready);
    }
}
