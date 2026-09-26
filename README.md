# open-higgsfield-ai-rs

[![Crates.io](https://img.shields.io/crates/v/open-higgsfield-ai.svg)](https://crates.io/crates/open-higgsfield-ai)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Live Demo](https://img.shields.io/badge/Live%20Demo-code.brandonhubbard.com-06b6d4.svg)](https://code.brandonhubbard.com/open-higgsfield-ai-rs/)

> High-performance native Rust fork and client engine for [Autom8AI/Open-Higgsfield-AI](https://github.com/Autom8AI/Open-Higgsfield-AI). Provides complete support for 220+ generative AI models, 3D cinema camera choreography, optical lens simulation, multi-image conditioning (up to 14 reference images), speech-driven LipSync, CMX 3600 EDL timeline export, and an embedded Axum REST Web Studio.

---

## 🌟 Key Capabilities

1. **🎬 CinemaStudio Engine**:
   - **6 Cinema Camera Bodies**: Modular 8K Digital (ARRI LF Spec), Full-Frame Cine Digital (RED V-Raptor), Grand Format 70mm Film (IMAX Grand), Studio Digital S35, Classic 16mm Film, and Premium Large Format Digital (Alexa 65).
   - **11 Optical Prime Lenses**: Classic Anamorphic (2x squeeze, oval bokeh), Compact Anamorphic (horizontal streak flare), Creative Tilt Lens, 70s Cinema Prime, Extreme Macro, Premium Modern Prime, Warm Cinema Prime, Swirl Bokeh Portrait, Vintage Prime, Halation Diffusion (Pro-Mist), and Clinical Sharp Prime.
   - **3D Spatial Trajectories**: Pan, Tilt, Roll, Optical Zoom, Dolly, Truck, Crane/Pedestal, and 360° Orbit.
   - **Reactive Photographic Prompt Compiler**: Generates exact photorealistic prompts and camera metadata for Nano Banana and Flux cinema pipelines.

2. **🖼️ ImageStudio (T2I & I2I)**:
   - Text-to-Image and Image-to-Image with 51+ models (Nano Banana, Flux.1 Schnell/Dev, Ideogram, SDXL, Recraft V3, Midjourney styles).
   - **Multi-image conditioning**: Supports up to 14 concurrent reference images (`images_list`) for Nano Banana 2 Edit.
   - Aspect ratio, resolution, seed, and strength control.

3. **🎥 VideoStudio (T2V & I2V)**:
   - 42+ Text-to-Video and 61+ Image-to-Video models (Seedance Lite, Kling 1.5 Pro, Luma Dream Machine Ray, MiniMax / Hailuo, CogVideoX 5B).
   - Duration (5s, 10s), frame rates, aspect ratios, and resolution tiers (480p, 720p, 1080p).

4. **🎙️ LipSyncStudio**:
   - Audio-driven facial synchronization and speech animation across 9 models (Infinite Talk, MuseTalk, Hallo Portrait, SadTalker, LivePortrait).
   - High-precision viseme and phoneme alignment with expression weight control.

5. **🎞️ Storyboard Sequence Composer**:
   - Multi-scene film sequence sequencing combining establishing camera shots, dialogue, and high-speed motion.
   - Direct compilation to standard **CMX 3600 Edit Decision Lists (EDL)** for seamless import into DaVinci Resolve and Adobe Premiere Pro.

6. **⚡ Axum Web Studio & REST Server**:
   - Embedded web studio interface served directly at `http://127.0.0.1:3000/`.
   - Comprehensive REST API endpoints (`/api/models`, `/api/generate/image`, `/api/generate/cinema`, `/api/generate/video`, `/api/generate/lipsync`, `/api/storyboard/compile`).

---

## 🚀 Live Interactive Demo

Experience the interactive 3D camera visualizer and 220+ model catalog browser online:  
👉 **[https://code.brandonhubbard.com/open-higgsfield-ai-rs/](https://code.brandonhubbard.com/open-higgsfield-ai-rs/)**

---

## 📦 Installation

Add `open-higgsfield-ai` to your `Cargo.toml`:

```toml
[dependencies]
open-higgsfield-ai = "0.0.1"
tokio = { version = "1.39", features = ["full"] }
```

Or install the standalone CLI:

```bash
cargo install --path .
```

---

## 💻 CLI Usage

The `open-higgsfield-ai` CLI provides subcommands for every studio workflow:

```bash
# 1. Browse and search 220+ cataloged models
open-higgsfield-ai models --category t2v
open-higgsfield-ai models --search flux
open-higgsfield-ai models --id nano-banana

# 2. CinemaStudio: 3D Camera & Lens compilation
open-higgsfield-ai cinema \
  --prompt "Neon rain in Neo Tokyo alleyway" \
  --camera modular8k \
  --lens anamorphic \
  --focal-length 35 \
  --aperture "f/1.4" \
  --lighting noir \
  --pan 20 \
  --dry-run

# 3. Text-to-Image Generation
open-higgsfield-ai image \
  --prompt "Editorial high-fashion avant-garde portrait" \
  --model nano-banana \
  --aspect-ratio "1:1"

# 4. Text-to-Video Generation
open-higgsfield-ai video \
  --prompt "Formula race car overtaking in torrential rain" \
  --model seedance-lite-t2v \
  --duration 5 \
  --aspect-ratio "16:9"

# 5. LipSync Character Speech Animation
open-higgsfield-ai lipsync \
  --image "https://example.com/portrait.jpg" \
  --audio "https://example.com/dialogue.mp3" \
  --model infinitetalk-image-to-video

# 6. Launch Embedded Web Studio & REST Server
open-higgsfield-ai serve --host 127.0.0.1 --port 3000
```

---

## 🦀 Rust API Example

```rust
use open_higgsfield_ai::{
    HiggsfieldClient, CinemaStudio, CinemaStudioRequest,
    CinemaCamera, CinemaLens, CinemaLighting, CameraMotion3D,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize client from environment (OPEN_HIGGSFIELD_API_KEY or MUAPI_KEY)
    let client = HiggsfieldClient::from_env()?;
    let studio = CinemaStudio::new(&client);

    let req = CinemaStudioRequest {
        prompt: "Detective standing in rain drenched Neo Tokyo alley".into(),
        camera: Some(CinemaCamera::Modular8KDigital),
        lens: Some(CinemaLens::ClassicAnamorphic),
        focal_length_mm: Some(35),
        aperture: Some("f/1.4".into()),
        lighting: Some(CinemaLighting::CyberpunkNeon),
        motion: Some(CameraMotion3D {
            pan_deg: 20.0,
            tilt_deg: -5.0,
            roll_deg: 0.0,
            zoom_scale: 1.0,
            dolly_m: 1.5,
            truck_m: 0.0,
            crane_m: 0.0,
            orbit_deg: 30.0,
        }),
        aspect_ratio: Some("21:9".into()),
        resolution: Some("4K".into()),
        seed: Some(42),
    };

    let response = studio.generate(&req).await?;
    if let Some(url) = response.output_url() {
        println!("Generated cinema frame: {}", url);
    }

    Ok(())
}
```

---

## 🏗️ Architecture

```
open-higgsfield-ai-rs
├── src/
│   ├── lib.rs              # Crate public interface
│   ├── client.rs           # Async Muapi gateway client (submit, poll, upload)
│   ├── error.rs            # Strongly-typed errors
│   ├── models/             # 220+ generative models registry & schemas
│   ├── studio/
│   │   ├── cinema.rs       # 3D camera controls, lenses, and prompt compiler
│   │   ├── image.rs        # T2I / I2I and multi-image conditioning (14 refs)
│   │   ├── video.rs        # T2V / I2V temporal motion controls
│   │   └── lipsync.rs      # Audio-driven speech and face synchronization
│   ├── storyboard.rs       # Multi-shot timeline composer & CMX 3600 EDL
│   ├── server.rs           # Axum REST server & embedded Web Studio
│   └── bin/main.rs         # Standalone CLI binary
├── data/
│   └── models.json         # Complete 220+ model parameter definitions
├── docs/
│   └── index.html          # Interactive Web Studio (Swiss Design System)
└── tests/
    └── test_engine.rs      # Engine and compiler integration tests
```

---

## 📄 License

MIT License. Developed by [Brandon Hubbard](https://code.brandonhubbard.com).
