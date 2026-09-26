use crate::error::{Error, Result};
use reqwest::{header, multipart, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, info, warn};

pub const DEFAULT_BASE_URL: &str = "https://api.muapi.ai";
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(2);
pub const DEFAULT_IMAGE_MAX_ATTEMPTS: usize = 60;
pub const DEFAULT_VIDEO_MAX_ATTEMPTS: usize = 900;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub id: Option<String>,
    pub request_id: Option<String>,
    pub status: Option<String>,
    pub url: Option<String>,
    pub outputs: Option<Vec<String>>,
    pub error: Option<String>,
    pub raw: Value,
}

impl GenerationResponse {
    pub fn resolved_id(&self) -> Option<&str> {
        self.request_id.as_deref().or(self.id.as_deref())
    }

    pub fn output_url(&self) -> Option<&str> {
        if let Some(ref u) = self.url {
            return Some(u.as_str());
        }
        if let Some(ref outputs) = self.outputs {
            if let Some(first) = outputs.first() {
                return Some(first.as_str());
            }
        }
        if let Some(nested_url) = self.raw.get("output").and_then(|o| o.get("url")).and_then(|u| u.as_str()) {
            return Some(nested_url);
        }
        None
    }

    pub fn is_completed(&self) -> bool {
        match self.status.as_deref().map(|s| s.to_lowercase()) {
            Some(ref s) if s == "completed" || s == "succeeded" || s == "success" => true,
            _ => false,
        }
    }

