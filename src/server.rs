use crate::client::HiggsfieldClient;
use crate::models::{ModelModality, ModelRegistry};
use crate::storyboard::Storyboard;
use crate::studio::{
    compile_cinema_prompt, CinemaStudio, CinemaStudioRequest, ImageStudio, ImageStudioRequest,
    LipSyncStudio, LipSyncStudioRequest, VideoStudio, VideoStudioRequest,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

pub const STUDIO_HTML: &str = include_str!("../docs/index.html");

#[derive(Clone, Debug)]
pub struct AppState {
    pub client: Option<Arc<HiggsfieldClient>>,
    pub registry: &'static ModelRegistry,
}

#[derive(Debug, Deserialize)]
pub struct ModelsQuery {
    pub category: Option<String>,
    pub search: Option<String>,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/api/health", get(health_handler))
        .route("/api/models", get(list_models_handler))
        .route("/api/models/:id", get(get_model_handler))
        .route("/api/cinema/compile", post(compile_cinema_handler))
        .route("/api/generate/image", post(generate_image_handler))
        .route("/api/generate/video", post(generate_video_handler))
        .route("/api/generate/cinema", post(generate_cinema_handler))
        .route("/api/generate/lipsync", post(generate_lipsync_handler))
        .route("/api/storyboard/compile", post(compile_storyboard_handler))
        .route("/api/upload", post(upload_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn index_handler() -> Html<&'static str> {
    Html(STUDIO_HTML)
}

async fn health_handler(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "open-higgsfield-ai",
        "version": env!("CARGO_PKG_VERSION"),
        "models_count": state.registry.total_count(),
        "has_client": state.client.is_some()
    }))
}

async fn list_models_handler(
    State(state): State<AppState>,
    Query(params): Query<ModelsQuery>,
) -> Json<Value> {
    let models = if let Some(ref q) = params.search {
        state.registry.search(q)
    } else if let Some(ref cat) = params.category {
        match cat.to_lowercase().as_str() {
            "t2i" => state.registry.by_modality(ModelModality::T2i),
            "t2v" => state.registry.by_modality(ModelModality::T2v),
            "i2i" => state.registry.by_modality(ModelModality::I2i),
            "i2v" => state.registry.by_modality(ModelModality::I2v),
            "v2v" => state.registry.by_modality(ModelModality::V2v),
            "lipsync" => state.registry.by_modality(ModelModality::LipSync),
            _ => state.registry.all().iter().collect(),
        }
    } else {
        state.registry.all().iter().collect()
    };

    Json(json!({
        "total": models.len(),
        "models": models
    }))
}

async fn get_model_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.registry.get(&id) {
        Some(m) => Ok(Json(json!(m))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Model '{id}' not found")})),
        )),
    }
}

async fn compile_cinema_handler(
    Json(req): Json<CinemaStudioRequest>,
) -> Json<Value> {
    let compiled = compile_cinema_prompt(&req);
    Json(json!(compiled))
}

async fn generate_image_handler(
    State(state): State<AppState>,
    Json(req): Json<ImageStudioRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let client = state.client.ok_or_else(|| {
        (
            StatusCode::PRECONDITION_REQUIRED,
            Json(json!({"error": "No API key configured on server. Provide OPEN_HIGGSFIELD_API_KEY."})),
        )
    })?;

    let studio = ImageStudio::new(&client);
    match studio.generate(&req).await {
        Ok(resp) => Ok(Json(json!(resp))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

async fn generate_video_handler(
    State(state): State<AppState>,
    Json(req): Json<VideoStudioRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let client = state.client.ok_or_else(|| {
        (
            StatusCode::PRECONDITION_REQUIRED,
            Json(json!({"error": "No API key configured on server."})),
        )
    })?;

    let studio = VideoStudio::new(&client);
    match studio.generate(&req).await {
        Ok(resp) => Ok(Json(json!(resp))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

async fn generate_cinema_handler(
    State(state): State<AppState>,
    Json(req): Json<CinemaStudioRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let client = state.client.ok_or_else(|| {
        (
            StatusCode::PRECONDITION_REQUIRED,
            Json(json!({"error": "No API key configured on server."})),
        )
    })?;

    let studio = CinemaStudio::new(&client);
    match studio.generate(&req).await {
        Ok(resp) => Ok(Json(json!(resp))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

async fn generate_lipsync_handler(
    State(state): State<AppState>,
    Json(req): Json<LipSyncStudioRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let client = state.client.ok_or_else(|| {
        (
            StatusCode::PRECONDITION_REQUIRED,
            Json(json!({"error": "No API key configured on server."})),
        )
    })?;

    let studio = LipSyncStudio::new(&client);
    match studio.generate(&req).await {
        Ok(resp) => Ok(Json(json!(resp))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

async fn compile_storyboard_handler(
    Json(storyboard): Json<Storyboard>,
) -> Json<Value> {
    let edl = storyboard.to_cmx3600_edl();
    let total_duration = storyboard.total_duration();

    Json(json!({
        "title": storyboard.title,
        "shots_count": storyboard.shots.len(),
        "total_duration_seconds": total_duration,
        "edl_cmx3600": edl,
        "storyboard": storyboard
    }))
}

async fn upload_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let client = state.client.ok_or_else(|| {
        (
            StatusCode::PRECONDITION_REQUIRED,
            Json(json!({"error": "No API key configured on server."})),
        )
    })?;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()})))
    })? {
        let name = field.name().unwrap_or("file").to_string();
        if name == "file" || name == "asset" {
            let file_name = field.file_name().unwrap_or("upload.bin").to_string();
            let content_type = field.content_type().map(|s| s.to_string());
            let data = field.bytes().await.map_err(|e| {
                (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()})))
            })?;

            let uploaded_url = client
                .upload_file(&file_name, data.to_vec(), content_type.as_deref())
                .await
                .map_err(|e| {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
                })?;

            return Ok(Json(json!({
                "status": "success",
                "url": uploaded_url,
                "file_name": file_name
            })));
        }
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "No file field found in multipart upload"})),
    ))
}

pub async fn run_server(host: &str, port: u16, client: Option<HiggsfieldClient>) -> anyhow::Result<()> {
    let registry = ModelRegistry::global();
    let app_state = AppState {
        client: client.map(Arc::new),
        registry,
    };

    let router = create_router(app_state);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("🚀 Open-Higgsfield-AI Web Studio & REST Server running on http://{}", addr);

    axum::serve(listener, router).await?;
    Ok(())
}
