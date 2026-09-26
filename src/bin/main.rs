use clap::{Parser, Subcommand};
use open_higgsfield_ai::{
    client::HiggsfieldClient,
    models::{ModelModality, ModelRegistry},
    server::run_server,
    storyboard::{ShotType, Storyboard, StoryboardShot},
    studio::{
        compile_cinema_prompt, CameraMotion3D, CinemaCamera, CinemaLens, CinemaLighting,
        CinemaStudio, CinemaStudioRequest, ImageStudio, ImageStudioRequest, LipSyncStudio,
        LipSyncStudioRequest, VideoStudio, VideoStudioRequest,
    },
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "open-higgsfield-ai",
    author = "Brandon Hubbard",
    version = env!("CARGO_PKG_VERSION"),
    about = "Native Rust engine and studio client for Open-Higgsfield-AI: 220+ image, video, lipsync, and cinema camera models with Axum web studio and CLI.",
    long_about = None
)]
struct Cli {
    #[arg(long, env = "OPEN_HIGGSFIELD_API_KEY", global = true)]
    api_key: Option<String>,

    #[arg(long, env = "OPEN_HIGGSFIELD_BASE_URL", global = true)]
    base_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Browse, filter, and inspect the 220+ generative models
    Models {
        /// Filter by modality: t2i, t2v, i2i, i2v, lipsync
        #[arg(short, long)]
        category: Option<String>,

        /// Search query (matches ID, name, or description)
        #[arg(short, long)]
        search: Option<String>,

        /// Show complete input schema and details for a specific model ID
        #[arg(short, long)]
        id: Option<String>,
    },

    /// CinemaStudio: 3D camera controls, anamorphic lenses, and lighting compilation
    Cinema {
        /// Creative prompt / scene description
        #[arg(short, long)]
        prompt: String,

        /// Camera body: modular8k, fullframe, 70mm, s35, 16mm, largeformat
        #[arg(short, long, default_value = "modular8k")]
        camera: String,

        /// Lens: tilt, anamorphic, macro, vintage70s, modernprime, warmprime, swirlbokeh, vintageprime, promist, sharp
        #[arg(short, long, default_value = "anamorphic")]
        lens: String,

        /// Focal length in mm (e.g. 14, 24, 35, 50, 85, 100)
        #[arg(short, long, default_value_t = 35)]
        focal_length: u32,

        /// Lens aperture (e.g. f/1.4, f/2.8, f/4, f/11)
        #[arg(short, long, default_value = "f/1.4")]
        aperture: String,

        /// Lighting preset: noir, goldenhour, cyberpunk, daylight, rim, softbox, fog, dusk
        #[arg(long, default_value = "goldenhour")]
        lighting: String,

        /// Camera pan in degrees (-180 to 180)
        #[arg(long, default_value_t = 0.0)]
        pan: f32,

        /// Camera tilt in degrees (-90 to 90)
        #[arg(long, default_value_t = 0.0)]
        tilt: f32,

        /// Camera roll in degrees (-180 to 180)
        #[arg(long, default_value_t = 0.0)]
        roll: f32,

        /// Optical zoom factor (e.g. 1.0, 1.5, 2.0)
        #[arg(long, default_value_t = 1.0)]
        zoom: f32,

        /// Dolly in/out in meters (+ forward, - backward)
        #[arg(long, default_value_t = 0.0)]
        dolly: f32,

        /// Orbit in degrees (0 to 360)
        #[arg(long, default_value_t = 0.0)]
        orbit: f32,

        /// Aspect ratio (16:9, 21:9, 9:16, 1:1)
        #[arg(long, default_value = "16:9")]
        aspect_ratio: String,

        /// Dry run: compile and print photographic prompt and camera parameters without submitting
        #[arg(long)]
        dry_run: bool,
    },

