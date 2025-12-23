#Spell: Headless_Audio_Delivery_Contract
^ Intent: define exact audio delivery semantics and ensure audio cannot compromise video capture stability through strict isolation

@Core:Audio_Delivery_Contract
  : (session, timeout) -> Option<AudioPacket>
  ! audio_optional_and_feature_gated
  ! audio_failure_does_not_stop_video_capture_by_default
  ! audio_unavailable_returns_explicit_error_not_silent_none
  ! next_audio_returns_None_only_on_timeout
  ! next_audio_returns_error_on_permanent_audio_failure
  ! audio_pts_synchronized_with_video_pts_clock
  ! audio_and_video_timestamps_comparable
  - audio_required_or_capture_blocked
  - video_capture_aborts_on_audio_device_absent
  - audio_failures_silently_suppress_video
  - implicit_pts_desynchronization
  ~ audio_enabled_only_if_configured_and_feature_enabled

@Core:AudioPacket
  : backend_audio -> normalized_audio_packet
  ! includes_timestamp_from_shared_pts_clock
  ! includes_format_samplerate_channels
  ! includes_sequence_number
  ! data_ownership_explicit_owned_bytes
  ! packet_size_bounded_and_documented
  ! buffer_contains_pcm_or_encoded_as_specified
  - unbounded_packet_buffers
  - implicit_resampling_without_disclosure
  - channel_mixing_without_notification
  ~ audio_capture_idempotent_on_stop_start