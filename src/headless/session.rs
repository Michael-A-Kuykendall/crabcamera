use crate::headless::controls::{validate_control_value, ControlId, ControlValue};
use crate::headless::errors::HeadlessError;
use crate::headless::types::{AudioMode, BufferPolicy, CaptureConfig, Frame, AudioPacket};
use crate::platform::PlatformCamera;
use crate::timing::PTSClock;
use crate::types::{CameraControls, CameraFrame, CameraInitParams};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
#[cfg(feature = "audio")]
use crate::audio::{AudioCapture, AudioFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionState {
    Open,
    Started,
    Stopped,
    Closed,
}

struct Queue<T> {
    inner: Mutex<QueueInner<T>>,
    cv: Condvar,
}

struct QueueInner<T> {
    items: VecDeque<T>,
    capacity: usize,
    dropped: u64,
    closed: bool,
}

impl<T> Queue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(QueueInner {
                items: VecDeque::with_capacity(capacity.min(1024)),
                capacity: capacity.max(1),
                dropped: 0,
                closed: false,
            }),
            cv: Condvar::new(),
        }
    }

    fn push_drop_oldest(&self, item: T) {
        let mut g = self.inner.lock().expect("lock poisoned");
        if g.closed {
            return;
        }

        if g.items.len() >= g.capacity {
            g.items.pop_front();
            g.dropped = g.dropped.saturating_add(1);
        }
        g.items.push_back(item);
        self.cv.notify_one();
    }

    fn pop_timeout(&self, timeout: Duration) -> Result<Option<T>, HeadlessError> {
        let mut g = self.inner.lock().expect("lock poisoned");

        if timeout == Duration::ZERO {
            return Ok(g.items.pop_front());
        }

        let deadline = Instant::now() + timeout;
        loop {
            if let Some(item) = g.items.pop_front() {
                return Ok(Some(item));
            }
            if g.closed {
                return Err(HeadlessError::closed());
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }

            let remaining = deadline - now;
            let (ng, _) = self.cv.wait_timeout(g, remaining).expect("lock poisoned");
            g = ng;
        }
    }

    fn dropped(&self) -> u64 {
        self.inner.lock().expect("lock poisoned").dropped
    }

    fn close(&self) {
        let mut g = self.inner.lock().expect("lock poisoned");
        g.closed = true;
        self.cv.notify_all();
    }
}

struct Inner {
    state: Mutex<SessionState>,
    camera: Mutex<Option<PlatformCamera>>,
    config: CaptureConfig,
    queue: Queue<Frame>,
    start_instant: Instant,
    next_sequence: Mutex<u64>,
    capture_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    stop_flag: Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "audio")]
    pts_clock: PTSClock,
    #[cfg(feature = "audio")]
    audio_enabled: bool,
    #[cfg(feature = "audio")]
    audio_queue: Option<Queue<AudioPacket>>,
    #[cfg(feature = "audio")]
    audio_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    #[cfg(feature = "audio")]
    audio_sequence: Mutex<u64>,
}

#[derive(Clone)]
pub struct SessionHandle {
    inner: Arc<Inner>,
}

pub struct HeadlessSession;

