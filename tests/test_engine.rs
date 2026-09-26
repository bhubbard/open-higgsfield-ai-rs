use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use open_higgsfield_ai::{
    client::GenerationResponse,
    models::{ModelModality, ModelRegistry},
    server::{create_router, AppState},
    storyboard::{ShotType, Storyboard, StoryboardShot},
    studio::{
        build_nano_banana_prompt, compile_cinema_prompt, CameraMotion3D, CinemaCamera, CinemaLens,
        CinemaLighting, CinemaStudioRequest,
    },
};
use serde_json::Value;
use tower::ServiceExt;

#[test]
fn test_models_registry_loads_all_models() {
    let registry = ModelRegistry::new();
    assert!(
        registry.total_count() >= 200,
        "Registry must contain at least 200 models, found: {}",
        registry.total_count()
    );

    let t2i_models = registry.by_modality(ModelModality::T2i);
    assert_eq!(t2i_models.len(), 51, "Must have exactly 51 T2I models");

    let t2v_models = registry.by_modality(ModelModality::T2v);
    assert_eq!(t2v_models.len(), 42, "Must have exactly 42 T2V models");

    let i2i_models = registry.by_modality(ModelModality::I2i);
    assert_eq!(i2i_models.len(), 57, "Must have exactly 57 I2I models");

    let i2v_models = registry.by_modality(ModelModality::I2v);
    assert_eq!(i2v_models.len(), 61, "Must have exactly 61 I2V models");

    let v2v_models = registry.by_modality(ModelModality::V2v);
    assert_eq!(v2v_models.len(), 1, "Must have 1 V2V model");

    let lipsync_models = registry.by_modality(ModelModality::LipSync);
    assert_eq!(lipsync_models.len(), 9, "Must have 9 LipSync models");
}

#[test]
fn test_models_search() {
    let registry = ModelRegistry::new();
    let flux_results = registry.search("flux");
    assert!(!flux_results.is_empty(), "Should find models matching 'flux'");

    let kling_results = registry.search("kling");
    assert!(!kling_results.is_empty(), "Should find models matching 'kling'");

    let banana_results = registry.search("nano-banana");
    assert!(banana_results.len() >= 2, "Should find nano-banana variants");
}

#[test]
fn test_model_helper_methods_and_defaults() {
    let registry = ModelRegistry::new();
    let nano = registry.get("nano-banana").expect("nano-banana should exist");

    assert_eq!(nano.name, "Nano Banana");
    assert_eq!(nano.resolved_endpoint(), "nano-banana");
    assert!(nano.aspect_ratios().contains(&"16:9".to_string()));
    assert!(nano.aspect_ratios().contains(&"1:1".to_string()));
    assert_eq!(nano.default_aspect_ratio(), Some("1:1".to_string()));

    // Test Nano Banana 2 Edit max images conditioning
    let nano2_edit = registry
        .get("nano-banana-2-edit")
        .expect("nano-banana-2-edit should exist");
    assert_eq!(nano2_edit.max_images(), 14, "Nano Banana 2 Edit supports up to 14 reference images");

    // Test Video Model duration and resolution defaults
    let seedance = registry
        .get("seedance-lite-t2v")
        .expect("seedance-lite-t2v should exist");
    assert_eq!(seedance.default_duration(), Some(5));
    assert_eq!(seedance.default_resolution(), Some("480p".to_string()));
    assert!(seedance.resolutions().contains(&"720p".to_string()));
    assert!(seedance.resolutions().contains(&"1080p".to_string()));
}

