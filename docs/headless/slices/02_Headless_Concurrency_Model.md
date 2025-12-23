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
