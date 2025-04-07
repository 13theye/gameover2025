pub mod config_load;
pub mod config_types;

pub use config_load::Config;
pub use config_types::{
    FrameRecorderConfig, OscConfig, PathConfig, RenderConfig, SpeedConfig, WindowConfig,
};
