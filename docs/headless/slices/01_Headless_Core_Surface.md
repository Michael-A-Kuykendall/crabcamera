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