#[test]
fn test_cinema_prompt_compilation() {
    let req = CinemaStudioRequest {
        prompt: "Neon cybernetic metropolis in dense rainfall".to_string(),
        camera: Some(CinemaCamera::Modular8KDigital),
        lens: Some(CinemaLens::ClassicAnamorphic),
        focal_length_mm: Some(35),
        aperture: Some("f/1.4".to_string()),
        lighting: Some(CinemaLighting::CyberpunkNeon),
        motion: Some(CameraMotion3D {
            pan_deg: 25.0,
            tilt_deg: -10.0,
            roll_deg: 0.0,
            zoom_scale: 1.2,
            dolly_m: 2.0,
            truck_m: 0.0,
            crane_m: 0.0,
            orbit_deg: 45.0,
        }),
        aspect_ratio: Some("21:9".to_string()),
        resolution: Some("4K".to_string()),
        seed: Some(42),
    };

    let compiled = compile_cinema_prompt(&req);

    assert!(compiled.full_prompt.contains("Neon cybernetic metropolis"));
    assert!(compiled.full_prompt.contains("modular 8K digital cinema camera"));
    assert!(compiled.full_prompt.contains("classic anamorphic lens"));
    assert!(compiled.full_prompt.contains("35mm (natural cinematic human-eye perspective)"));
    assert!(compiled.full_prompt.contains("aperture f/1.4"));
    assert!(compiled.full_prompt.contains("cyberpunk neon"));
    assert!(compiled.full_prompt.contains("slow pan right"));
    assert!(compiled.full_prompt.contains("smooth dolly in"));
    assert!(compiled.full_prompt.contains("360 orbit camera rotation 45°"));
}

#[test]
fn test_build_nano_banana_prompt_exact() {
    let prompt = build_nano_banana_prompt(
        "A portrait of a futuristic warrior",
        "Modular 8K Digital",
        "Classic Anamorphic",
        35,
        "f/1.4",
    );

    assert!(prompt.contains("A portrait of a futuristic warrior"));
    assert!(prompt.contains("shot on a modular 8K digital cinema camera"));
    assert!(prompt.contains("using a classic anamorphic lens at 35mm (natural cinematic perspective)"));
    assert!(prompt.contains("aperture f/1.4"));
    assert!(prompt.contains("shallow depth of field, creamy bokeh"));
    assert!(prompt.contains("cinematic lighting, natural color science, high dynamic range, professional photography, ultra-detailed, 8K resolution"));
}

#[test]
fn test_camera_and_lens_parsing() {
    let cam1: CinemaCamera = "s35".parse().unwrap();
    assert_eq!(cam1, CinemaCamera::StudioDigitalS35);

    let cam2: CinemaCamera = "70mm".parse().unwrap();
    assert_eq!(cam2, CinemaCamera::GrandFormat70mmFilm);

    let cam3: CinemaCamera = "modular8k".parse().unwrap();
    assert_eq!(cam3, CinemaCamera::Modular8KDigital);

    let lens1: CinemaLens = "tilt".parse().unwrap();
    assert_eq!(lens1, CinemaLens::CreativeTiltLens);

    let lens2: CinemaLens = "anamorphic".parse().unwrap();
    assert_eq!(lens2, CinemaLens::ClassicAnamorphic);

    let lens3: CinemaLens = "macro".parse().unwrap();
    assert_eq!(lens3, CinemaLens::ExtremeMacro);
}

#[test]
fn test_camera_motion_3d_formatting() {
    let neutral = CameraMotion3D::default();
    assert_eq!(neutral.description(), "locked-off cinematic tripod shot");

    let dynamic = CameraMotion3D {
        pan_deg: -30.0,
        tilt_deg: 15.0,
        roll_deg: 0.0,
        zoom_scale: 2.0,
        dolly_m: -1.5,
        truck_m: 2.0,
        crane_m: 3.0,
        orbit_deg: 180.0,
    };
    let desc = dynamic.description();
    assert!(desc.contains("slow pan left"));
    assert!(desc.contains("smooth tilt up"));
    assert!(desc.contains("optical zoom 2.0x"));
    assert!(desc.contains("smooth dolly out"));
    assert!(desc.contains("tracking truck right"));
    assert!(desc.contains("crane shot ascending"));
    assert!(desc.contains("360 orbit camera rotation 180°"));
}

