pub mod decode;
pub mod output;

pub use decode::{probe_path, AudioError};
pub use output::{list_output_devices, play_sine, OutputDeviceInfo, PlaybackInfo};
