use crate::studio::CameraMotion3D;
use serde::{Deserialize, Serialize};

/// Spring-damper physics model inspired by `poormans-camera-rs`.
/// Replaces rigid linear transitions with organic physical camera inertia and damping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSpringDamping {
    pub mass: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub shake_intensity: f32,
}

impl Default for CameraSpringDamping {
    fn default() -> Self {
        Self {
            mass: 1.0,
            stiffness: 120.0,
            damping: 18.0,
            shake_intensity: 0.05,
        }
    }
}

impl CameraSpringDamping {
    pub fn cinematic_handheld() -> Self {
        Self {
            mass: 1.2,
            stiffness: 80.0,
            damping: 12.0,
            shake_intensity: 0.15,
        }
    }

    pub fn heavy_technocrane() -> Self {
        Self {
            mass: 4.5,
            stiffness: 240.0,
            damping: 45.0,
            shake_intensity: 0.0,
        }
    }

    pub fn steadicam() -> Self {
        Self {
            mass: 2.0,
            stiffness: 150.0,
            damping: 25.0,
            shake_intensity: 0.02,
        }
    }

    /// Computes smoothed trajectory sample points over duration_secs at given fps.
    pub fn smooth_trajectory(
        &self,
        motion: &CameraMotion3D,
        duration_secs: f32,
        fps: u32,
    ) -> Vec<CameraMotion3D> {
        let total_frames = ((duration_secs * (fps as f32)).round() as usize).max(1);
        let dt = 1.0 / (fps as f32);

        let mut samples = Vec::with_capacity(total_frames);

        let mut current_pan = 0.0f32;
        let mut pan_vel = 0.0f32;

        let mut current_tilt = 0.0f32;
        let mut tilt_vel = 0.0f32;

        let mut current_zoom = 1.0f32;
        let mut zoom_vel = 0.0f32;

        let mut current_dolly = 0.0f32;
        let mut dolly_vel = 0.0f32;

        for frame in 0..total_frames {
            let progress = (frame as f32) / (total_frames as f32);
            let target_pan = motion.pan_deg * progress;
            let target_tilt = motion.tilt_deg * progress;
            let target_zoom = 1.0 + (motion.zoom_scale - 1.0) * progress;
            let target_dolly = motion.dolly_m * progress;

            // Spring force calculation: F = -k * (x - target) - c * v
            let pan_force = -self.stiffness * (current_pan - target_pan) - self.damping * pan_vel;
            pan_vel += (pan_force / self.mass) * dt;
            current_pan += pan_vel * dt;

            let tilt_force = -self.stiffness * (current_tilt - target_tilt) - self.damping * tilt_vel;
            tilt_vel += (tilt_force / self.mass) * dt;
            current_tilt += tilt_vel * dt;

            let zoom_force = -self.stiffness * (current_zoom - target_zoom) - self.damping * zoom_vel;
            zoom_vel += (zoom_force / self.mass) * dt;
            current_zoom += zoom_vel * dt;

            let dolly_force = -self.stiffness * (current_dolly - target_dolly) - self.damping * dolly_vel;
            dolly_vel += (dolly_force / self.mass) * dt;
            current_dolly += dolly_vel * dt;

            // Add subtle organic micro-shake if enabled
            let shake = if self.shake_intensity > 0.0 {
                (frame as f32 * 0.3).sin() * self.shake_intensity
            } else {
                0.0
            };

            samples.push(CameraMotion3D {
                pan_deg: current_pan + shake,
                tilt_deg: current_tilt + shake * 0.5,
                roll_deg: motion.roll_deg * progress,
                zoom_scale: current_zoom,
                dolly_m: current_dolly,
                truck_m: motion.truck_m * progress,
                crane_m: motion.crane_m * progress,
                orbit_deg: motion.orbit_deg * progress,
            });
        }

        samples
    }
}