    pub fn is_failed(&self) -> bool {
        match self.status.as_deref().map(|s| s.to_lowercase()) {
            Some(ref s) if s == "failed" || s == "error" => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HiggsfieldClient {
    base_url: String,
    api_key: String,
    client: Client,
    poll_interval: Duration,
}

impl HiggsfieldClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_options(api_key, DEFAULT_BASE_URL, DEFAULT_POLL_INTERVAL)
    }

    pub fn from_env() -> Result<Self> {
        let key = std::env::var("OPEN_HIGGSFIELD_API_KEY")
            .or_else(|_| std::env::var("MUAPI_KEY"))
            .or_else(|_| std::env::var("HIGGSFIELD_API_KEY"))
            .map_err(|_| Error::MissingApiKey)?;

        Ok(Self::new(key))
    }

    pub fn with_options(api_key: impl Into<String>, base_url: impl Into<String>, poll_interval: Duration) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            client,
            poll_interval,
        }
    }

    pub fn set_base_url(&mut self, base_url: impl Into<String>) {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
    }

    pub fn set_poll_interval(&mut self, interval: Duration) {
        self.poll_interval = interval;
    }

    /// Submit a generation payload to an endpoint and optionally poll for completion.
    pub async fn submit<T: Serialize>(&self, endpoint: &str, payload: &T) -> Result<GenerationResponse> {
        let url = format!("{}/api/v1/{}", self.base_url, endpoint);
        debug!("Submitting request to {url}");

        let resp = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-api-key", &self.api_key)
            .json(payload)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let raw: Value = resp.json().await?;
        let id = raw.get("id").and_then(|v| v.as_str()).map(ToString::to_string);
        let request_id = raw.get("request_id").and_then(|v| v.as_str()).map(ToString::to_string);
        let status_str = raw.get("status").and_then(|v| v.as_str()).map(ToString::to_string);
        let url_str = raw.get("url").and_then(|v| v.as_str()).map(ToString::to_string);
        let outputs = raw
            .get("outputs")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(ToString::to_string)).collect());
        let error_str = raw.get("error").and_then(|v| v.as_str()).map(ToString::to_string);

        Ok(GenerationResponse {
            id,
            request_id,
            status: status_str,
            url: url_str,
            outputs,
            error: error_str,
            raw,
        })
    }

    /// Poll for the result of an asynchronous prediction request.
    pub async fn poll_result(&self, request_id: &str, max_attempts: usize) -> Result<GenerationResponse> {
        let poll_url = format!("{}/api/v1/predictions/{}/result", self.base_url, request_id);
        info!("Polling for result: {poll_url} (max attempts: {max_attempts})");

        for attempt in 1..=max_attempts {
            tokio::time::sleep(self.poll_interval).await;

            let resp = match self
                .client
                .get(&poll_url)
                .header("x-api-key", &self.api_key)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    warn!("Network error while polling (attempt {attempt}/{max_attempts}): {e}");
                    if attempt == max_attempts {
                        return Err(Error::Network(e));
                    }
                    continue;
                }
            };

            let status = resp.status();
            if status.is_server_error() {
                debug!("Server error {status} on poll attempt {attempt}, retrying...");
                continue;
            }

            if !status.is_success() {
                let err_text = resp.text().await.unwrap_or_default();
                return Err(Error::Api {
                    status: status.as_u16(),
                    message: err_text,
                });
            }

            let raw: Value = resp.json().await?;
            let status_val = raw.get("status").and_then(|s| s.as_str()).unwrap_or("").to_lowercase();

            if status_val == "completed" || status_val == "succeeded" || status_val == "success" {
                let outputs: Option<Vec<String>> = raw.get("outputs").and_then(|v| v.as_array()).map(|arr| {
                    arr.iter().filter_map(|x| x.as_str().map(ToString::to_string)).collect()
                });
                let mut url = raw.get("url").and_then(|v| v.as_str()).map(ToString::to_string);
                if url.is_none() {
                    if let Some(ref o) = outputs {
                        url = o.first().cloned();
                    }
                }
                return Ok(GenerationResponse {
                    id: Some(request_id.to_string()),
                    request_id: Some(request_id.to_string()),
                    status: Some(status_val),
                    url,
                    outputs,
                    error: None,
                    raw,
                });
            }

            if status_val == "failed" || status_val == "error" {
                let err_msg = raw
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown model generation failure");
                return Err(Error::GenerationFailed(err_msg.to_string()));
            }

            debug!("Poll {attempt}/{max_attempts}: status={status_val}");
        }

        Err(Error::Timeout {
            attempts: max_attempts,
            elapsed_secs: (max_attempts as u64) * self.poll_interval.as_secs(),
        })
    }

    /// Submit a job and automatically wait/poll for completion.
    pub async fn submit_and_poll<T: Serialize>(
        &self,
        endpoint: &str,
        payload: &T,
        max_attempts: usize,
    ) -> Result<GenerationResponse> {
        let initial = self.submit(endpoint, payload).await?;

        // If the initial response already contains output URL or completed status, return it
        if initial.output_url().is_some() || initial.is_completed() {
            return Ok(initial);
        }

        let request_id = initial
            .resolved_id()
            .ok_or_else(|| Error::GenerationFailed("No request ID returned from API".to_string()))?;

        self.poll_result(request_id, max_attempts).await
    }

    /// Upload a binary asset (image, video, or audio) to Muapi.ai file hosting.
    pub async fn upload_file(&self, filename: &str, bytes: Vec<u8>, mime_type: Option<&str>) -> Result<String> {
        let url = format!("{}/api/v1/upload_file", self.base_url);
        let mime = mime_type.unwrap_or("application/octet-stream");

        let part = multipart::Part::bytes(bytes)
            .file_name(filename.to_string())
            .mime_str(mime)
            .map_err(|e| Error::InvalidParameter(e.to_string()))?;

        let form = multipart::Form::new().part("file", part);

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .multipart(form)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let data: Value = resp.json().await?;
        let file_url = data
            .get("url")
            .or_else(|| data.get("file_url"))
            .or_else(|| data.get("data").and_then(|d| d.get("url")))
            .and_then(|u| u.as_str())
            .ok_or_else(|| Error::GenerationFailed("Upload succeeded but no URL was returned".to_string()))?;

        Ok(file_url.to_string())
    }
}