#[test]
fn test_storyboard_cmx3600_edl_export() {
    let mut sb = Storyboard::new("Cyberpunk Short Film");
    sb.add_shot(StoryboardShot {
        shot_number: 1,
        title: "Establishing Neon Cityscape".to_string(),
        shot_type: ShotType::Cinema,
        duration_seconds: 4.0,
        prompt: "Panoramic view of Neo-Tokyo".to_string(),
        cinema_spec: None,
        model: Some("nano-banana".to_string()),
        audio_url: None,
        image_url: None,
        result_url: None,
    });

    sb.add_shot(StoryboardShot {
        shot_number: 2,
        title: "Detective Walking".to_string(),
        shot_type: ShotType::Video,
        duration_seconds: 5.0,
        prompt: "Detective in trenchcoat walks forward".to_string(),
        cinema_spec: None,
        model: Some("seedance-lite-t2v".to_string()),
        audio_url: None,
        image_url: None,
        result_url: None,
    });

    assert_eq!(sb.shots.len(), 2);
    assert_eq!(sb.total_duration(), 9.0);

    let edl = sb.to_cmx3600_edl();
    assert!(edl.contains("TITLE: Cyberpunk Short Film"));
    assert!(edl.contains("FCM: NON-DROP FRAME"));
    assert!(edl.contains("001  AX       V     C        00:00:00:00 00:00:04:00 00:00:00:00 00:00:04:00"));
    assert!(edl.contains("002  AX       V     C        00:00:00:00 00:00:05:00 00:00:04:00 00:00:09:00"));
}

#[test]
fn test_generation_response_parsing() {
    // 1. Direct url
    let resp1 = GenerationResponse {
        id: Some("req-123".to_string()),
        request_id: Some("req-123".to_string()),
        status: Some("completed".to_string()),
        url: Some("https://example.com/direct.png".to_string()),
        outputs: None,
        error: None,
        raw: serde_json::json!({}),
    };
    assert_eq!(resp1.output_url(), Some("https://example.com/direct.png"));
    assert!(resp1.is_completed());
    assert!(!resp1.is_failed());

    // 2. Outputs array
    let resp2 = GenerationResponse {
        id: Some("req-456".to_string()),
        request_id: Some("req-456".to_string()),
        status: Some("success".to_string()),
        url: None,
        outputs: Some(vec!["https://example.com/array.mp4".to_string()]),
        error: None,
        raw: serde_json::json!({}),
    };
    assert_eq!(resp2.output_url(), Some("https://example.com/array.mp4"));
    assert!(resp2.is_completed());

    // 3. Nested raw output
    let resp3 = GenerationResponse {
        id: Some("req-789".to_string()),
        request_id: Some("req-789".to_string()),
        status: Some("succeeded".to_string()),
        url: None,
        outputs: None,
        error: None,
        raw: serde_json::json!({
            "output": {
                "url": "https://example.com/nested.webp"
            }
        }),
    };
    assert_eq!(resp3.output_url(), Some("https://example.com/nested.webp"));

    // 4. Failed
    let resp4 = GenerationResponse {
        id: Some("req-err".to_string()),
        request_id: Some("req-err".to_string()),
        status: Some("failed".to_string()),
        url: None,
        outputs: None,
        error: Some("CUDA OOM".to_string()),
        raw: serde_json::json!({}),
    };
    assert!(resp4.is_failed());
    assert!(!resp4.is_completed());
}

#[tokio::test]
async fn test_server_router_endpoints() {
    let registry = ModelRegistry::global();
    let state = AppState {
        client: None,
        registry,
    };
    let app = create_router(state);

    // 1. Health check
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let health_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(health_json["status"], "ok");
    assert_eq!(health_json["service"], "open-higgsfield-ai");
    assert_eq!(health_json["models_count"], 230);

    // 2. Query models endpoint
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/models?category=lipsync")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let models_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(models_json["total"], 9);

    // 3. Get single model
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/models/nano-banana")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let model_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(model_json["name"], "Nano Banana");

    // 4. Cinema compile endpoint
    let cinema_payload = serde_json::json!({
        "prompt": "Cyberpunk hero on motorcycle",
        "camera": "FullFrameCineDigital",
        "lens": "CreativeTiltLens",
        "focal_length_mm": 50,
        "aperture": "f/1.4",
        "lighting": "GoldenHour"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cinema/compile")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&cinema_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let compile_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(compile_json["full_prompt"].as_str().unwrap().contains("Cyberpunk hero on motorcycle"));
    assert!(compile_json["camera_spec"].as_str().unwrap().contains("full-frame digital cinema camera"));
}
