#Spell: Headless_Frame_Delivery_Contract
^ Intent: define exact video frame delivery semantics including ordering, drops, observability, and timeout behavior

@Core:Frame_Delivery_Contract
  : (session, timeout) -> Option<Frame>
  ! frames_have_monotonic_sequence_numbers
  ! frames_have_monotonic_timestamps
  ! next_frame_returns_None_only_on_timeout
  ! next_frame_returns_error_on_permanent_failure
  ! delivery_order_preserved_for_delivered_frames
  ! drop_count_exposed_via_session_telemetry
  ! buffer_policy_honored_and_observable
  ! frame_contains_format_metadata
  - silent_drop_without_observability
  - unbounded_memory_growth
  - ambiguous_None_meaning_timeout_vs_end
  - frame_reordering_implicit
  - data_lifetime_undefined
  ~ backend_provides_frames_or_explicit_timeout

@Core:BufferPolicy
  : producer_rate -> bounded_queue_behavior
  ! DropOldest_drops_oldest_frames_first
  ! QueueN_bounds_memory_by_N_frames
  ! drop_count_incremented_and_exposed
  ! policy_honored_on_every_drop
  ! policy_selected_at_session_creation
  - unbounded_queue_growth
  - implicit_blocking_producer_forever
  - silent_drops_without_telemetry
  ~ max_frame_size_known_or_bounded

@Core:Frame
  : backend_frame -> normalized_frame
  ! includes_timestamp_monotonic
  ! includes_sequence_number_monotonic
  ! includes_format_metadata
  ! data_ownership_explicit_owned_bytes
  ! size_and_stride_consistent_with_format
  - borrowed_data_with_undefined_lifetime
  - platform_opaque_handles
  - implicit_drop_semantics