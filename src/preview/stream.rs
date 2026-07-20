use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "tauri")]
use tauri::Emitter;
use tauri::Runtime;

use crate::platform::PlatformCamera;
use crate::preview::encode::{downsample_frame, encode_frame_jpeg};
use crate::preview::types::{PreviewConfig, PreviewFrameEvent};
use crate::quality::smart_trigger::{SmartTrigger, TriggerStatus};
use crate::quality::QualityReport;

pub struct PreviewStream {
    tx: broadcast::Sender<PreviewFrameEvent>,
    cancel: CancellationToken,
}

impl PreviewStream {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self {
            tx,
            cancel: CancellationToken::new(),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PreviewFrameEvent> {
        self.tx.subscribe()
    }

    pub async fn start<R: Runtime>(
        &self,
        camera: Arc<StdMutex<PlatformCamera>>,
        config: PreviewConfig,
        mut trigger: SmartTrigger,
        #[cfg(feature = "tauri")] app: Option<tauri::AppHandle<R>>,
    ) -> Result<(), String> {
        config.validate()?;

        let tx = self.tx.clone();
        let cancel = self.cancel.clone();
        let mut frame_number = 0u64;
        let mut last_quality: Option<QualityReport> = None;
        let mut last_sampled_frame = 0u64;

        #[cfg(feature = "tauri")]
        if let Some(ref a) = app {
            let _ = a.emit("crabcamera://preview-state", &serde_json::json!({"running": true}));
        }

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    () = cancel.cancelled() => {
                        #[cfg(feature = "tauri")]
                        if let Some(ref a) = app {
                            let _ = a.emit("crabcamera://preview-state", &serde_json::json!({"running": false}));
                        }
                        break;
                    }
                    () = tokio::time::sleep(Duration::from_millis(u64::from(1000 / config.fps_target))) => {}
                }

                let camera_arc = camera.clone();
                let frame = match tokio::task::spawn_blocking(move || {
                    let mut cam = camera_arc.lock().expect("camera lock");
                    cam.capture_frame()
                })
                .await
                {
                    Ok(Ok(f)) => f,
                    _ => continue,
                };

                frame_number += 1;

                let should_analyze =
                    frame_number.is_multiple_of(u64::from(config.quality_sample_rate));

                let (quality_event, stale_flag, trigger_ready, jpeg_data) = if config.downscale < 1.0 {
                    let preview = downsample_frame(&frame, config.downscale);

                    let (quality, stale, trigger_status) = if should_analyze {
                        let (status, report) = trigger.process_frame(&preview);
                        last_quality = Some(report.clone());
                        last_sampled_frame = frame_number;
                        (Some(report), false, status)
                    } else if let Some(ref cached) = last_quality {
                        (Some(cached.clone()), true, TriggerStatus::Thinking("stale".into()))
                    } else {
                        (None, false, TriggerStatus::Thinking("initial".into()))
                    };

                    let jpeg = match encode_frame_jpeg(&preview, config.jpeg_quality) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };

                    (quality, stale, trigger_status == TriggerStatus::Ready, jpeg)
                } else {
                    let (quality, stale, trigger_status) = if should_analyze {
                        let (status, report) = trigger.process_frame(&frame);
                        last_quality = Some(report.clone());
                        last_sampled_frame = frame_number;
                        (Some(report), false, status)
                    } else if let Some(ref cached) = last_quality {
                        (Some(cached.clone()), true, TriggerStatus::Thinking("stale".into()))
                    } else {
                        (None, false, TriggerStatus::Thinking("initial".into()))
                    };

                    let jpeg = match encode_frame_jpeg(&frame, config.jpeg_quality) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };

                    (quality, stale, trigger_status == TriggerStatus::Ready, jpeg)
                };

                let event = PreviewFrameEvent {
                    jpeg_data,
                    quality: quality_event,
                    stale: stale_flag,
                    last_sampled_frame,
                    is_smart_trigger_ready: trigger_ready,
                    timestamp: chrono::Utc::now(),
                    frame_number,
                };

                let _ = tx.send(event.clone());

                #[cfg(feature = "tauri")]
                if let Some(ref a) = app {
                    let _ = a.emit("crabcamera://preview-frame", &event);
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.cancel.cancel();
    }
}

impl Default for PreviewStream {
    fn default() -> Self {
        Self::new()
    }
}
