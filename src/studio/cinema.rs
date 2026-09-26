use crate::client::{DEFAULT_IMAGE_MAX_ATTEMPTS, GenerationResponse, HiggsfieldClient};
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CinemaCamera {
    Modular8KDigital,
    FullFrameCineDigital,
    GrandFormat70mmFilm,
    StudioDigitalS35,
    Classic16mmFilm,
    PremiumLargeFormatDigital,
}

impl CinemaCamera {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Modular8KDigital => "modular 8K digital cinema camera",
            Self::FullFrameCineDigital => "full-frame digital cinema camera",
            Self::GrandFormat70mmFilm => "grand format 70mm film camera",
            Self::StudioDigitalS35 => "Super 35 studio digital camera",
            Self::Classic16mmFilm => "classic 16mm film camera",
            Self::PremiumLargeFormatDigital => "premium large-format digital cinema camera",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Modular8KDigital => "Modular 8K Digital",
            Self::FullFrameCineDigital => "Full-Frame Cine Digital",
            Self::GrandFormat70mmFilm => "Grand Format 70mm Film",
            Self::StudioDigitalS35 => "Studio Digital S35",
            Self::Classic16mmFilm => "Classic 16mm Film",
            Self::PremiumLargeFormatDigital => "Premium Large Format Digital",
        }
    }

    pub fn all() -> &'static [CinemaCamera] {
        &[
            Self::Modular8KDigital,
            Self::FullFrameCineDigital,
            Self::GrandFormat70mmFilm,
            Self::StudioDigitalS35,
            Self::Classic16mmFilm,
            Self::PremiumLargeFormatDigital,
        ]
    }
}