impl HeadlessSession {
    pub fn open(config: CaptureConfig) -> Result<SessionHandle, HeadlessError> {
        // Audio is compile-time gated; enforce config coherence.
        match config.audio_mode {
            AudioMode::Enabled => {
                #[cfg(not(feature = "audio"))]
                {
                    return Err(HeadlessError::unsupported(
                        "audio requested but crate built without audio feature",
                    ));
                }
            }
            AudioMode::Disabled => {}
        }

        let params = CameraInitParams {
            device_id: config.device_id.clone(),
            format: config.format.clone(),
            controls: CameraControls::default(),
        };

        let camera = PlatformCamera::new(params).map_err(HeadlessError::backend)?;

        let capacity = match config.buffer_policy {
            BufferPolicy::DropOldest { capacity } => capacity,
        };

        #[cfg(feature = "audio")]
        let (pts_clock, audio_enabled, audio_queue) = if matches!(config.audio_mode, AudioMode::Enabled) {
            let pts_clock = PTSClock::new();
            let audio_queue = Some(Queue::new(10)); // Small buffer for audio
            (pts_clock, true, audio_queue)
        } else {
            (PTSClock::new(), false, None::<Queue<AudioPacket>>)
        };

        #[cfg(not(feature = "audio"))]
        let (pts_clock, audio_enabled, audio_queue) = (PTSClock::new(), false, None::<Queue<AudioPacket>>);

        Ok(SessionHandle {
            inner: Arc::new(Inner {
                state: Mutex::new(SessionState::Open),
                camera: Mutex::new(Some(camera)),
                config,
                queue: Queue::new(capacity),
                start_instant: Instant::now(),
                next_sequence: Mutex::new(1),
                capture_thread: Mutex::new(None),
                stop_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                #[cfg(feature = "audio")]
                pts_clock,
                #[cfg(feature = "audio")]
                audio_enabled,
                #[cfg(feature = "audio")]
                audio_queue,
                #[cfg(feature = "audio")]
                audio_thread: Mutex::new(None),
                #[cfg(feature = "audio")]
                audio_sequence: Mutex::new(1),
            }),
        })
    }
}

impl SessionHandle {
    pub fn start(&self) -> Result<(), HeadlessError> {
        let mut state = self.inner.state.lock().expect("lock poisoned");
        match *state {
            SessionState::Closed => return Err(HeadlessError::already_closed()),
            SessionState::Started => return Err(HeadlessError::already_started()),
            SessionState::Stopped | SessionState::Open => {}
        }

        self.inner
            .stop_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let inner = self.inner.clone();
        let handle = std::thread::Builder::new()
            .name("crabcamera-headless-capture".to_string())
            .spawn(move || capture_loop(inner))
            .map_err(|e| HeadlessError::invalid_argument(format!("spawn failed: {e}")))?;

        *self.inner.capture_thread.lock().expect("lock poisoned") = Some(handle);
        *state = SessionState::Started;

        // Camera warmup: wait for first frame to ensure camera is ready
        let warmup_start = Instant::now();
        while warmup_start.elapsed() < Duration::from_secs(5) {
            if let Ok(Some(_)) = self.inner.queue.pop_timeout(Duration::from_millis(100)) {
                // Discard warmup frame
                break;
            }
        }

        #[cfg(feature = "audio")]
        if self.inner.audio_enabled {
            let inner = self.inner.clone();
            let audio_handle = std::thread::Builder::new()
                .name("crabcamera-headless-audio".to_string())
                .spawn(move || audio_capture_loop(inner))
                .map_err(|e| HeadlessError::invalid_argument(format!("audio spawn failed: {e}")))?;
            *self.inner.audio_thread.lock().expect("lock poisoned") = Some(audio_handle);
        }

        Ok(())
    }

    pub fn stop(&self, join_timeout: Duration) -> Result<(), HeadlessError> {
        let state = self.inner.state.lock().expect("lock poisoned");
        match *state {
            SessionState::Closed => return Err(HeadlessError::already_closed()),
            SessionState::Stopped => return Err(HeadlessError::already_stopped()),
            SessionState::Started => {}
            SessionState::Open => return Err(HeadlessError::already_stopped()), // Open is like stopped
        }

        self.inner
            .stop_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);

        let join_handle = self.inner.capture_thread.lock().expect("lock poisoned").take();
        drop(state);

