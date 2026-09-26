pub mod camera_physics;
pub mod client;
pub mod error;
pub mod local;
pub mod models;
pub mod server;
pub mod storyboard;
pub mod studio;

pub use camera_physics::CameraSpringDamping;
pub use client::{GenerationResponse, HiggsfieldClient, DEFAULT_BASE_URL};
pub use error::{Error, Result};
pub use local::{HardwareProfile, LocalEngine};
pub use models::{Model, ModelModality, ModelRegistry};
pub use server::run_server;
pub use storyboard::{ShotType, Storyboard, StoryboardShot};
pub use studio::{
    build_nano_banana_prompt, compile_cinema_prompt, CameraMotion3D, CinemaCamera, CinemaLens,
    CinemaLighting, CinemaStudio, CinemaStudioRequest, CompiledCinemaPrompt, ImageStudio,
    ImageStudioRequest, LipSyncStudio, LipSyncStudioRequest, VideoStudio, VideoStudioRequest,
};
