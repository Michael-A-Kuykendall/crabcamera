#[cfg(feature = "audio")]
use crate::audio::{AudioCapture, AudioFrame};
use crate::headless::controls::{validate_control_value, ControlId, ControlValue};
use crate::headless::errors::HeadlessError;
use crate::headless::types::{AudioMode, AudioPacket, BufferPolicy, CaptureConfig, Frame};
use crate::platform::PlatformCamera;
use crate::timing::PTSClock;
use crate::types::{CameraControls, CameraFrame, CameraInitParams};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

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
    #[allow(dead_code)] // Used conditionally based on audio feature
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

/// A handle to an active headless capture session.
///
/// This struct allows controlling the session (start, stop, close), retrieving frames,
/// and adjusting camera settings. It is thread-safe and can be cloned to share access
/// to the underlying session state.
#[derive(Clone)]
pub struct SessionHandle {
    inner: Arc<Inner>,
}

/// A factory for creating headless capture sessions.
///
/// This struct provides the entry point for initializing a camera device and
/// preparing it for capture.
pub struct HeadlessSession;

impl HeadlessSession {
    /// Opens a camera device and prepares a capture session.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the capture session, including device ID, format, and buffer policy.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `SessionHandle` on success, or a `HeadlessError` if initialization fails.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::BackendError`: If the camera backend fails to initialize the device.
    /// * `HeadlessError::Unsupported`: If the requested configuration (e.g., audio) is not supported by the build.
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
        let (pts_clock, audio_enabled, audio_queue) =
            if matches!(config.audio_mode, AudioMode::Enabled) {
                let pts_clock = PTSClock::new();
                let audio_queue = Some(Queue::new(10)); // Small buffer for audio
                (pts_clock, true, audio_queue)
            } else {
                (PTSClock::new(), false, None::<Queue<AudioPacket>>)
            };

        #[cfg(not(feature = "audio"))]
        let (pts_clock, audio_enabled, audio_queue) =
            (PTSClock::new(), false, None::<Queue<AudioPacket>>);

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
    /// Starts the capture loop in a background thread.
    ///
    /// This method spawns a thread to continuously capture frames from the camera
    /// and pushes them to the internal queue based on the buffer policy.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::AlreadyStarted`: If the session is already running.
    /// * `HeadlessError::AlreadyClosed`: If the session has been permanently closed.
    /// * `HeadlessError::InvalidArgument`: If thread spawning fails.
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

    /// Stops the capture session, joining the threads.
    ///
    /// The `close` method also stops the session, but this method allows for a controlled
    /// shutdown without cleaning up all resources, potentially allowing a restart (though currently only `start` is available).
    ///
    /// # Arguments
    ///
    /// * `join_timeout` - The maximum time to wait for the capture thread to terminate.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::AlreadyClosed`: If the session has already been closed.
    /// * `HeadlessError::AlreadyStopped`: If the session is already stopped.
    /// * `HeadlessError::ThreadJoin`: If the capture thread panics or times out.
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

        let join_handle = self
            .inner
            .capture_thread
            .lock()
            .expect("lock poisoned")
            .take();
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
            let audio_join_handle = self
                .inner
                .audio_thread
                .lock()
                .expect("lock poisoned")
                .take();
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

    /// Permanently closes the session and releases associated resources.
    ///
    /// This method stops the capture thread, closes the queues, and releases the
    /// camera device. The session cannot be reused after calling `close`.
    ///
    /// # Arguments
    ///
    /// * `join_timeout` - The timeout for stopping the background threads.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::AlreadyClosed`: If the session has already been closed.
    /// * `HeadlessError`: Propagates any error encountered during `stop()`.
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

    /// Returns the number of frames dropped due to buffer overflow or decoding issues.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::Closed`: If called on a closed session.
    pub fn dropped_frames(&self) -> Result<u64, HeadlessError> {
        self.ensure_not_closed()?;
        Ok(self.inner.queue.dropped())
    }

    /// Retrieves the next available frame from the capture queue, waiting up to `timeout`.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The maximum duration to wait for a frame.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Frame))`: If a frame is retrieved successfully.
    /// * `Ok(None)`: If the timeout is reached before a frame is available.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::Closed`: If called on a closed session.
    /// * `HeadlessError::Stopped`: If called on a stopped session.
    /// * `HeadlessError::InvalidArgument`: If called on a session that has not been started.
    pub fn get_frame(&self, timeout: Duration) -> Result<Option<Frame>, HeadlessError> {
        self.ensure_not_closed()?;
        let state = *self.inner.state.lock().expect("lock poisoned");
        match state {
            SessionState::Closed => return Err(HeadlessError::closed()),
            SessionState::Stopped => return Err(HeadlessError::stopped()),
            SessionState::Open => {
                return Err(HeadlessError::invalid_argument("session not started"))
            }
            SessionState::Started => {}
        }
        self.inner.queue.pop_timeout(timeout)
    }

