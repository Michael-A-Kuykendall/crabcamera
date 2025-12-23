#Spell: Headless_Lifecycle_Idempotency
^ Intent: make session lifecycle safe under retries, Ctrl-C, and partial failures without deadlocks or resource leaks

@Core:Lifecycle_Idempotency
  : (open, start, stop, close, drop) -> bounded_shutdown
  ! open_allocates_no_capture_threads
  ! open_returns_error_if_device_invalid
  ! start_is_idempotent_or_explicit_AlreadyStarted_error
  ! stop_is_idempotent_or_explicit_AlreadyStopped_error
  ! close_is_idempotent_or_explicit_AlreadyClosed_error
  ! close_joins_all_threads_with_timeout
  ! drop_is_best_effort_non_hanging_within_100ms
  ! all_state_transitions_explicit_in_error_enum
  - infinite_join_on_any_operation
  - deadlock_on_stop_or_close
  - implicit_restart_on_error
  - threads_left_running_after_close
  ~ platform_backend_can_signal_stop
  ~ timeouts_are_reasonable_and_configurable

@Core:Errors
  : backend_failures -> explicit_lifecycle_errors
  ! distinguishes_AlreadyStarted_AlreadyStopped_AlreadyClosed
  ! distinguishes_timeout_vs_permanent_failure
  - generic_failure_message
  ~ errors_implement_std_error_trait