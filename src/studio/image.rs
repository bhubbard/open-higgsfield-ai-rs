use crate::client::{DEFAULT_IMAGE_MAX_ATTEMPTS, GenerationResponse, HiggsfieldClient};
use crate::error::Result;
use crate::models::ModelRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStudioRequest {
    pub model: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub aspect_ratio: Option<String>,
    pub resolution: Option<String>,
    pub quality: Option<String>,
    pub seed: Option<i64>,
    pub strength: Option<f32>,
    /// Single image conditioning URL
    pub image_url: Option<String>,
    /// Multi-image conditioning references (up to 14 images)
    pub images_list: Option<Vec<String>>,
}

pub struct ImageStudio<'a> {
    client: &'a HiggsfieldClient,
    registry: &'static ModelRegistry,
}

impl<'a> ImageStudio<'a> {
    pub fn new(client: &'a HiggsfieldClient) -> Self {
        Self {
            client,
            registry: ModelRegistry::global(),
        }
    }

    pub async fn generate(&self, req: &ImageStudioRequest) -> Result<GenerationResponse> {
        let model_info = self.registry.get(&req.model);
        let endpoint = model_info
            .map(|m| m.resolved_endpoint())
            .unwrap_or(&req.model);

        let mut payload = serde_json::json!({
            "prompt": req.prompt,
        });

        if let Some(ref ar) = req.aspect_ratio {
            payload["aspect_ratio"] = serde_json::Value::String(ar.clone());
        }
        if let Some(ref res) = req.resolution {
            payload["resolution"] = serde_json::Value::String(res.clone());
        }
        if let Some(ref q) = req.quality {
            payload["quality"] = serde_json::Value::String(q.clone());
        }
        if let Some(seed) = req.seed {
            if seed >= 0 {
                payload["seed"] = serde_json::Value::Number(seed.into());
            }
        }

        // Multi-image conditioning handling
        let image_field = model_info
            .and_then(|m| m.image_field.as_deref())
            .unwrap_or("image_url");

        let mut all_images = Vec::new();
        if let Some(ref list) = req.images_list {
            all_images.extend(list.iter().take(14).cloned());
        } else if let Some(ref single) = req.image_url {
            all_images.push(single.clone());
        }

        if !all_images.is_empty() {
            if image_field == "images_list" {
                payload["images_list"] = serde_json::to_value(&all_images)?;
            } else {
                payload[image_field] = serde_json::Value::String(all_images[0].clone());
            }
            payload["strength"] = serde_json::json!(req.strength.unwrap_or(0.6));
        }

        self.client
            .submit_and_poll(endpoint, &payload, DEFAULT_IMAGE_MAX_ATTEMPTS)
            .await
    }
}