    /// Retrieves the next available audio packet from the queue, waiting up to `timeout`.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The maximum duration to wait for an audio packet.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(AudioPacket))`: If a packet is retrieved.
    /// * `Ok(None)`: If no packet is available within the timeout.
    /// * `Err(HeadlessError::Unsupported)`: if audio support is compiled out or not enabled.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::Closed`: If called on a closed session.
    /// * `HeadlessError::Stopped`: If called on a stopped session.
    /// * `HeadlessError::InvalidArgument`: If called on a session that has not been started.
    pub fn get_audio_packet(
        &self,
        timeout: Duration,
    ) -> Result<Option<AudioPacket>, HeadlessError> {
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
                SessionState::Open => {
                    return Err(HeadlessError::invalid_argument("session not started"))
                }
                SessionState::Started => {}
            }
            if let Some(audio_queue) = &self.inner.audio_queue {
                audio_queue.pop_timeout(timeout)
            } else {
                Err(HeadlessError::unsupported("audio not available"))
            }
        }
    }

    /// Sets a camera control value directly.
    ///
    /// The control value is validated against the platform capabilities if possible,
    /// and then applied to the camera backend.
    ///
    /// # Arguments
    ///
    /// * `control_id` - The identifier of the control to change.
    /// * `value` - The new value for the control.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::BackendError`: If the camera backend rejects the setting.
    /// * `HeadlessError::InvalidControl`: If the value is out of range or incorrect type.
    /// * `HeadlessError::Closed`: If the session is closed.
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
            .map(|_result| ())
    }

    /// Retrieves the current values of all supported camera controls.
    ///
    /// This queries the backend for the current state of settings like exposure,
    /// focus, and white balance.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::BackendError`: If the query fails.
    /// * `HeadlessError::Closed`: If the session is closed.
    pub fn get_controls(&self) -> Result<CameraControls, HeadlessError> {
        self.ensure_not_closed()?;
        let camera_guard = self.inner.camera.lock().expect("lock poisoned");
        let cam_guard = camera_guard.as_ref().ok_or_else(HeadlessError::closed)?;
        cam_guard.get_controls().map_err(HeadlessError::backend)
    }

    /// Lists all known controls with their current values.
    ///
    /// This is a convenience method that returns a `Vec` of `(ControlInfo, Option<ControlValue>)`
    /// for introspection. It retrieves all current settings via `get_controls()` and maps
    /// them to their definition.
    ///
    /// # Return
    ///
    /// A vector of tuples where the first element describes the control and the second
    /// contains its current value, if available.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::BackendError`: If `get_controls` fails.
    /// * `HeadlessError::Closed`: If the session in closed.
    pub fn list_controls(
        &self,
    ) -> Result<
        Vec<(
            crate::headless::controls::ControlInfo,
            Option<crate::headless::controls::ControlValue>,
        )>,
        HeadlessError,
    > {
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
                ControlId::WhiteBalance => current
                    .white_balance
                    .clone()
                    .map(ControlValue::WhiteBalance),
                ControlId::Aperture => current.aperture.map(ControlValue::F32),
                ControlId::Zoom => current.zoom.map(ControlValue::F32),
                ControlId::Brightness => current.brightness.map(ControlValue::F32),
                ControlId::Contrast => current.contrast.map(ControlValue::F32),
                ControlId::Saturation => current.saturation.map(ControlValue::F32),
                ControlId::Sharpness => current.sharpness.map(ControlValue::F32),
                ControlId::NoiseReduction => current.noise_reduction.map(ControlValue::Bool),
                ControlId::ImageStabilization => {
                    current.image_stabilization.map(ControlValue::Bool)
                }
            };
            result.push((info, value));
        }
        Ok(result)
    }

    /// Retrieves the current value of a specific control.
    ///
    /// The value is extracted from the result of `get_controls()`, mapped
    /// to a `ControlValue`. Note that some controls may be missing if not supported.
    ///
    /// # Arguments
    ///
    /// * `control_id` - The identifier of the control to query.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ControlValue))`: If the control is supported and has a value.
    /// * `Ok(None)`: If the control is unsupported.
    ///
    /// # Errors
    ///
    /// * `HeadlessError::BackendError`: If `get_controls` fails.
    /// * `HeadlessError::Closed`: If the session is closed.
    pub fn get_control(
        &self,
        control_id: crate::headless::controls::ControlId,
    ) -> Result<Option<crate::headless::controls::ControlValue>, HeadlessError> {
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
        if inner.stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
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
    let mut audio_capture =
        match AudioCapture::new(inner.config.audio_device_id.as_deref(), 48000, 2, pts_clock) {
            Ok(cap) => cap,
            Err(_) => return, // Audio failed
        };

    if audio_capture.start().is_err() {
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
    let data = frame
        .samples
        .iter()
        .flat_map(|&s| s.to_le_bytes())
        .collect();

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
        (ControlId::WhiteBalance, ControlValue::WhiteBalance(v)) => {
            controls.white_balance = Some(v)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::errors::HeadlessErrorKind;
    use crate::types::{CameraFormat, WhiteBalance};

    fn make_test_handle(state: SessionState) -> SessionHandle {
        let config = CaptureConfig::new("test-device".to_string(), CameraFormat::standard());
        SessionHandle {
            inner: Arc::new(Inner {
                state: Mutex::new(state),
                camera: Mutex::new(None),
                config,
                queue: Queue::new(2),
                start_instant: Instant::now(),
                next_sequence: Mutex::new(1),
                capture_thread: Mutex::new(None),
                stop_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                #[cfg(feature = "audio")]
                pts_clock: PTSClock::new(),
                #[cfg(feature = "audio")]
                audio_enabled: false,
                #[cfg(feature = "audio")]
                audio_queue: None,
                #[cfg(feature = "audio")]
                audio_thread: Mutex::new(None),
                #[cfg(feature = "audio")]
                audio_sequence: Mutex::new(1),
            }),
        }
    }

    #[test]
    fn test_queue_capacity_drop_and_close_behavior() {
        let q = Queue::new(2);
        q.push_drop_oldest(1u8);
        q.push_drop_oldest(2u8);
        q.push_drop_oldest(3u8); // drops 1

        assert_eq!(q.dropped(), 1);
        assert_eq!(q.pop_timeout(Duration::ZERO).expect("pop should work"), Some(2));
        assert_eq!(q.pop_timeout(Duration::ZERO).expect("pop should work"), Some(3));
        assert_eq!(q.pop_timeout(Duration::ZERO).expect("empty queue"), None);

        q.close();
        let err = q
            .pop_timeout(Duration::from_millis(1))
            .expect_err("closed queue should error");
        assert_eq!(err.kind, HeadlessErrorKind::Closed);
    }

    #[test]
    fn test_queue_zero_capacity_is_clamped_to_one() {
        let q = Queue::new(0);
        q.push_drop_oldest(10u8);
        q.push_drop_oldest(11u8);
        assert_eq!(q.dropped(), 1);
        assert_eq!(q.pop_timeout(Duration::ZERO).expect("pop should work"), Some(11));
    }

    #[test]
    fn test_start_stop_and_close_state_transitions_without_camera() {
        let handle = make_test_handle(SessionState::Open);

        // Preload warmup frame so start exits warmup loop quickly.
        handle.inner.queue.push_drop_oldest(Frame {
            sequence: 1,
            timestamp_us: 1,
            width: 1,
            height: 1,
            format: "RGB".to_string(),
            device_id: "test-device".to_string(),
            data: vec![0, 0, 0],
        });

        handle.start().expect("start should succeed from open state");
        assert_eq!(*handle.inner.state.lock().expect("state lock"), SessionState::Started);

        handle
            .stop(Duration::from_millis(50))
            .expect("stop should succeed from started state");
        assert_eq!(*handle.inner.state.lock().expect("state lock"), SessionState::Stopped);

        handle
            .close(Duration::from_millis(50))
            .expect("close should succeed");
        assert_eq!(*handle.inner.state.lock().expect("state lock"), SessionState::Closed);
    }

    #[test]
    fn test_get_frame_state_guards_and_started_pop() {
        let closed = make_test_handle(SessionState::Closed);
        assert_eq!(
            closed
                .get_frame(Duration::ZERO)
                .expect_err("closed should fail")
                .kind,
            HeadlessErrorKind::Closed
        );

        let stopped = make_test_handle(SessionState::Stopped);
        assert_eq!(
            stopped
                .get_frame(Duration::ZERO)
                .expect_err("stopped should fail")
                .kind,
            HeadlessErrorKind::Stopped
        );

        let open = make_test_handle(SessionState::Open);
        assert_eq!(
            open.get_frame(Duration::ZERO)
                .expect_err("open should fail")
                .kind,
            HeadlessErrorKind::InvalidArgument
        );

        let started = make_test_handle(SessionState::Started);
        started.inner.queue.push_drop_oldest(Frame {
            sequence: 9,
            timestamp_us: 99,
            width: 2,
            height: 2,
            format: "RGB".to_string(),
            device_id: "dev".to_string(),
            data: vec![1, 2, 3, 4],
        });

        let frame = started
            .get_frame(Duration::ZERO)
            .expect("started get_frame should succeed")
            .expect("frame should be present");
        assert_eq!(frame.sequence, 9);
    }

    #[test]
    fn test_dropped_frames_and_audio_packet_guard() {
        let started = make_test_handle(SessionState::Started);
        started.inner.queue.push_drop_oldest(Frame {
            sequence: 1,
            timestamp_us: 1,
            width: 1,
            height: 1,
            format: "RGB".to_string(),
            device_id: "dev".to_string(),
            data: vec![0],
        });
        started.inner.queue.push_drop_oldest(Frame {
            sequence: 2,
            timestamp_us: 2,
            width: 1,
            height: 1,
            format: "RGB".to_string(),
            device_id: "dev".to_string(),
            data: vec![0],
        });
        started.inner.queue.push_drop_oldest(Frame {
            sequence: 3,
            timestamp_us: 3,
            width: 1,
            height: 1,
            format: "RGB".to_string(),
            device_id: "dev".to_string(),
            data: vec![0],
        });
        assert_eq!(started.dropped_frames().expect("dropped should work"), 1);

        let closed = make_test_handle(SessionState::Closed);
        assert_eq!(
            closed
                .dropped_frames()
                .expect_err("closed should fail")
                .kind,
            HeadlessErrorKind::Closed
        );

        #[cfg(not(feature = "audio"))]
        {
            let err = started
                .get_audio_packet(Duration::ZERO)
                .expect_err("audio disabled should return unsupported");
            assert_eq!(err.kind, HeadlessErrorKind::Unsupported);
        }
    }

    #[test]
    fn test_apply_control_to_struct_and_normalize_frame() {
        let mut controls = CameraControls::default();
        apply_control_to_struct(&mut controls, ControlId::AutoFocus, ControlValue::Bool(false));
        apply_control_to_struct(
            &mut controls,
            ControlId::FocusDistance,
            ControlValue::F32(0.7),
        );
        apply_control_to_struct(
            &mut controls,
            ControlId::WhiteBalance,
            ControlValue::WhiteBalance(WhiteBalance::Cloudy),
        );
        assert_eq!(controls.auto_focus, Some(false));
        assert_eq!(controls.focus_distance, Some(0.7));
        assert_eq!(controls.white_balance, Some(WhiteBalance::Cloudy));

        let handle = make_test_handle(SessionState::Started);
        let frame = CameraFrame::new(vec![1, 2, 3], 3, 1, "dev".to_string()).with_format("RGB".to_string());
        let normalized = normalize_frame(&handle.inner, frame);

        assert_eq!(normalized.sequence, 1);
        assert_eq!(normalized.width, 3);
        assert_eq!(normalized.height, 1);
        assert_eq!(normalized.device_id, "dev");
        assert_eq!(normalized.data.len(), 3);
        assert!(normalized.timestamp_us > 0);
    }

    #[test]
    fn test_stop_and_close_error_guards() {
        let closed = make_test_handle(SessionState::Closed);
        assert_eq!(
            closed
                .stop(Duration::from_millis(1))
                .expect_err("closed stop should fail")
                .kind,
            HeadlessErrorKind::AlreadyClosed
        );
        assert_eq!(
            closed
                .close(Duration::from_millis(1))
                .expect_err("closed close should fail")
                .kind,
            HeadlessErrorKind::AlreadyClosed
        );

        let open = make_test_handle(SessionState::Open);
        assert_eq!(
            open.stop(Duration::from_millis(1))
                .expect_err("open stop should fail")
                .kind,
            HeadlessErrorKind::AlreadyStopped
        );
    }
}
