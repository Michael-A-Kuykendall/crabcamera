/// Preview stream types (events and configuration).
pub mod types;
/// JPEG encoding and downscaling helpers.
pub mod encode;
/// PreviewStream — push-based frame + metadata delivery.
pub mod stream;

pub use types::{PreviewConfig, PreviewFrameEvent};
pub use stream::PreviewStream;