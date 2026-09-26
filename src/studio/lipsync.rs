use crate::client::{DEFAULT_VIDEO_MAX_ATTEMPTS, GenerationResponse, HiggsfieldClient};
use crate::error::Result;
use crate::models::ModelRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LipSyncStudioRequest {
    pub model: String,
    pub audio_url: String,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub prompt: Option<String>,
    pub resolution: Option<String>,
    pub seed: Option<i64>,
}

pub struct LipSyncStudio<'a> {
    client: &'a HiggsfieldClient,
    registry: &'static ModelRegistry,
}

impl<'a> LipSyncStudio<'a> {
    pub fn new(client: &'a HiggsfieldClient) -> Self {
        Self {
            client,
            registry: ModelRegistry::global(),
        }
    }

    pub async fn generate(&self, req: &LipSyncStudioRequest) -> Result<GenerationResponse> {
        let model_info = self.registry.get(&req.model);
        let endpoint = model_info
            .map(|m| m.resolved_endpoint())
            .unwrap_or(&req.model);

        let mut payload = serde_json::json!({
            "audio_url": req.audio_url,
        });

        if let Some(ref img) = req.image_url {
            payload["image_url"] = serde_json::Value::String(img.clone());
        }
        if let Some(ref vid) = req.video_url {
            payload["video_url"] = serde_json::Value::String(vid.clone());
        }
        if let Some(ref p) = req.prompt {
            payload["prompt"] = serde_json::Value::String(p.clone());
        }
        if let Some(ref res) = req.resolution {
            payload["resolution"] = serde_json::Value::String(res.clone());
        }
        if let Some(seed) = req.seed {
            if seed >= 0 {
                payload["seed"] = serde_json::Value::Number(seed.into());
            }
        }

        self.client
            .submit_and_poll(endpoint, &payload, DEFAULT_VIDEO_MAX_ATTEMPTS)
            .await
    }
}