impl std::str::FromStr for CinemaCamera {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().replace(['-', '_', ' '], "").as_str() {
            "modular8kdigital" | "modular8k" => Ok(Self::Modular8KDigital),
            "fullframecinedigital" | "fullframe" | "cine" => Ok(Self::FullFrameCineDigital),
            "grandformat70mmfilm" | "70mm" | "grandformat" => Ok(Self::GrandFormat70mmFilm),
            "studiodigitals35" | "s35" | "super35" => Ok(Self::StudioDigitalS35),
            "classic16mmfilm" | "16mm" => Ok(Self::Classic16mmFilm),
            "premiumlargeformatdigital" | "largeformat" => Ok(Self::PremiumLargeFormatDigital),
            _ => Err(Error::InvalidParameter(format!("Unknown cinema camera: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CinemaLens {
    CreativeTiltLens,
    CompactAnamorphic,
    ExtremeMacro,
    Vintage70sCinemaPrime,
    ClassicAnamorphic,
    PremiumModernPrime,
    WarmCinemaPrime,
    SwirlBokehPortrait,
    VintagePrime,
    HalationDiffusion,
    ClinicalSharpPrime,
}

impl CinemaLens {
    pub fn description(&self) -> &'static str {
        match self {
            Self::CreativeTiltLens => "creative tilt lens effect",
            Self::CompactAnamorphic => "compact anamorphic lens with horizontal flares",
            Self::ExtremeMacro => "extreme macro lens with microscopically fine detail",
            Self::Vintage70sCinemaPrime => "1970s vintage cinema prime lens with organic warmth",
            Self::ClassicAnamorphic => "classic anamorphic lens with 2x squeeze and oval bokeh",
            Self::PremiumModernPrime => "premium modern cinema prime lens with zero distortion",
            Self::WarmCinemaPrime => "warm-toned cinema prime lens with golden skin tones",
            Self::SwirlBokehPortrait => "petzval swirl bokeh portrait lens",
            Self::VintagePrime => "vintage uncoated prime lens with soft blooming highlights",
            Self::HalationDiffusion => "pro-mist halation diffusion filter",
            Self::ClinicalSharpPrime => "ultra-sharp clinical prime lens with micro-contrast",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::CreativeTiltLens => "Creative Tilt Lens",
            Self::CompactAnamorphic => "Compact Anamorphic",
            Self::ExtremeMacro => "Extreme Macro",
            Self::Vintage70sCinemaPrime => "70s Cinema Prime",
            Self::ClassicAnamorphic => "Classic Anamorphic",
            Self::PremiumModernPrime => "Premium Modern Prime",
            Self::WarmCinemaPrime => "Warm Cinema Prime",
            Self::SwirlBokehPortrait => "Swirl Bokeh Portrait",
            Self::VintagePrime => "Vintage Prime",
            Self::HalationDiffusion => "Halation Diffusion",
            Self::ClinicalSharpPrime => "Clinical Sharp Prime",
        }
    }

    pub fn all() -> &'static [CinemaLens] {
        &[
            Self::CreativeTiltLens,
            Self::CompactAnamorphic,
            Self::ExtremeMacro,
            Self::Vintage70sCinemaPrime,
            Self::ClassicAnamorphic,
            Self::PremiumModernPrime,
            Self::WarmCinemaPrime,
            Self::SwirlBokehPortrait,
            Self::VintagePrime,
            Self::HalationDiffusion,
            Self::ClinicalSharpPrime,
        ]
    }
}

impl std::str::FromStr for CinemaLens {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().replace(['-', '_', ' '], "").as_str() {
            "creativetiltlens" | "tilt" | "tiltshift" => Ok(Self::CreativeTiltLens),
            "compactanamorphic" => Ok(Self::CompactAnamorphic),
            "extrememacro" | "macro" => Ok(Self::ExtremeMacro),
            "70scinemaprime" | "vintage70s" => Ok(Self::Vintage70sCinemaPrime),
            "classicanamorphic" | "anamorphic" => Ok(Self::ClassicAnamorphic),
            "premiummodernprime" | "modernprime" => Ok(Self::PremiumModernPrime),
            "warmcinemaprime" | "warmprime" => Ok(Self::WarmCinemaPrime),
            "swirlbokehportrait" | "swirlbokeh" | "petzval" => Ok(Self::SwirlBokehPortrait),
            "vintageprime" => Ok(Self::VintagePrime),
            "halationdiffusion" | "promist" | "halation" => Ok(Self::HalationDiffusion),
            "clinicalsharpprime" | "sharp" => Ok(Self::ClinicalSharpPrime),
            _ => Err(Error::InvalidParameter(format!("Unknown cinema lens: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CinemaLighting {
    FilmNoir,
    GoldenHour,
    CyberpunkNeon,
    NaturalDaylight,
    RimLight,
    SoftStudio,
    VolumetricFog,
    MoodyDusk,
}

impl CinemaLighting {
    pub fn description(&self) -> &'static str {
        match self {
            Self::FilmNoir => "dramatic film noir chiascuro with sharp Venetian blind shadows",
            Self::GoldenHour => "radiant golden hour sunlight with warm amber backlighting",
            Self::CyberpunkNeon => "vibrant cyberpunk neon illumination with teal and magenta rim glow",
            Self::NaturalDaylight => "clean diffused natural daylight with neutral balance",
            Self::RimLight => "dramatic contre-jour edge lighting separating subject from deep background",
            Self::SoftStudio => "luxurious diffused studio softbox portrait lighting with subtle catchlights",
            Self::VolumetricFog => "atmospheric volumetric god rays cutting through cinematic haze",
            Self::MoodyDusk => "deep twilight blue hour lighting with subtle tungsten accents",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::FilmNoir => "Film Noir",
            Self::GoldenHour => "Golden Hour",
            Self::CyberpunkNeon => "Cyberpunk Neon",
            Self::NaturalDaylight => "Natural Daylight",
            Self::RimLight => "Rim / Contre-Jour",
            Self::SoftStudio => "Soft Studio Diffused",
            Self::VolumetricFog => "Volumetric God Rays",
            Self::MoodyDusk => "Moody Dusk / Blue Hour",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraMotion3D {
    /// Horizontal rotation in degrees (-180 to 180)
    pub pan_deg: f32,
    /// Vertical rotation in degrees (-90 to 90)
    pub tilt_deg: f32,
    /// Roll angle around optical axis (-180 to 180)
    pub roll_deg: f32,
    /// Focal zoom factor (e.g. 1.0 = neutral, 2.0 = 2x zoom)
    pub zoom_scale: f32,
    /// Dolly displacement in meters (positive = forward/in, negative = backward/out)
    pub dolly_m: f32,
    /// Truck displacement in meters (positive = right, negative = left)
    pub truck_m: f32,
    /// Crane / pedestal displacement in meters (positive = up, negative = down)
    pub crane_m: f32,
    /// Circular orbit in degrees around subject
    pub orbit_deg: f32,
}

impl Default for CameraMotion3D {
    fn default() -> Self {
        Self {
            pan_deg: 0.0,
            tilt_deg: 0.0,
            roll_deg: 0.0,
            zoom_scale: 1.0,
            dolly_m: 0.0,
            truck_m: 0.0,
            crane_m: 0.0,
            orbit_deg: 0.0,
        }
    }
}

impl CameraMotion3D {
    pub fn description(&self) -> String {
        let mut parts = Vec::new();
        if self.pan_deg.abs() > 1.0 {
            parts.push(format!("slow pan {}°", if self.pan_deg > 0.0 { "right" } else { "left" }));
        }
        if self.tilt_deg.abs() > 1.0 {
            parts.push(format!("smooth tilt {}", if self.tilt_deg > 0.0 { "up" } else { "down" }));
        }
        if self.roll_deg.abs() > 1.0 {
            parts.push(format!("dramatic roll {}°", self.roll_deg));
        }
        if (self.zoom_scale - 1.0).abs() > 0.05 {
            parts.push(format!("optical zoom {:.1}x", self.zoom_scale));
        }
        if self.dolly_m.abs() > 0.1 {
            parts.push(format!("smooth dolly {}", if self.dolly_m > 0.0 { "in" } else { "out" }));
        }
        if self.truck_m.abs() > 0.1 {
            parts.push(format!("tracking truck {}", if self.truck_m > 0.0 { "right" } else { "left" }));
        }
        if self.crane_m.abs() > 0.1 {
            parts.push(format!("crane shot {}", if self.crane_m > 0.0 { "ascending" } else { "descending" }));
        }
        if self.orbit_deg.abs() > 5.0 {
            parts.push(format!("360 orbit camera rotation {:.0}°", self.orbit_deg));
        }

        if parts.is_empty() {
            "locked-off cinematic tripod shot".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CinemaStudioRequest {
    pub prompt: String,
    pub camera: Option<CinemaCamera>,
    pub lens: Option<CinemaLens>,
    pub focal_length_mm: Option<u32>,
    pub aperture: Option<String>,
    pub lighting: Option<CinemaLighting>,
    pub motion: Option<CameraMotion3D>,
    pub aspect_ratio: Option<String>,
    pub resolution: Option<String>,
    pub seed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledCinemaPrompt {
    pub full_prompt: String,
    pub camera_spec: String,
    pub lens_spec: String,
    pub perspective_spec: String,
    pub aperture_spec: String,
    pub lighting_spec: String,
    pub motion_spec: String,
}

pub fn compile_cinema_prompt(req: &CinemaStudioRequest) -> CompiledCinemaPrompt {
    let camera = req.camera.unwrap_or(CinemaCamera::Modular8KDigital);
    let lens = req.lens.unwrap_or(CinemaLens::ClassicAnamorphic);
    let focal = req.focal_length_mm.unwrap_or(35);
    let aperture = req.aperture.as_deref().unwrap_or("f/1.4");
    let lighting = req.lighting.unwrap_or(CinemaLighting::FilmNoir);
    let motion = req.motion.clone().unwrap_or_default();

    let camera_desc = camera.description();
    let lens_desc = lens.description();

    let perspective = match focal {
        f if f <= 10 => "ultra-wide fish-eye perspective",
        f if f <= 18 => "wide-angle expansive perspective",
        f if f <= 28 => "wide-angle dynamic environmental perspective",
        f if f <= 40 => "natural cinematic human-eye perspective",
        f if f <= 60 => "standard portrait perspective",
        f if f <= 90 => "classic portrait telephoto perspective",
        _ => "compressed telephoto perspective with dramatic background isolation",
    };

    let depth_effect = match aperture {
        "f/1.2" | "f/1.4" => "ultra-shallow depth of field, creamy creamy bokeh",
        "f/2" | "f/2.8" => "shallow depth of field, soft circular bokeh highlights",
        "f/4" | "f/5.6" => "balanced depth of field with crisp foreground sharpness",
        _ => "deep focus clarity, pin-sharp foreground to background infinite depth",
    };

    let lighting_desc = lighting.description();
    let motion_desc = motion.description();

    let parts = vec![
        req.prompt.trim().to_string(),
        format!("shot on a {camera_desc}"),
        format!("using a {lens_desc} at {focal}mm ({perspective})"),
        format!("aperture {aperture}, {depth_effect}"),
        lighting_desc.to_string(),
        format!("camera trajectory: {motion_desc}"),
        "natural cinematic color grading".to_string(),
        "35mm film grain texture".to_string(),
        "high dynamic range".to_string(),
        "ultra-detailed 8K resolution".to_string(),
        "masterpiece cinema still".to_string(),
    ];

    let full_prompt = parts.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ");

    CompiledCinemaPrompt {
        full_prompt,
        camera_spec: camera_desc.to_string(),
        lens_spec: lens_desc.to_string(),
        perspective_spec: perspective.to_string(),
        aperture_spec: format!("{aperture} ({depth_effect})"),
        lighting_spec: lighting_desc.to_string(),
        motion_spec: motion_desc,
    }
}

pub struct CinemaStudio<'a> {
    client: &'a HiggsfieldClient,
}

impl<'a> CinemaStudio<'a> {
    pub fn new(client: &'a HiggsfieldClient) -> Self {
        Self { client }
    }

    /// Compile cinema prompt and submit generation to the Nano Banana or Flux endpoint.
    pub async fn generate(&self, req: &CinemaStudioRequest) -> Result<GenerationResponse> {
        let compiled = compile_cinema_prompt(req);

        let mut payload = serde_json::json!({
            "prompt": compiled.full_prompt,
            "aspect_ratio": req.aspect_ratio.as_deref().unwrap_or("16:9"),
        });

        if let Some(res) = &req.resolution {
            payload["resolution"] = serde_json::Value::String(res.clone());
        }
        if let Some(seed) = req.seed {
            if seed >= 0 {
                payload["seed"] = serde_json::Value::Number(seed.into());
            }
        }

        self.client
            .submit_and_poll("nano-banana", &payload, DEFAULT_IMAGE_MAX_ATTEMPTS)
            .await
    }
}
