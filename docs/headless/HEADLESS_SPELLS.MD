#Spell: Headless_Core_Surface
^ Intent: expose CrabCamera capture and control as a UI-free library surface usable without Tauri

@HeadlessSurface
: (device_query, format_query, control_query, session_ops) -> (devices, formats, controls, session)
! tauri_independent
! platform_agnostic_types_only
! deterministic_api_shape
! public_buffers_are_owned_and_normalized
- ui_event_loop_dependency
- tauri_command_macros
- global_singleton_required
~ platform_backends_exist
> PlatformCamera
> Types
> Errors

@Types
: platform_state -> (DeviceInfo, FormatInfo, ControlInfo, Frame, AudioPacket)
! serializable_boundary_safe
! no_tauri_types
! owned_buffer_representation
- ui_handles
- window_handles

@Errors
: backend_failures -> stable_error_set
! error_is_structured
! distinguishes_timeout_vs_failure
- stringly_typed_errors

---

#Spell: Headless_Concurrency_Model
^ Intent: define concurrency behavior for headless capture with explicit timeout semantics

@ConcurrencyModel
: (session_ops, frame_ops, audio_ops, timeout) -> blocking_calls_with_timeouts
! no_required_async_runtime
! all_waits_bounded_by_timeout
! timeout_zero_is_poll
! timeout_units_are_duration
! internal_threads_interruptible
- caller_must_use_async_runtime
- unbounded_blocking
~ platform_backends_support_interrupt
> Lifecycle_Idempotency
> Frame_Delivery_Contract
> Audio_Delivery_Contract

---

#Spell: Headless_Session_API
^ Intent: provide a session-based headless capture API with explicit state transitions

@HeadlessSession
: CaptureConfig -> SessionHandle
! session_scoped_state_only
! no_global_registry
! explicit_state_machine
! resources_released_on_close
- implicit_singletons
- hidden_background_daemons
~ device_id_valid_or_error
> PlatformCamera
> Lifecycle_Idempotency
> Frame_Delivery_Contract
> Control_Surface_Contract
> Errors

@CaptureConfig
: (device_id, format, audio_mode, buffer_policy) -> session_parameters
! all_fields_explicit
! defaults_documented
- undocumented_defaults
~ requested_format_supported_or_error

---

#Spell: Lifecycle_Idempotency
^ Intent: guarantee safe lifecycle transitions across all session states

@Lifecycle
: (open, start, stop, close, drop) -> bounded_shutdown
! open_allocates_no_capture_threads
! start_idempotent_or_error
! stop_idempotent_or_error
! close_idempotent_or_error
! close_joins_threads_with_timeout
! drop_best_effort_non_hanging
! calls_after_close_return_error
- infinite_join
- deadlock_on_stop
- implicit_restart
~ backend_can_signal_stop
> Errors

---

#Spell: Frame_Delivery_Contract
^ Intent: define exact video frame delivery semantics with clarified timestamps and drops

@FrameDelivery
: (session, timeout) -> optional_frame
! frames_monotonic_sequence
! frame_timestamp_from_internal_monotonic_clock
! none_only_on_timeout
! error_on_session_failure
! error_on_closed_or_stopped
! delivery_order_preserved
- silent_drop
- unbounded_memory_growth
- ambiguous_none
~ backend_provides_frames
> BufferPolicy
> Errors

@BufferPolicy
: producer_rate -> bounded_queue_behavior
! dropoldest_drops_oldest
! queue_n_bounds_memory
! drop_count_cumulative_per_session
! drop_count_read_only
- unbounded_queue
- implicit_forever_block
~ max_frame_size_known

@Frame
: backend_frame -> normalized_frame
! includes_timestamp
! includes_sequence
! includes_format
! owned_packed_layout
- borrowed_undefined_lifetime

---

#Spell: Audio_Delivery_Contract
^ Intent: define audio delivery semantics with explicit failure and recovery behavior

@AudioDelivery
: (session, timeout) -> optional_audio_packet
! audio_optional_feature_gated_compile_time
! audio_failure_does_not_stop_video
! audio_failure_is_terminal
! none_only_on_timeout
! error_on_audio_failure
! error_on_closed_or_stopped
- audio_required
- video_abort_on_audio_missing
~ audio_enabled_if_compiled
> Errors
> Lifecycle_Idempotency

@AudioPacket
: backend_audio -> normalized_audio_packet
! includes_timestamp
! includes_format
! bounded_packet_size
! owned_buffer
- unbounded_packet_buffers

---

#Spell: Control_Surface_Contract
^ Intent: expose camera controls with explicit validation and stability rules

@ControlSurface
: (device_id, control_id, value) -> applied_or_error
! deterministic_control_listing
! validates_range_and_kind
! unsupported_returns_error
! round_trip_when_supported
! device_id_stable_within_process
- silent_clamping
- best_effort_without_error
~ hardware_controls_vary
> Errors
> Types

---

#Spell: CLI_Headless_Harness
^ Intent: provide a reference CLI exercising headless contracts deterministically

@CLI
: argv -> exit_code
! uses_headless_session_only
! ctrl_c_triggers_stop_and_close
! json_output_supported
! timeouts_configurable
! device_ids_not_persisted_across_runs
- embedded_business_logic
- ui_preview_required
- background_daemon_install
~ filesystem_writable
> HeadlessSession
> Lifecycle_Idempotency
> Frame_Delivery_Contract
> Control_Surface_Contract

@CLICommands
: (devices, formats, controls, set, capture) -> outputs
! stable_ids_within_listing
! capture_reports_fps_and_drops
- recording_by_default
- network_streaming

---

#Spell: Tauri_Adapter_Preservation
^ Intent: preserve existing Tauri behavior while routing through headless core

@TauriAdapter
: tauri_calls -> core_calls
! thin_adapter_only
! no_platform_logic
! existing_contracts_preserved
- duplicated_capture_logic
- new_global_state
~ async_bridge_required
> HeadlessSession
> Types
> Errors

---

#Spell: Mock_Backend_For_CI
^ Intent: enable deterministic CI testing via explicit backend injection seam

@MockBackend
: deterministic_seed -> synthetic_devices_and_frames
! injectable_backend_entrypoint
! supports_devices_formats_controls
! supports_drop_policy
! supports_timeout_paths
- nondeterministic_generation
- real_hardware_dependency
~ mock_feature_enabled
> PlatformCamera
> Frame_Delivery_Contract
> Control_Surface_Contract

@CIMatrix
: features -> tested_builds
! enumerated_feature_sets_only
! includes_no_default_features
- combinatorial_explosion
- untested_paths
~ ci_available
