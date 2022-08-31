#![feature(unwrap_infallible)]

mod capture;
mod hook;
mod ipc;
mod stream;

pub use rwig_common::audio;

pub use capture::Capture;
pub use hook::{CompatibilityInfo, Injector};
pub use stream::CaptureStream;
