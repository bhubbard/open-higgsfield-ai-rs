use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

const MODELS_JSON: &str = include_str!("../../data/models.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelModality {
    T2i,
    T2v,
    I2i,
    I2v,
    V2v,
    LipSync,
    ImageLipSync,
    VideoLipSync,
}

impl std::fmt::Display for ModelModality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::T2i => write!(f, "Text-to-Image (T2I)"),
            Self::T2v => write!(f, "Text-to-Video (T2V)"),
            Self::I2i => write!(f, "Image-to-Image (I2I)"),
            Self::I2v => write!(f, "Image-to-Video (I2V)"),
            Self::V2v => write!(f, "Video-to-Video (V2V)"),
            Self::LipSync => write!(f, "LipSync"),
            Self::ImageLipSync => write!(f, "Image LipSync"),
            Self::VideoLipSync => write!(f, "Video LipSync"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInputSpec {
    pub title: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub input_type: Option<String>,
    pub description: Option<String>,
    pub default: Option<serde_json::Value>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<serde_json::Value>>,
    pub examples: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub endpoint: Option<String>,
    pub modality: ModelModality,
    pub family: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "hasPrompt")]
    pub has_prompt: Option<bool>,
    #[serde(rename = "imageField")]
    pub image_field: Option<String>,
    #[serde(rename = "maxImages")]
    pub max_images: Option<usize>,
    #[serde(default)]
    pub inputs: HashMap<String, serde_json::Value>,
}

impl Model {
    pub fn resolved_endpoint(&self) -> &str {
        self.endpoint.as_deref().unwrap_or(&self.id)
    }

    pub fn max_images(&self) -> usize {
        self.max_images.unwrap_or(1)
    }

    pub fn aspect_ratios(&self) -> Vec<String> {
        if let Some(ar_val) = self.inputs.get("aspect_ratio") {
            if let Some(arr) = ar_val.get("enum").and_then(|v| v.as_array()) {
                return arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
        vec![
            "16:9".to_string(),
            "9:16".to_string(),
            "1:1".to_string(),
            "4:3".to_string(),
            "21:9".to_string(),
        ]
    }

    pub fn durations(&self) -> Vec<u32> {
        if let Some(dur_val) = self.inputs.get("duration") {
            if let Some(arr) = dur_val.get("enum").and_then(|v| v.as_array()) {
                return arr.iter().filter_map(|v| v.as_u64().map(|n| n as u32)).collect();
            }
            if let Some(def) = dur_val.get("default").and_then(|v| v.as_u64()) {
                return vec![def as u32];
            }
        }
        vec![5, 10]
    }

    pub fn resolutions(&self) -> Vec<String> {
        if let Some(res_val) = self.inputs.get("resolution") {
            if let Some(arr) = res_val.get("enum").and_then(|v| v.as_array()) {
                return arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
        vec!["720p".to_string(), "1080p".to_string()]
    }

    pub fn modes(&self) -> Vec<String> {
        if let Some(mode_val) = self.inputs.get("mode") {
            if let Some(arr) = mode_val.get("enum").and_then(|v| v.as_array()) {
                return arr.iter().filter_map(|v| v.as_str().map(ToString::to_string)).collect();
            }
        }
        Vec::new()
    }

    pub fn qualities(&self) -> Vec<String> {
        if let Some(q_val) = self.inputs.get("quality") {
            if let Some(arr) = q_val.get("enum").and_then(|v| v.as_array()) {
                return arr.iter().filter_map(|v| v.as_str().map(ToString::to_string)).collect();
            }
        }
        Vec::new()
    }

    pub fn quality_field(&self) -> Option<&'static str> {
        if self.inputs.contains_key("resolution") {
            Some("resolution")
        } else if self.inputs.contains_key("quality") {
            Some("quality")
        } else {
            None
        }
    }

    pub fn default_aspect_ratio(&self) -> Option<String> {
        self.inputs
            .get("aspect_ratio")
            .and_then(|v| v.get("default"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
    }

    pub fn default_duration(&self) -> Option<u32> {
        self.inputs
            .get("duration")
            .and_then(|v| v.get("default"))
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
    }

    pub fn default_resolution(&self) -> Option<String> {
        self.inputs
            .get("resolution")
            .and_then(|v| v.get("default"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawModelsCatalog {
    pub t2i: Vec<serde_json::Value>,
    pub t2v: Vec<serde_json::Value>,
    pub i2i: Vec<serde_json::Value>,
    pub i2v: Vec<serde_json::Value>,
    pub v2v: Vec<serde_json::Value>,
    pub lipsync: Vec<serde_json::Value>,
    pub image_lipsync: Vec<serde_json::Value>,
    pub video_lipsync: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    models: Vec<Model>,
    by_id: HashMap<String, usize>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<ModelRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::new)
    }

    pub fn new() -> Self {
        let raw: RawModelsCatalog = serde_json::from_str(MODELS_JSON)
            .expect("Embedded models.json must be valid JSON");

        let mut models = Vec::new();
        let mut by_id = HashMap::new();

        let mut ingest_category = |items: Vec<serde_json::Value>, modality: ModelModality| {
            for val in items {
                let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if id.is_empty() {
                    continue;
                }
                let name = val.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
                let endpoint = val.get("endpoint").and_then(|v| v.as_str()).map(|s| s.to_string());
                let family = val.get("family").and_then(|v| v.as_str()).map(|s| s.to_string());
                let category = val.get("category").and_then(|v| v.as_str()).map(|s| s.to_string());
                let description = val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                let has_prompt = val.get("hasPrompt").and_then(|v| v.as_bool());
                let image_field = val.get("imageField").and_then(|v| v.as_str()).map(|s| s.to_string());
                let max_images = val.get("maxImages").and_then(|v| v.as_u64()).map(|n| n as usize);

                let inputs: HashMap<String, serde_json::Value> = val
                    .get("inputs")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.clone().into_iter().collect())
                    .unwrap_or_default();

                let model = Model {
                    id: id.clone(),
                    name,
                    endpoint,
                    modality,
                    family,
                    category,
                    description,
                    has_prompt,
                    image_field,
                    max_images,
                    inputs,
                };

                let idx = models.len();
                models.push(model);
                by_id.insert(id, idx);
            }
        };

        ingest_category(raw.t2i, ModelModality::T2i);
        ingest_category(raw.t2v, ModelModality::T2v);
        ingest_category(raw.i2i, ModelModality::I2i);
        ingest_category(raw.i2v, ModelModality::I2v);
        ingest_category(raw.v2v, ModelModality::V2v);
        ingest_category(raw.lipsync, ModelModality::LipSync);
        ingest_category(raw.image_lipsync, ModelModality::ImageLipSync);
        ingest_category(raw.video_lipsync, ModelModality::VideoLipSync);

        Self { models, by_id }
    }

    pub fn all(&self) -> &[Model] {
        &self.models
    }

    pub fn total_count(&self) -> usize {
        self.models.len()
    }

    pub fn get(&self, id: &str) -> Option<&Model> {
        self.by_id.get(id).map(|&idx| &self.models[idx])
    }

    pub fn by_modality(&self, modality: ModelModality) -> Vec<&Model> {
        self.models.iter().filter(|m| m.modality == modality).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Model> {
        let q = query.to_lowercase();
        self.models
            .iter()
            .filter(|m| {
                m.id.to_lowercase().contains(&q)
                    || m.name.to_lowercase().contains(&q)
                    || m.description.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || m.family.as_deref().unwrap_or("").to_lowercase().contains(&q)
            })
            .collect()
    }
}
