pub mod cinema;
pub mod image;
pub mod lipsync;
pub mod video;

pub use cinema::{
    build_nano_banana_prompt, compile_cinema_prompt, CameraMotion3D, CinemaCamera, CinemaLens,
    CinemaLighting, CinemaStudio, CinemaStudioRequest, CompiledCinemaPrompt,
};
pub use image::{ImageStudio, ImageStudioRequest};
pub use lipsync::{LipSyncStudio, LipSyncStudioRequest};
pub use video::{VideoStudio, VideoStudioRequest};