    /// ImageStudio: Text-to-Image (T2I) & Image-to-Image (I2I) generation
    Image {
        /// Prompt describing the image
        #[arg(short, long)]
        prompt: String,

        /// Model ID (default: nano-banana)
        #[arg(short, long, default_value = "nano-banana")]
        model: String,

        /// Aspect ratio (1:1, 16:9, 9:16, 4:3, etc.)
        #[arg(short, long, default_value = "1:1")]
        aspect_ratio: String,

        /// Reference image URL for image-to-image conditioning
        #[arg(long)]
        image: Option<String>,

        /// Conditioning strength (0.0 to 1.0, default 0.6)
        #[arg(long, default_value_t = 0.6)]
        strength: f32,
    },

    /// VideoStudio: Text-to-Video (T2V) & Image-to-Video (I2V) generation
    Video {
        /// Motion prompt describing the action
        #[arg(short, long)]
        prompt: String,

        /// Video Model ID (e.g. seedance-lite-t2v, kling, minimax)
        #[arg(short, long, default_value = "seedance-lite-t2v")]
        model: String,

        /// Duration in seconds (5 or 10)
        #[arg(short, long, default_value_t = 5)]
        duration: u32,

        /// Aspect ratio (16:9, 9:16, 1:1, etc.)
        #[arg(long, default_value = "16:9")]
        aspect_ratio: String,

        /// Source image URL for Image-to-Video animation
        #[arg(long)]
        image: Option<String>,
    },

    /// LipSyncStudio: Audio-driven character speech and facial animation
    Lipsync {
        /// Audio file URL or hosted track
        #[arg(short, long)]
        audio: String,

        /// Portrait image URL
        #[arg(short, long)]
        image: Option<String>,

        /// Portrait video URL
        #[arg(short, long)]
        video: Option<String>,

        /// LipSync Model ID (default: infinitetalk-image-to-video)
        #[arg(short, long, default_value = "infinitetalk-image-to-video")]
        model: String,
    },

    /// Storyboard sequence composer: compile EDL or render multi-shot timelines
    Storyboard {
        /// Path to storyboard JSON file
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Export CMX 3600 Edit Decision List (EDL) for DaVinci Resolve / Premiere
        #[arg(long)]
        export_edl: Option<PathBuf>,

        /// Render all shots sequentially
        #[arg(long)]
        render: bool,
    },

