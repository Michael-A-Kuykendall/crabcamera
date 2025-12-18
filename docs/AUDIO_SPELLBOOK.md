# Sorcery Spellbook — CrabCamera Audio Recording v0.5.0

*A sealed architectural intent compression for adding synchronized audio recording to CrabCamera.*

---

## #Spell: AudioArchitecture

^ Intent: extend CrabCamera video recording to include synchronized audio capture and muxing into standards-compliant MP4 while preserving determinism, legal safety, and graceful degradation

@AudioSystem
  : camera_frames + microphone_pcm -> mp4_file
  ! supports_video_only_mode
  ! supports_video_plus_audio_mode
  ! audio_and_video_share_common_pts_baseline
  ! produces_playable_output_in_mainstream_players
  ! codec = Opus
  ! default_audio = OptIn
  ! sample_rate = 48000
  ! channel_policy = Preserve
  - implicit_codec_selection
  - silent_behavior_change_between_versions
  - resampling_without_disclosure
  - implicit_channel_mixing
  - blocking_video_on_audio_init_failure
  - gpl_contamination
  ~ muxide_supports_opus_audio_tracks
  ~ recording_pipeline_is_single_writer

---

## #Spell: AudioDeviceEnumerate

^ Intent: expose stable, cross-platform enumeration of audio input devices for user selection and default discovery

@AudioDevice
  : system_inputs -> Vec<AudioDevice>
  ! includes(id, name, sample_rate, channels, is_default)
  ! input_devices_only
  ! deterministic_ordering
  - starting_audio_capture
  - inferring_missing_fields
  ~ host_audio_api_exposes_default_device

@AudioAPI
  : () -> (list_devices, default_device)
  ! returns_typed_errors
  - stringly_typed_failure

> @cpal_input_host

---

## #Spell: AudioPTSClock

^ Intent: define a single monotonic timebase for all audio and video timestamps

@PTSClock
  : start_instant -> pts_seconds
  ! monotonic
  ! non_decreasing
  ! shared_by_audio_and_video
  - system_wall_clock
  ~ Instant_is_monotonic

@AudioTimestamp
  : callback_time -> pts
  ! derived_from_PTSClock
  - derived_from_sample_count_without_policy

> @PTSClock

---

## #Spell: AudioCapturePCM

^ Intent: capture microphone audio as timestamped PCM frames with bounded memory and deterministic lifecycle

@AudioCapture
  : (device_id, sample_rate, channels) -> AudioCapture
  ! produces_interleaved_f32_pcm
  ! bounded_buffer
  ! start_is_idempotent
  ! stop_is_idempotent
  ! joins_capture_thread_on_stop
  - unbounded_memory_growth
  - blocking_callback
  ~ device_supports_requested_format_or_negotiation

@AudioFrame
  : pcm_callback -> AudioFrame
  ! includes(samples, sample_rate, channels, timestamp)
  - heap_allocation_per_sample

> @AudioPTSClock
> @cpal_input_stream

---

## #Spell: AudioEncodeOpus

^ Intent: encode PCM audio into Opus packets suitable for MP4 muxing using a license-clean codec

@OpusEncoder
  : pcm_frame -> opus_packet
  ! accepts_f32_pcm
  ! outputs_valid_opus_packets
  ! flush_emits_remaining_packets
  ! operates_at_48khz
  - hidden_resampling
  - changing_channel_count
  ~ opus_requires_48khz
  ~ muxide_opus_accepts_raw_packets

@EncodedAudio
  : pcm -> Vec<u8>
  ! is_opus_packet
  - adts_headers

> @AudioCapturePCM
> @opus_codec

---

## #Spell: RecorderIntegrateAudio

^ Intent: integrate audio capture and encoding into the existing Recorder without destabilizing video recording

@Recorder
  : config -> recorder
  ! supports_audio_optional
  ! configures_muxer_audio_track_when_enabled
  ! continues_video_if_audio_fails
  - writing_audio_without_track_declaration
  - blocking_video_on_audio

@WriteFrame
  : CameraFrame -> ()
  ! writes_video_pts_from_PTSClock
  ! drains_audio_non_blocking
  ! writes_audio_pts_from_audio_frames
  - busy_wait
  - unbounded_audio_drain

> @AudioCapturePCM
> @AudioEncodeOpus
> @AudioPTSClock
> @Muxide

---

## #Spell: AVSyncPolicy

^ Intent: guarantee bounded audio/video synchronization drift in recorded output

@SyncPolicy
  : (start_time, event_time) -> pts
  ! shared_baseline
  ! max_drift <= 100ms
  ! target_drift <= 50ms
  - dual_clock_sources
  ~ capture_callbacks_can_sample_time

> @AudioPTSClock

---

## #Spell: TauriAudioCommands

^ Intent: expose audio device discovery and audio-enabled recording through Tauri commands safely

@Commands
  : ui_request -> recording_action
  ! list_audio_devices_returns_structured_data
  ! start_recording_accepts_audio_device_option
  ! user_safe_error_strings
  - leaking_internal_error_types
  ~ async_safe_execution

> @AudioDeviceEnumerate
> @RecorderIntegrateAudio

---

## #Spell: AudioErrorRecovery

^ Intent: define graceful degradation when audio capture fails without corrupting video output

@ErrorPolicy
  : audio_error -> recovery_action
  ! video_continues_on_audio_failure
  ! error_logged
  ! session_status_reflects_audio_state
  - panic
  - silent_data_loss
  ~ video_pipeline_is_independent

---

## #Spell: AudioDeviceHotplug

^ Intent: DEFERRED to v0.6.0

@HotplugPolicy
  : device_change -> action
  - mid_recording_device_switch

---

## #Spell: RecordingTests_AV

^ Intent: prove that produced recordings contain valid audio and video tracks with bounded sync error

@IntegrationTest
  : record_duration -> mp4
  ! contains_video_track
  ! contains_audio_track_when_enabled
  ! sync_within_policy
  - manual_playback_validation
  ~ external_probe_tool_available

@UnitTests
  : () -> assertions
  ! device_enumeration_safe
  ! capture_start_stop_safe
  ! encoded_audio_headers_valid

> @RecorderIntegrateAudio

---

## #Spell: CargoAudioGating

^ Intent: ensure audio functionality is opt-in and does not affect builds without audio enabled

@CargoFeatures
  : features -> compilation
  ! audio_requires_explicit_feature
  ! recording_without_audio_unaffected
  - implicit_audio_dependency
  ~ cargo_feature_resolution

---

## Seal Status

All spells are sealed. No open questions remain.
