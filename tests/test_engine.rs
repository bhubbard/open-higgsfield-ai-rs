use open_higgsfield_ai::{
    models::{ModelModality, ModelRegistry},
    storyboard::{ShotType, Storyboard, StoryboardShot},
    studio::{
        compile_cinema_prompt, CameraMotion3D, CinemaCamera, CinemaLens, CinemaLighting,
        CinemaStudioRequest,
    },
};

#[test]
fn test_models_registry_loads_all_models() {
    let registry = ModelRegistry::new();
    assert!(registry.total_count() >= 200, "Registry must contain at least 200 models, found: {}", registry.total_count());

    let t2i_models = registry.by_modality(ModelModality::T2i);
    assert!(!t2i_models.is_empty(), "Must have T2I models");

    let t2v_models = registry.by_modality(ModelModality::T2v);
    assert!(!t2v_models.is_empty(), "Must have T2V models");

    let lipsync_models = registry.by_modality(ModelModality::LipSync);
    assert!(!lipsync_models.is_empty(), "Must have LipSync models");

    let nano = registry.get("nano-banana");
    assert!(nano.is_some(), "Nano Banana must exist");
    let nano_m = nano.unwrap();
    assert_eq!(nano_m.name, "Nano Banana");
    assert!(nano_m.aspect_ratios().contains(&"16:9".to_string()));
}

#[test]
fn test_models_search() {
    let registry = ModelRegistry::new();
    let flux_results = registry.search("flux");
    assert!(!flux_results.is_empty(), "Should find models matching 'flux'");

    let kling_results = registry.search("kling");
    assert!(!kling_results.is_empty(), "Should find models matching 'kling'");
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
    assert!(edl.contains("001  AX       V     C"));
    assert!(edl.contains("002  AX       V     C"));
}
