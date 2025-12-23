#Spell: Headless_Session_API
^ Intent: provide a session-based headless capture API that eliminates global registry coupling and enables safe resource cleanup

@Core:HeadlessSession
  : CaptureConfig -> SessionHandle
  ! session_scoped_state_only
  ! no_global_registry_required_in_headless_mode
  ! resources_released_deterministically_on_close
  ! open_start_stop_close_ordered_contract
  ! all_operations_return_explicit_result
  - implicit_singleton_registry
  - hidden_background_daemons_without_join
  - session_state_in_command_context
  ~ device_id_is_valid_or_errors

@Core:CaptureConfig
  : (device_id, format, audio_mode, buffer_policy, timeout_ms) -> session_parameters
  ! all_fields_explicit
  ! defaults_are_documented_and_reasonable
  ! audio_optional_and_feature_gated
  ! buffer_policy_respected_or_explicit_error
  - magic_defaults_undocumented
  - derived_or_inferred_values
  ~ requested_format_supported_or_errors_early

@Core:SessionHandle
  : lifecycle_opaque -> owned_session_value
  ! send_safe
  ! cannot_be_shared_safely_by_default
  ! close_consumes_by_move
  - sync_across_thread_boundaries
  - copyable_or_clonable_without_effort
  ~ drop_impl_available_for_best_effort_cleanup