#Spell: Headless_Control_Surface_Contract
^ Intent: expose camera controls with strict validation, explicit unsupported semantics, and consistent error handling across platforms

@Core:Control_Surface_Contract
  : (device_id, control_id, value) -> applied_or_explicit_error
  ! list_controls_is_deterministic_order
  ! list_controls_includes_current_and_range
  ! set_control_validates_range_and_kind_before_apply
  ! unsupported_controls_return_explicit_Unsupported_error
  ! get_control_round_trips_when_set
  ! set_control_during_capture_allowed
  ! control_apply_failure_does_not_stop_capture
  ! platform_differences_documented_explicitly
  - best_effort_set_without_error
  - silent_clamping_without_disclosure
  - implicit_control_application_delay
  - cross_platform_behavior_undefined
  ~ platform_controls_vary_by_hardware_and_driver