    /// Launch embedded Axum web studio and REST API server
    Serve {
        /// Bind host address
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Bind port
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let registry = ModelRegistry::global();

    let build_client = || -> Result<HiggsfieldClient, anyhow::Error> {
        if let Some(ref key) = cli.api_key {
            let mut c = HiggsfieldClient::new(key.clone());
            if let Some(ref base) = cli.base_url {
                c.set_base_url(base.clone());
            }
            Ok(c)
        } else {
            HiggsfieldClient::from_env().map_err(|e| anyhow::anyhow!(e))
        }
    };

    match cli.command {
        Commands::Models { category, search, id } => {
            if let Some(model_id) = id {
                if let Some(m) = registry.get(&model_id) {
                    println!("\n📦 Model: {} ({})", m.name, m.id);
                    println!("Modality:     {}", m.modality);
                    println!("Endpoint:     {}", m.resolved_endpoint());
                    if let Some(ref d) = m.description {
                        println!("Description:  {}", d);
                    }
                    if let Some(ref f) = m.family {
                        println!("Family:       {}", f);
                    }
                    println!("Aspect Ratios: {:?}", m.aspect_ratios());
                    println!("Durations:    {:?}", m.durations());
                    println!("Resolutions:  {:?}", m.resolutions());
                    println!("\nInputs Schema:\n{}", serde_json::to_string_pretty(&m.inputs)?);
                } else {
                    eprintln!("❌ Model '{}' not found in registry (total models: {})", model_id, registry.total_count());
                }
                return Ok(());
            }

            let models = if let Some(q) = search {
                println!("🔍 Searching models matching '{}'...", q);
                registry.search(&q)
            } else if let Some(cat) = category {
                let modality = match cat.to_lowercase().as_str() {
                    "t2i" => ModelModality::T2i,
                    "t2v" => ModelModality::T2v,
                    "i2i" => ModelModality::I2i,
                    "i2v" => ModelModality::I2v,
                    "v2v" => ModelModality::V2v,
                    "lipsync" => ModelModality::LipSync,
                    _ => anyhow::bail!("Unknown modality '{}'. Use: t2i, t2v, i2i, i2v, lipsync", cat),
                };
                println!("📂 Listing {} models...", modality);
                registry.by_modality(modality)
            } else {
                println!("📚 All available models ({} total):", registry.total_count());
                registry.all().iter().collect()
            };

            println!("{:<32} {:<24} {:<12}", "MODEL ID", "NAME", "MODALITY");
            println!("{}", "-".repeat(70));
            for m in models {
                println!("{:<32} {:<24} {:<12}", m.id, m.name, m.modality.to_string());
            }
        }

        Commands::Cinema {
            prompt,
            camera,
            lens,
            focal_length,
            aperture,
            lighting,
            pan,
            tilt,
            roll,
            zoom,
            dolly,
            orbit,
            aspect_ratio,
            dry_run,
        } => {
            let cam_enum: CinemaCamera = camera.parse()?;
            let lens_enum: CinemaLens = lens.parse()?;
            let light_enum = match lighting.to_lowercase().as_str() {
                "noir" => CinemaLighting::FilmNoir,
                "goldenhour" | "golden" => CinemaLighting::GoldenHour,
                "cyberpunk" | "neon" => CinemaLighting::CyberpunkNeon,
                "daylight" => CinemaLighting::NaturalDaylight,
                "rim" => CinemaLighting::RimLight,
                "softbox" | "soft" => CinemaLighting::SoftStudio,
                "fog" | "godrays" => CinemaLighting::VolumetricFog,
                "dusk" | "twilight" => CinemaLighting::MoodyDusk,
                _ => CinemaLighting::GoldenHour,
            };

            let motion = CameraMotion3D {
                pan_deg: pan,
                tilt_deg: tilt,
                roll_deg: roll,
                zoom_scale: zoom,
                dolly_m: dolly,
                truck_m: 0.0,
                crane_m: 0.0,
                orbit_deg: orbit,
            };

            let req = CinemaStudioRequest {
                prompt,
                camera: Some(cam_enum),
                lens: Some(lens_enum),
                focal_length_mm: Some(focal_length),
                aperture: Some(aperture),
                lighting: Some(light_enum),
                motion: Some(motion),
                aspect_ratio: Some(aspect_ratio),
                resolution: Some("1080p".to_string()),
                seed: None,
            };

            let compiled = compile_cinema_prompt(&req);

            println!("\n🎬 CinemaStudio Prompt Compilation:");
            println!("------------------------------------------------------------");
            println!("Camera:      {}", compiled.camera_spec);
            println!("Lens:        {}", compiled.lens_spec);
            println!("Perspective: {}", compiled.perspective_spec);
            println!("Aperture:    {}", compiled.aperture_spec);
            println!("Lighting:    {}", compiled.lighting_spec);
            println!("Motion:      {}", compiled.motion_spec);
            println!("\n✨ Compiled Photographic Prompt:\n{}", compiled.full_prompt);

            if dry_run {
                println!("\n[Dry Run] Request compiled without submitting to API.");
                return Ok(());
            }

            let client = build_client()?;
            let studio = CinemaStudio::new(&client);
            println!("\n🚀 Submitting generation to Nano Banana cinema pipeline...");
            let res = studio.generate(&req).await?;
            if let Some(url) = res.output_url() {
                println!("✅ Generated Cinema Output: {}", url);
            } else {
                println!("Job submitted: {:?}", res);
            }
        }

        Commands::Image {
            prompt,
            model,
            aspect_ratio,
            image,
            strength,
        } => {
            let client = build_client()?;
            let studio = ImageStudio::new(&client);
            let req = ImageStudioRequest {
                model,
                prompt,
                negative_prompt: None,
                aspect_ratio: Some(aspect_ratio),
                resolution: None,
                quality: None,
                seed: None,
                strength: Some(strength),
                image_url: image,
                images_list: None,
            };

            println!("🚀 Generating image with {}...", req.model);
            let res = studio.generate(&req).await?;
            if let Some(url) = res.output_url() {
                println!("✅ Generated Image: {}", url);
            } else {
                println!("Generation response: {:?}", res);
            }
        }

        Commands::Video {
            prompt,
            model,
            duration,
            aspect_ratio,
            image,
        } => {
            let client = build_client()?;
            let studio = VideoStudio::new(&client);
            let req = VideoStudioRequest {
                model,
                prompt: Some(prompt),
                aspect_ratio: Some(aspect_ratio),
                duration: Some(duration),
                resolution: Some("720p".to_string()),
                quality: None,
                mode: None,
                image_url: image,
            };

            println!("🚀 Generating video with {} ({}s)...", req.model, duration);
            let res = studio.generate(&req).await?;
            if let Some(url) = res.output_url() {
                println!("✅ Generated Video: {}", url);
            } else {
                println!("Generation response: {:?}", res);
            }
        }

        Commands::Lipsync {
            audio,
            image,
            video,
            model,
        } => {
            let client = build_client()?;
            let studio = LipSyncStudio::new(&client);
            let req = LipSyncStudioRequest {
                model,
                audio_url: audio,
                image_url: image,
                video_url: video,
                prompt: None,
                resolution: Some("720p".to_string()),
                seed: None,
            };

            println!("🚀 Generating LipSync animation with {}...", req.model);
            let res = studio.generate(&req).await?;
            if let Some(url) = res.output_url() {
                println!("✅ Generated LipSync Video: {}", url);
            } else {
                println!("Generation response: {:?}", res);
            }
        }

        Commands::Storyboard {
            file,
            export_edl,
            render,
        } => {
            let mut storyboard = if let Some(ref path) = file {
                let content = std::fs::read_to_string(path)?;
                serde_json::from_str::<Storyboard>(&content)?
            } else {
                let mut sb = Storyboard::new("Demo Cinematic Sequence");
                sb.add_shot(StoryboardShot {
                    shot_number: 1,
                    title: "Wide Establishing Cyberpunk Vista".to_string(),
                    shot_type: ShotType::Cinema,
                    duration_seconds: 4.0,
                    prompt: "Neon drenched rain streets of Tokyo 2088".to_string(),
                    cinema_spec: None,
                    model: Some("nano-banana".to_string()),
                    audio_url: None,
                    image_url: None,
                    result_url: None,
                });
                sb.add_shot(StoryboardShot {
                    shot_number: 2,
                    title: "Medium Tracking Shot - Cybernetic Detective".to_string(),
                    shot_type: ShotType::Video,
                    duration_seconds: 5.0,
                    prompt: "Cybernetic detective walking through rainy alleyway".to_string(),
                    cinema_spec: None,
                    model: Some("seedance-lite-t2v".to_string()),
                    audio_url: None,
                    image_url: None,
                    result_url: None,
                });
                sb
            };

            println!("\n🎬 Storyboard: {}", storyboard.title);
            println!("Shots count: {}", storyboard.shots.len());
            println!("Total duration: {:.1}s", storyboard.total_duration());

            if let Some(edl_path) = export_edl {
                let edl = storyboard.to_cmx3600_edl();
                std::fs::write(&edl_path, edl)?;
                println!("💾 Exported CMX 3600 EDL to: {}", edl_path.display());
            }

            if render {
                let client = build_client()?;
                println!("🚀 Rendering all shots sequentially...");
                let _results = storyboard.render_all(&client).await?;
                println!("✅ All shots rendered successfully!");
            }
        }

        Commands::Serve { host, port } => {
            let client = build_client().ok();
            run_server(&host, port, client).await?;
        }
    }

    Ok(())
}
