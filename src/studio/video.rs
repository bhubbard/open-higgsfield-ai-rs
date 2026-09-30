use crate::client::{DEFAULT_VIDEO_MAX_ATTEMPTS, GenerationResponse, HiggsfieldClient};
use crate::error::Result;
use crate::models::ModelRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStudioRequest {
    pub model: String,
    pub prompt: Option<String>,
    pub aspect_ratio: Option<String>,
    pub duration: Option<u32>,
    pub resolution: Option<String>,
    pub quality: Option<String>,
    pub mode: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug)]
pub struct VideoStudio<'a> {
    client: &'a HiggsfieldClient,
    registry: &'static ModelRegistry,
}

impl<'a> VideoStudio<'a> {
    pub fn new(client: &'a HiggsfieldClient) -> Self {
        Self {
            client,
            registry: ModelRegistry::global(),
        }
    }

    pub async fn generate(&self, req: &VideoStudioRequest) -> Result<GenerationResponse> {
        let model_info = self.registry.get(&req.model);
        let endpoint = model_info
            .map(|m| m.resolved_endpoint())
            .unwrap_or(&req.model);

        let mut payload = serde_json::Map::new();

        if let Some(ref p) = req.prompt {
            payload.insert("prompt".to_string(), serde_json::Value::String(p.clone()));
        }
        if let Some(ref ar) = req.aspect_ratio {
            payload.insert("aspect_ratio".to_string(), serde_json::Value::String(ar.clone()));
        }
        if let Some(dur) = req.duration {
            payload.insert("duration".to_string(), serde_json::Value::Number(dur.into()));
        }
        if let Some(ref res) = req.resolution {
            payload.insert("resolution".to_string(), serde_json::Value::String(res.clone()));
        }
        if let Some(ref q) = req.quality {
            payload.insert("quality".to_string(), serde_json::Value::String(q.clone()));
        }
        if let Some(ref m) = req.mode {
            payload.insert("mode".to_string(), serde_json::Value::String(m.clone()));
        }

        if let Some(ref img) = req.image_url {
            let image_field = model_info
                .and_then(|m| m.image_field.as_deref())
                .unwrap_or("image_url");

            if image_field == "images_list" {
                payload.insert("images_list".to_string(), serde_json::json!([img]));
            } else {
                payload.insert(image_field.to_string(), serde_json::Value::String(img.clone()));
            }
        }

        let body = serde_json::Value::Object(payload);
        self.client
            .submit_and_poll(endpoint, &body, DEFAULT_VIDEO_MAX_ATTEMPTS)
            .await
    }
}