        if let Some(handle) = join_handle {
            let start = Instant::now();
            let mut handle = Some(handle);
            loop {
                let finished = handle.as_ref().is_some_and(|h| h.is_finished());
                if finished {
                    let _ = handle.take().unwrap().join();
                    break;
                }
                if start.elapsed() >= join_timeout {
                    // Best-effort: do not hang forever. Keep handle so a later stop/close can retry.
                    *self.inner.capture_thread.lock().expect("lock poisoned") = handle.take();
                    return Err(HeadlessError::timeout());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }

        #[cfg(feature = "audio")]
        {
            let audio_join_handle = self.inner.audio_thread.lock().expect("lock poisoned").take();
            if let Some(handle) = audio_join_handle {
                let start = Instant::now();
                let mut handle = Some(handle);
                loop {
                    let finished = handle.as_ref().is_some_and(|h| h.is_finished());
                    if finished {
                        let _ = handle.take().unwrap().join();
                        break;
                    }
                    if start.elapsed() >= join_timeout {
                        *self.inner.audio_thread.lock().expect("lock poisoned") = handle.take();
                        return Err(HeadlessError::timeout());
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }

        let mut state = self.inner.state.lock().expect("lock poisoned");
        if *state != SessionState::Closed {
            *state = SessionState::Stopped;
        }
        Ok(())
    }

    pub fn close(&self, join_timeout: Duration) -> Result<(), HeadlessError> {
        {
            let state = *self.inner.state.lock().expect("lock poisoned");
            if state == SessionState::Closed {
                return Err(HeadlessError::already_closed());
            }
        }

        if let Err(e) = self.stop(join_timeout) {
            log::warn!("Error stopping session during close: {}", e);
        }

        self.inner.queue.close();
        *self.inner.camera.lock().expect("lock poisoned") = None;
        #[cfg(feature = "audio")]
        {
            if let Some(audio_queue) = &self.inner.audio_queue {
                audio_queue.close();
            }
        }
        *self.inner.state.lock().expect("lock poisoned") = SessionState::Closed;
        Ok(())
    }

    pub fn dropped_frames(&self) -> Result<u64, HeadlessError> {
        self.ensure_not_closed()?;
        Ok(self.inner.queue.dropped())
    }

    pub fn get_frame(&self, timeout: Duration) -> Result<Option<Frame>, HeadlessError> {
        self.ensure_not_closed()?;
        let state = *self.inner.state.lock().expect("lock poisoned");
        match state {
            SessionState::Closed => return Err(HeadlessError::closed()),
            SessionState::Stopped => return Err(HeadlessError::stopped()),
            SessionState::Open => return Err(HeadlessError::invalid_argument("session not started")),
            SessionState::Started => {}
        }
        self.inner.queue.pop_timeout(timeout)
    }

    pub fn get_audio_packet(&self, timeout: Duration) -> Result<Option<AudioPacket>, HeadlessError> {
        #[cfg(not(feature = "audio"))]
        return Err(HeadlessError::unsupported("audio not compiled in"));

        #[cfg(feature = "audio")]
        {
            if !self.inner.audio_enabled {
                return Err(HeadlessError::unsupported("audio not enabled"));
            }
            let state = *self.inner.state.lock().expect("lock poisoned");
            match state {
                SessionState::Closed => return Err(HeadlessError::closed()),
                SessionState::Stopped => return Err(HeadlessError::stopped()),
                SessionState::Open => return Err(HeadlessError::invalid_argument("session not started")),
                SessionState::Started => {}
            }
            if let Some(audio_queue) = &self.inner.audio_queue {
                audio_queue.pop_timeout(timeout)
            } else {
                Err(HeadlessError::unsupported("audio not available"))
            }
        }
    }

    pub fn set_control(
        &self,
        control_id: ControlId,
        value: ControlValue,
    ) -> Result<(), HeadlessError> {
        self.ensure_not_closed()?;
        validate_control_value(control_id, &value)?;

        let mut controls = self.get_controls()?;
        apply_control_to_struct(&mut controls, control_id, value);

        let mut camera_guard = self.inner.camera.lock().expect("lock poisoned");
        let cam_guard = camera_guard.as_mut().ok_or_else(HeadlessError::closed)?;

        cam_guard
            .apply_controls(&controls)
            .map_err(HeadlessError::backend)
            .map(|_| ())
    }

    pub fn get_controls(&self) -> Result<CameraControls, HeadlessError> {
        self.ensure_not_closed()?;
        let camera_guard = self.inner.camera.lock().expect("lock poisoned");
        let cam_guard = camera_guard.as_ref().ok_or_else(HeadlessError::closed)?;
        cam_guard.get_controls().map_err(HeadlessError::backend)
    }

    pub fn list_controls(&self) -> Result<Vec<(crate::headless::controls::ControlInfo, Option<crate::headless::controls::ControlValue>)>, HeadlessError> {
        use crate::headless::controls::{all_controls, ControlId, ControlValue};
        self.ensure_not_closed()?;
        let current = self.get_controls()?;
        let mut result = Vec::new();
        for info in all_controls() {
            let value = match info.id {
                ControlId::AutoFocus => current.auto_focus.map(ControlValue::Bool),
                ControlId::FocusDistance => current.focus_distance.map(ControlValue::F32),
                ControlId::AutoExposure => current.auto_exposure.map(ControlValue::Bool),
                ControlId::ExposureTime => current.exposure_time.map(ControlValue::F32),
                ControlId::IsoSensitivity => current.iso_sensitivity.map(ControlValue::U32),
                ControlId::WhiteBalance => current.white_balance.clone().map(ControlValue::WhiteBalance),
                ControlId::Aperture => current.aperture.map(ControlValue::F32),
                ControlId::Zoom => current.zoom.map(ControlValue::F32),
                ControlId::Brightness => current.brightness.map(ControlValue::F32),
                ControlId::Contrast => current.contrast.map(ControlValue::F32),
                ControlId::Saturation => current.saturation.map(ControlValue::F32),
                ControlId::Sharpness => current.sharpness.map(ControlValue::F32),
                ControlId::NoiseReduction => current.noise_reduction.map(ControlValue::Bool),
                ControlId::ImageStabilization => current.image_stabilization.map(ControlValue::Bool),
            };
            result.push((info, value));
        }
        Ok(result)
    }

    pub fn get_control(&self, control_id: crate::headless::controls::ControlId) -> Result<Option<crate::headless::controls::ControlValue>, HeadlessError> {
        use crate::headless::controls::{ControlId, ControlValue};
        self.ensure_not_closed()?;
        let current = self.get_controls()?;
        let value = match control_id {
            ControlId::AutoFocus => current.auto_focus.map(ControlValue::Bool),
            ControlId::FocusDistance => current.focus_distance.map(ControlValue::F32),
            ControlId::AutoExposure => current.auto_exposure.map(ControlValue::Bool),
            ControlId::ExposureTime => current.exposure_time.map(ControlValue::F32),
            ControlId::IsoSensitivity => current.iso_sensitivity.map(ControlValue::U32),
            ControlId::WhiteBalance => current.white_balance.map(ControlValue::WhiteBalance),
            ControlId::Aperture => current.aperture.map(ControlValue::F32),
            ControlId::Zoom => current.zoom.map(ControlValue::F32),
            ControlId::Brightness => current.brightness.map(ControlValue::F32),
            ControlId::Contrast => current.contrast.map(ControlValue::F32),
            ControlId::Saturation => current.saturation.map(ControlValue::F32),
            ControlId::Sharpness => current.sharpness.map(ControlValue::F32),
            ControlId::NoiseReduction => current.noise_reduction.map(ControlValue::Bool),
            ControlId::ImageStabilization => current.image_stabilization.map(ControlValue::Bool),
        };
        Ok(value)
    }

    fn ensure_not_closed(&self) -> Result<(), HeadlessError> {
        let state = *self.inner.state.lock().expect("lock poisoned");
        if state == SessionState::Closed {
            return Err(HeadlessError::closed());
        }
        Ok(())
    }
}

impl Drop for SessionHandle {
    fn drop(&mut self) {
        if let Err(e) = self.close(Duration::from_millis(100)) {
            log::warn!("Error closing session in drop: {}", e);
        }
    }
}

fn capture_loop(inner: Arc<Inner>) {
    let mut camera = match inner.camera.lock().expect("lock poisoned").take() {
        Some(cam) => cam,
        None => return,
    };

    let _ = camera.start_stream();

    loop {
        if inner
            .stop_flag
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            break;
        }

        match camera.capture_frame() {
            Ok(frame) => {
                let normalized = normalize_frame(&inner, frame);
                inner.queue.push_drop_oldest(normalized);
            }
            Err(_e) => {
                // Session failure -> close queue so reads error out.
                inner.queue.close();
                break;
            }
        }
    }

    let _ = camera.stop_stream();

    // Return camera back to session for control queries after stop.
    *inner.camera.lock().expect("lock poisoned") = Some(camera);
}

#[cfg(feature = "audio")]
fn audio_capture_loop(inner: Arc<Inner>) {
    let pts_clock = PTSClock::new();
    let mut audio_capture = match AudioCapture::new(inner.config.audio_device_id.clone(), 48000, 2, pts_clock) {
        Ok(cap) => cap,
        Err(_) => return, // Audio failed
    };

    if let Err(_) = audio_capture.start() {
        // Audio failed, but don't stop video
        return;
    }

    loop {
        if inner.stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        match audio_capture.recv_timeout(Duration::from_millis(100)) {
            Ok(frame) => {
                if let Some(audio_queue) = &inner.audio_queue {
                    let normalized = normalize_audio_packet(&inner, frame);
                    audio_queue.push_drop_oldest(normalized);
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // Continue
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                // Stop
                break;
            }
        }
    }

    let _ = audio_capture.stop();
    // Audio capture ends here
}

#[cfg(not(feature = "audio"))]
fn audio_capture_loop(_inner: Arc<Inner>) {
    // No-op
}

fn normalize_frame(inner: &Inner, frame: CameraFrame) -> Frame {
    let sequence = {
        let mut g = inner.next_sequence.lock().expect("lock poisoned");
        let v = *g;
        *g = g.saturating_add(1);
        v
    };

    #[cfg(feature = "audio")]
    let timestamp_us = (inner.pts_clock.pts() * 1_000_000.0) as u64;
    #[cfg(not(feature = "audio"))]
    let timestamp_us = inner.start_instant.elapsed().as_micros() as u64;

    Frame {
        sequence,
        timestamp_us,
        width: frame.width,
        height: frame.height,
        format: frame.format,
        device_id: frame.device_id,
        data: frame.data,
    }
}

#[cfg(feature = "audio")]
fn normalize_audio_packet(inner: &Inner, frame: AudioFrame) -> AudioPacket {
    let sequence = {
        let mut g = inner.audio_sequence.lock().expect("lock poisoned");
        let v = *g;
        *g = g.saturating_add(1);
        v
    };

    let timestamp_us = (frame.timestamp * 1_000_000.0) as u64;

    // Convert f32 samples to bytes
    let data = frame.samples.iter().map(|&s| s.to_le_bytes()).flatten().collect();

    AudioPacket {
        sequence,
        timestamp_us,
        sample_rate: frame.sample_rate,
        channels: frame.channels,
        format: "pcm_f32".to_string(),
        data,
    }
}

fn apply_control_to_struct(controls: &mut CameraControls, id: ControlId, value: ControlValue) {
    match (id, value) {
        (ControlId::AutoFocus, ControlValue::Bool(v)) => controls.auto_focus = Some(v),
        (ControlId::FocusDistance, ControlValue::F32(v)) => controls.focus_distance = Some(v),
        (ControlId::AutoExposure, ControlValue::Bool(v)) => controls.auto_exposure = Some(v),
        (ControlId::ExposureTime, ControlValue::F32(v)) => controls.exposure_time = Some(v),
        (ControlId::IsoSensitivity, ControlValue::U32(v)) => controls.iso_sensitivity = Some(v),
        (ControlId::WhiteBalance, ControlValue::WhiteBalance(v)) => controls.white_balance = Some(v),
        (ControlId::Aperture, ControlValue::F32(v)) => controls.aperture = Some(v),
        (ControlId::Zoom, ControlValue::F32(v)) => controls.zoom = Some(v),
        (ControlId::Brightness, ControlValue::F32(v)) => controls.brightness = Some(v),
        (ControlId::Contrast, ControlValue::F32(v)) => controls.contrast = Some(v),
        (ControlId::Saturation, ControlValue::F32(v)) => controls.saturation = Some(v),
        (ControlId::Sharpness, ControlValue::F32(v)) => controls.sharpness = Some(v),
        (ControlId::NoiseReduction, ControlValue::Bool(v)) => controls.noise_reduction = Some(v),
        (ControlId::ImageStabilization, ControlValue::Bool(v)) => {
            controls.image_stabilization = Some(v)
        }
        _ => {}
    }
}
