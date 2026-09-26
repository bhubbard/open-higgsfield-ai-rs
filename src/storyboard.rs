use crate::client::{GenerationResponse, HiggsfieldClient};
use crate::error::Result;
use crate::studio::{
    CinemaStudioRequest, ImageStudio, ImageStudioRequest, LipSyncStudio, LipSyncStudioRequest,
    VideoStudio, VideoStudioRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShotType {
    Cinema,
    Image,
    Video,
    LipSync,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryboardShot {
    pub shot_number: usize,
    pub title: String,
    pub shot_type: ShotType,
    pub duration_seconds: f32,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cinema_spec: Option<CinemaStudioRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storyboard {
    pub title: String,
    pub aspect_ratio: String,
    pub fps: u32,
    pub shots: Vec<StoryboardShot>,
}

impl Default for Storyboard {
    fn default() -> Self {
        Self {
            title: "Untitled Production".to_string(),
            aspect_ratio: "16:9".to_string(),
            fps: 24,
            shots: Vec::new(),
        }
    }
}

impl Storyboard {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            aspect_ratio: "16:9".to_string(),
            fps: 24,
            shots: Vec::new(),
        }
    }

    pub fn add_shot(&mut self, mut shot: StoryboardShot) {
        shot.shot_number = self.shots.len() + 1;
        self.shots.push(shot);
    }

    pub fn total_duration(&self) -> f32 {
        self.shots.iter().map(|s| s.duration_seconds).sum()
    }

    /// Compile a standard CMX 3600 Edit Decision List (EDL) for NLEs (DaVinci Resolve / Premiere Pro).
    pub fn to_cmx3600_edl(&self) -> String {
        let mut edl = String::new();
        edl.push_str(&format!("TITLE: {}\n", self.title));
        edl.push_str("FCM: NON-DROP FRAME\n\n");

        let mut timeline_frames: u64 = 0;

        for (i, shot) in self.shots.iter().enumerate() {
            let duration_frames = (shot.duration_seconds * (self.fps as f32)).round() as u64;
            let src_in = "00:00:00:00";
            let src_out = frames_to_timecode(duration_frames, self.fps);

            let rec_in = frames_to_timecode(timeline_frames, self.fps);
            let rec_out = frames_to_timecode(timeline_frames + duration_frames, self.fps);

            edl.push_str(&format!(
                "{:03}  AX       V     C        {} {} {} {}\n",
                i + 1,
                src_in,
                src_out,
                rec_in,
                rec_out
            ));
            edl.push_str(&format!("* FROM CLIP NAME: SHOT_{:02}_{}\n", shot.shot_number, shot.title.replace(' ', "_")));
            edl.push_str(&format!("* PROMPT: {}\n\n", shot.prompt));

            timeline_frames += duration_frames;
        }

        edl
    }

    /// Render all shots sequentially through the HiggsfieldClient.
    pub async fn render_all(&mut self, client: &HiggsfieldClient) -> Result<Vec<GenerationResponse>> {
        let mut results = Vec::new();

        for shot in &mut self.shots {
            let resp = match shot.shot_type {
                ShotType::Cinema => {
                    let req = shot.cinema_spec.clone().unwrap_or_else(|| CinemaStudioRequest {
                        prompt: shot.prompt.clone(),
                        camera: None,
                        lens: None,
                        focal_length_mm: Some(35),
                        aperture: Some("f/1.4".to_string()),
                        lighting: None,
                        motion: None,
                        aspect_ratio: Some(self.aspect_ratio.clone()),
                        resolution: Some("1080p".to_string()),
                        seed: None,
                    });
                    let studio = crate::studio::CinemaStudio::new(client);
                    studio.generate(&req).await?
                }
                ShotType::Image => {
                    let studio = ImageStudio::new(client);
                    let req = ImageStudioRequest {
                        model: shot.model.clone().unwrap_or_else(|| "nano-banana".to_string()),
                        prompt: shot.prompt.clone(),
                        negative_prompt: None,
                        aspect_ratio: Some(self.aspect_ratio.clone()),
                        resolution: None,
                        quality: None,
                        seed: None,
                        strength: None,
                        image_url: shot.image_url.clone(),
                        images_list: None,
                    };
                    studio.generate(&req).await?
                }
                ShotType::Video => {
                    let studio = VideoStudio::new(client);
                    let req = VideoStudioRequest {
                        model: shot.model.clone().unwrap_or_else(|| "seedance-lite-t2v".to_string()),
                        prompt: Some(shot.prompt.clone()),
                        aspect_ratio: Some(self.aspect_ratio.clone()),
                        duration: Some(shot.duration_seconds.round() as u32),
                        resolution: Some("720p".to_string()),
                        quality: None,
                        mode: None,
                        image_url: shot.image_url.clone(),
                    };
                    studio.generate(&req).await?
                }
                ShotType::LipSync => {
                    let studio = LipSyncStudio::new(client);
                    let req = LipSyncStudioRequest {
                        model: shot.model.clone().unwrap_or_else(|| "infinitetalk-image-to-video".to_string()),
                        audio_url: shot.audio_url.clone().unwrap_or_default(),
                        image_url: shot.image_url.clone(),
                        video_url: None,
                        prompt: Some(shot.prompt.clone()),
                        resolution: Some("720p".to_string()),
                        seed: None,
                    };
                    studio.generate(&req).await?
                }
            };

            if let Some(url) = resp.output_url() {
                shot.result_url = Some(url.to_string());
            }
            results.push(resp);
        }

        Ok(results)
    }
}

fn frames_to_timecode(frames: u64, fps: u32) -> String {
    let f = frames % (fps as u64);
    let total_secs = frames / (fps as u64);
    let s = total_secs % 60;
    let total_mins = total_secs / 60;
    let m = total_mins % 60;
    let h = total_mins / 60;
    format!("{:02}:{:02}:{:02}:{:02}", h, m, s, f)
}
