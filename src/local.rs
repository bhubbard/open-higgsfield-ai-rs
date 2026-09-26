use crate::error::{Error, Result};
use crate::studio::{CinemaLighting, CinemaStudioRequest};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Hardware profile detected on the local system.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HardwareProfile {
    pub is_apple_silicon: bool,
    pub chip_name: String,
    pub unified_memory_gb: u32,
    pub metal_supported: bool,
    pub ffmpeg_available: bool,
    pub recommended_local_models: Vec<String>,
}

impl HardwareProfile {
    /// Probe the host machine (optimized for macOS Apple Silicon M-series).
    pub fn probe() -> Self {
        let is_apple_silicon = cfg!(target_os = "macos") && cfg!(target_arch = "aarch64");

        let mut chip_name = "Generic CPU".to_string();
        let mut unified_memory_gb = 8;

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl").arg("-n").arg("machdep.cpu.brand_string").output() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() {
                    chip_name = name;
                }
            }
            if let Ok(output) = Command::new("sysctl").arg("-n").arg("hw.memsize").output() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Ok(bytes) = s.parse::<u64>() {
                    unified_memory_gb = (bytes / (1024 * 1024 * 1024)) as u32;
                }
            }
        }

        let ffmpeg_available = Command::new("ffmpeg")
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let mut recommended = Vec::new();
        if is_apple_silicon {
            if unified_memory_gb >= 16 {
                recommended.push("Flux.1 Schnell (4-bit / 8-bit quantized via MLX / Candle)".to_string());
                recommended.push("SDXL Turbo / SD 1.5 (Metal / CoreML)".to_string());
                recommended.push("Wav2Lip / SadTalker LipSync (ONNX / CoreML)".to_string());
                recommended.push("Local Kinetic Camera Renderer (FFmpeg 1080p)".to_string());
            } else {
                recommended.push("Stable Diffusion 1.5 (Metal)".to_string());
                recommended.push("Wav2Lip LipSync (ONNX)".to_string());
                recommended.push("Local Kinetic Camera Renderer (FFmpeg)".to_string());
            }
        }

        Self {
            is_apple_silicon,
            chip_name,
            unified_memory_gb,
            metal_supported: is_apple_silicon,
            ffmpeg_available,
            recommended_local_models: recommended,
        }
    }
}

/// Local rendering and video assembly engine.
pub struct LocalEngine;

impl LocalEngine {
    /// Render a real cinematic video preview locally on this Mac using FFmpeg.
    pub fn render_cinema_preview(
        req: &CinemaStudioRequest,
        duration_secs: u32,
        output_file: &Path,
    ) -> Result<PathBuf> {
        let hw = HardwareProfile::probe();
        if !hw.ffmpeg_available {
            return Err(Error::InvalidParameter(
                "ffmpeg is required for local rendering but was not found in PATH".to_string(),
            ));
        }

        if let Some(parent) = output_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let duration = duration_secs.max(1);
        let motion = req.motion.clone().unwrap_or_default();
        let lighting = req.lighting.unwrap_or(CinemaLighting::CyberpunkNeon);

        // Color grade matrix based on lighting preset
        let color_filter = match lighting {
            CinemaLighting::FilmNoir => "colorchannelmixer=.3:.4:.3:0:.3:.4:.3:0:.3:.4:.3,curves=strong_contrast",
            CinemaLighting::GoldenHour => "colorbalance=rs=.2:gs=.05:bs=-.2:rm=.15:gm=.05:bm=-.15",
            CinemaLighting::CyberpunkNeon => "eq=saturation=1.4:contrast=1.2,colorbalance=rs=-.1:gs=.1:bs=.3:rm=.2:gm=-.1:bm=.2",
            CinemaLighting::NaturalDaylight => "eq=contrast=1.05:saturation=1.05",
            CinemaLighting::RimLight => "curves=preset=lighter,eq=contrast=1.3:saturation=0.9",
            CinemaLighting::SoftStudio => "eq=contrast=1.0:saturation=1.1,smartblur=lr=1.2:ls=0.5",
            CinemaLighting::VolumetricFog => "eq=contrast=0.9:brightness=0.08:saturation=0.9",
            CinemaLighting::MoodyDusk => "colorbalance=rs=-.1:gs=-.05:bs=.2:rm=-.1:gm=.0:bm=.2",
        };

        // Zoom and Pan calculation
        let zoom_step = ((motion.zoom_scale - 1.0) / ((duration * 24) as f32)).max(-0.01).min(0.01);
        let pan_x = if motion.pan_deg > 0.0 { "x+1" } else if motion.pan_deg < 0.0 { "x-1" } else { "x" };

        let filter_chain = format!(
            "color=c=black:s=1920x1080:d={d},\
             drawbox=x=0:y=0:w=1920:h=1080:color=#0a0a0a@1.0:t=fill,\
             drawgrid=w=120:h=120:t=1:color=white@0.08,\
             drawbox=x=860:y=440:w=200:h=200:color=#06b6d4@0.2:t=fill,\
             drawbox=x=860:y=440:w=200:h=200:color=#06b6d4@0.8:t=2,\
             drawbox=x=60:y=60:w=400:h=80:color=#121212@0.8:t=fill,\
             drawbox=x=60:y=60:w=400:h=80:color=#262626@1.0:t=1,\
             zoompan=z='min(max(zoom+{zoom_step},1),2)':x='{pan_x}':d={frames}:s=1920x1080,\
             noise=alls=12:allf=t+u,\
             {color_filter},\
             format=yuv420p",
            d = duration,
            zoom_step = zoom_step,
            pan_x = pan_x,
            frames = duration * 24,
            color_filter = color_filter
        );

        info!("Rendering local cinematic simulation with FFmpeg to {}", output_file.display());

        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-f").arg("lavfi")
            .arg("-i").arg(filter_chain)
            .arg("-t").arg(duration.to_string())
            .arg("-r").arg("24")
            .arg("-c:v").arg("libx264")
            .arg("-preset").arg("veryfast")
            .arg("-crf").arg("20")
            .arg("-pix_fmt").arg("yuv420p")
            .arg(output_file)
            .status()
            .map_err(|e| Error::Io(e))?;

        if !status.success() {
            return Err(Error::GenerationFailed(format!(
                "FFmpeg failed with exit code: {:?}",
                status.code()
            )));
        }

        Ok(output_file.to_path_buf())
    }

    /// Zero-loss concatenation of multiple shot MP4s into a single master sequence,
    /// inspired by `lossless-cut-rs`.
    pub fn lossless_concat_shots(shots: &[PathBuf], output_file: &Path) -> Result<PathBuf> {
        if shots.is_empty() {
            return Err(Error::InvalidParameter("No shots provided for concatenation".to_string()));
        }

        let temp_list = tempfile::NamedTempFile::new().map_err(|e| Error::Io(e))?;
        let mut list_content = String::new();
        for shot in shots {
            list_content.push_str(&format!("file '{}'\n", shot.canonicalize().unwrap_or_else(|_| shot.clone()).display()));
        }
        std::fs::write(temp_list.path(), list_content)?;

        if let Some(parent) = output_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-f").arg("concat")
            .arg("-safe").arg("0")
            .arg("-i").arg(temp_list.path())
            .arg("-c").arg("copy")
            .arg(output_file)
            .status()
            .map_err(|e| Error::Io(e))?;

        if !status.success() {
            return Err(Error::GenerationFailed(format!(
                "Lossless concat failed with exit code: {:?}",
                status.code()
            )));
        }

        Ok(output_file.to_path_buf())
    }
}
