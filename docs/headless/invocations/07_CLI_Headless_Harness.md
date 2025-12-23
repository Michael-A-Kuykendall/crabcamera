#Spell: CLI_Headless_Harness
^ Intent: provide a reference CLI that exercises headless contracts end-to-end without becoming a new product surface or embedding business logic

@CLI:CrabCamera
  : argv -> exit_code
  ! uses_headless_session_only
  ! ctrl_c_triggers_stop_then_close_cleanly
  ! supports_json_output_for_automation
  ! timeouts_user_configurable_via_flags
  ! all_commands_return_deterministic_output
  ! supports_mock_backend_for_ci
  - embedding_business_logic_in_cli
  - ui_preview_features
  - background_service_installation
  - persistent_configuration_state
  ~ filesystem_writable_for_optional_outputs

@CLI:Commands
  : (devices, formats, controls, set, capture, record) -> human_and_json_output
  ! devices_outputs_stable_ids_and_names
  ! devices_outputs_json_mode_for_parsing
  ! formats_enumerates_all_supported_modes
  ! controls_lists_current_values_and_ranges
  ! capture_reports_drop_count_fps_latency
  ! capture_respects_buffer_policy_via_observability
  ! record_uses_headless_session_not_separate_path
  - recording_by_default
  - network_streaming_modes
  - graphical_preview
  ~ filesystem_able_to_write_outputs

@CLI:Output_Formats
  : verbose_telemetry -> table_and_json
  ! human_readable_tables_with_alignment
  ! json_output_deterministic_and_parseable
  ! timestamps_iso8601_or_milliseconds
  - binary_or_protobuf_output
  - custom_csv_formats
  ~ jq_and_shell_compatible