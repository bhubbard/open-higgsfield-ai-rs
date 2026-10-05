# 🦀 Native Rust Conversion Roadmap: Higgsfield Generative Video Stack

A comprehensive engineering plan and todo checklist for porting the open-source generative video, camera control, character consistency, and inference orchestration stack to native **Rust**.

---

## 📐 High-Level Rust Architecture & ML Stack

To achieve native C/C++ parity with zero-cost abstractions and memory safety, components are targeted to appropriate Rust ML runtimes:

* **Pure Native Tensors & Weights:** [`candle`](https://github.com/huggingface/candle) (Hugging Face) or [`burn`](https://github.com/tracel-ai/burn) (Burn Engine with WGPU/CUDA/LibTorch backends).
* **High-Throughput Vision/ONNX Encoders:** [`ort`](https://github.com/pykeio/ort) (Fast ONNX Runtime bindings for face embeddings, pose keypoint detectors, and optical flow).
* **Low-Level CUDA / Triton Acceleration:** [`cudarc`](https://github.com/coreylowman/cudarc) for custom spatio-temporal attention & FlashAttention-3 kernels.
* **Media Decoding / Encoding:** [`ffmpeg-next`](https://github.com/zmwangx/rust-ffmpeg) or native video demuxers ([`mp4`](https://crates.io/crates/mp4), [`rav1e`](https://crates.io/crates/rav1e)).
* **Serving & Orchestration:** [`axum`](https://github.com/tokio-rs/axum), [`tokio`](https://tokio.rs), and lock-free priority channels for dynamic batching.

---

## 1. 🎥 Base Video Generation Backbones (Diffusion Transformers)

### 1.1 LTX-Video (Lightricks) — *Priority: Phase 1 (Fastest to Port)*
- **Source:** [`Lightricks/LTX-Video`](https://github.com/Lightricks/LTX-Video)
- **Architecture:** Latent DiT, 28 transformer blocks, 3D VAE with spatial-temporal downsampling, rectified flow matching.
- **Target Backend:** `candle` (CUDA / Metal / CPU)
- [ ] Implement 3D Causal Conv VAE decoder in `candle-nn`.
- [ ] Port spatial-temporal 3D Rotary Position Embeddings (3D-RoPE).
- [ ] Port cross-attention blocks with T5 text conditioning.
- [ ] Implement Euler / Rectified Flow ODE step sampler in Rust.
- [ ] Load Safetensors weights directly without Python dependencies.
- [ ] Benchmark generation latency vs Python PyTorch baseline.

### 1.2 Wan2.1 (Wan-Video / Alibaba) — *Priority: Phase 2 (State-of-the-Art Open Foundation Video Model)*
- **Source:** [`Wan-Video/Wan2.1`](https://github.com/Wan-Video/Wan2.1)
- **Model Checkpoints Supported:**
  - `Wan2.1-T2V-14B` (Text-to-Video 14B, native 720p / 1080p, cinema quality)
  - `Wan2.1-T2V-1.3B` (Text-to-Video 1.3B, 480p, optimized for single consumer GPU / 8GB–12GB VRAM)
  - `Wan2.1-I2V-14B-720P` (Image-to-Video 14B, 720p high-fidelity motion continuation)
  - `Wan2.1-I2V-14B-480P` (Image-to-Video 14B, 480p fast generation)
- **Core Architecture & Math:**
  - **3D Causal Video VAE:** $4\times$ temporal downsampling, $8\times$ spatial downsampling, 16 latent channels. Asymmetric causal 3D convolutions preventing future frame leakage.
  - **Dual Multimodal Text Encoders:** Google UMT5-XXL (multilingual text reasoning) + CLIP-ViT-H/14 (visual semantic alignment).
  - **3D Multi-Frequency RoPE:** Separate temporal, vertical, and horizontal rotary positional frequencies: $\Theta_t, \Theta_h, \Theta_w$.
  - **Flow Matching ODE Integrator:** Continuous-time rectified flow formulation with adaptive Runge-Kutta / Euler ODE step solvers.
  - **I2V Conditioning:** 20-channel latent concatenation (16-channel noisy video latent + 4-channel image latent + binary temporal mask) and CLIP vision cross-attention.
- **Native Rust Porting Checklist:**
  - [ ] **Wan 3D Causal VAE (`higgsfield-vae`):**
    - [ ] Implement causal temporal padding and 3D convolution layers in `candle-nn`.
    - [ ] Port spatial-temporal residual blocks and upsampling/downsampling 3D convs.
    - [ ] Implement tiled 3D VAE decoding in Rust to decode $1280\times 720\times 81$ frame videos without VRAM overflow.
  - [ ] **Wan Dual Text Encoders:**
    - [ ] Port UMT5-XXL tokenizer (SentencePiece) and transformer encoder in pure `candle`.
    - [ ] Port OpenCLIP-ViT-H/14 text and vision encoders in `candle`.
    - [ ] Support dynamic offloading: unload text encoders from VRAM to host RAM before DiT initialization.
  - [ ] **Wan DiT Transformer Blocks (`higgsfield-dit`):**
    - [ ] Implement RMSNorm with learnable scaling.
    - [ ] Implement SwiGLU feed-forward networks (FFN) with fused activation.
    - [ ] Port 3D RoPE with exact frequency precomputation: $\mathbf{R}_{3D}(t, h, w) = \mathbf{R}_t \otimes \mathbf{R}_h \otimes \mathbf{R}_w$.
    - [ ] Implement Cross-Attention blocks with text token projections and modulated adaptive layer norms (AdaLN-Zero).
  - [ ] **Flow Matching Sampler:**
    - [ ] Implement Euler and Midpoint Flow Matching ODE samplers: $x_{t+\Delta t} = x_t + \Delta t \cdot v_\theta(x_t, t, c)$.
    - [ ] Add linear and dynamic shift time-step schedules.
  - [ ] **Image-to-Video (I2V) Pipeline:**
    - [ ] Encode input reference image through Wan-VAE encoder.
    - [ ] Construct channel-concatenated input tensor ($16 + 4 = 20$ channels).
    - [ ] Inject CLIP visual tokens into cross-attention layers.
  - [ ] **Hardware Quantization & Acceleration:**
    - [ ] Support native FP8 (`e4m3fn` weights with dynamic FP16/BF16 activation scaling).
    - [ ] Integrate SageAttention / FlashAttention-3 kernels via `cudarc`.
    - [ ] Implement Sequence Parallelism (splitting temporal video tokens across multi-GPU via RingAttention in Rust).

### 1.3 HunyuanVideo (Tencent) — *Priority: Phase 3 (Foundation Cinema Engine)*
- **Source:** [`Tencent/HunyuanVideo`](https://github.com/Tencent/HunyuanVideo)
- **Architecture:** 13B dual-stream DiT (visual + text streams communicate via cross-attention).
- **Target Backend:** `candle` with multi-stream execution or `tch-rs` (LibTorch FFI fallback for multi-node).
- [ ] Implement Hunyuan Dual-Stream Block architecture in Rust.
- [ ] Support LLaVA / MLLM text prompt embeddings loader.
- [ ] Implement 3D VAE with causal temporal padding.
- [ ] Implement classifier-free guidance (CFG) schedule loop.

### 1.4 CogVideoX & CogVideoX-5B (THUDM) — *Priority: Phase 4*
- **Source:** [`THUDM/CogVideoX`](https://github.com/THUDM/CogVideoX)
- **Architecture:** 3D VAE + Expert Transformer with 3D RoPE.
- [ ] Implement CogVideoX 3D VAE with patchification / unpatchification layers.
- [ ] Port Expert Transformer block (shared text-video attention projections).
- [ ] Implement progressive latent caching in native Rust memory buffers.

### 1.5 Wan-Video/diffusers (Official Wan Pipeline for Rust) — *Priority: Phase 1*
- **Source:** [`Wan-Video/diffusers`](https://github.com/Wan-Video/diffusers)
- **Architecture:** Official Wan DiT pipeline implementation for Hugging Face Diffusers:
  - `WanPipeline` (text-to-video and image-to-video unified execution flow).
  - `WanTransformer3DModel` (spatio-temporal self-attention, cross-attention, AdaLN-Zero).
  - ControlNet Union multi-control scaling interface (`from_multi` per-control-type weighting).
- [ ] Port `WanPipeline` into native `candle` / `burn` matching the official Diffusers execution semantics.
- [ ] Implement ControlNet Union multi-control scaling (simultaneous depth + pose + camera trajectory guidance in a single forward pass).
- [ ] Implement callback tensor hooks for progressive frame streaming and live browser previews.

---

## 2. 👤 Identity & Subject Consistency (FaceID / Character Lock)

### 2.1 PuLID & InstantID — *Priority: Phase 1*
- **Sources:** [`ToTheBeginning/PuLID`](https://github.com/ToTheBeginning/PuLID), [`InstantID/InstantID`](https://github.com/InstantID/InstantID)
- **Architecture:** InsightFace (Antelopev2) feature extraction + decoupled cross-attention / ID adapter.
- **Target Backend:** `ort` (ONNX Runtime) for InsightFace + `candle` for adapter cross-attention.
- [ ] Package InsightFace SCRFD face detector and ArcFace / Glint360k embedder into native Rust ONNX pipeline (`ort`).
- [ ] Implement 2D face alignment & affine transformation in Rust using `imageproc` and `nalgebra`.
- [ ] Implement PuLID ID-encoder (MLP projector mapping 512-d face vector to DiT cross-attention tokens).
- [ ] Port attention modulation layers for injection into DiT intermediate layers.

### 2.2 ConsisID (PKU-YuanGroup) — *Priority: Phase 2*
- **Source:** [`PKU-YuanGroup/ConsisID`](https://github.com/PKU-YuanGroup/ConsisID)
- **Architecture:** Frequency-aware identity conditioning built for DiT video models.
- [ ] Implement facial high/low-frequency decomposition filters in Rust.
- [ ] Port cross-frame identity attention constraint modules.

### 2.3 IP-Adapter & Multi-Image Reference Conditioning — *Priority: Phase 1*
- **Source:** [`tencent-ailab/IP-Adapter`](https://github.com/tencent-ailab/IP-Adapter)
- **Architecture:** CLIP-ViT image feature extractor + decoupled cross-attention layers.
- [ ] Port CLIP-ViT vision encoder to `candle` (or run via `ort`).
- [ ] Implement decoupled cross-attention forward pass: $Attn(Q, K_{text}, V_{text}) + \lambda \cdot Attn(Q, K_{img}, V_{img})$.
- [ ] Support multi-reference image conditioning (up to 14 reference shots) with weighted token pooling.

---

## 3. 🎬 Motion, Pose, and Camera Controllability

### 3.1 CameraCtrl & CameraCtrl-Plus — *Priority: Phase 1 (Higgsfield Core Feature)*
- **Source:** [`hehao13/CameraCtrl`](https://github.com/hehao13/CameraCtrl)
- **Architecture:** Plücker coordinate ray embeddings derived from camera intrinsic/extrinsic matrices.
- [ ] Implement camera trajectory generator in Rust (`nalgebra`):
  - [ ] Pan (yaw rotation around vertical axis)
  - [ ] Tilt (pitch rotation)
  - [ ] Roll (roll rotation around optical axis)
  - [ ] Dolly (translation along camera z-axis)
  - [ ] Truck / Pedestal (translation along x/y axes)
  - [ ] 360° Orbit (polar coordinate trajectory around target focus point)
- [ ] Calculate ray origin and direction vectors for each pixel frame: $r = o + t \cdot d$.
- [ ] Compute Plücker coordinates $L = (d, o \times d) \in \mathbb{R}^6$ per frame.
- [ ] Implement Plücker coordinate encoder (Conv3D / Linear projection) and inject into DiT self-attention.

### 3.2 MotionCtrl — *Priority: Phase 2*
- **Source:** [`TencentARC/MotionCtrl`](https://github.com/TencentARC/MotionCtrl)
- **Architecture:** Independent RT (Rotation & Translation) camera matrix conditioning + object trajectory masking.
- [ ] Implement camera posture embedding layer ($3 \times 4$ camera extrinsics).
- [ ] Implement bounding-box trajectory spline interpolator in Rust.

### 3.3 DWPose (Whole-Body Keypoint Detector) — *Priority: Phase 1*
- **Source:** [`IDEA-Research/DWPose`](https://github.com/IDEA-Research/DWPose)
- **Architecture:** RTMPose-based 133 whole-body keypoint detector (body, hands, face landmarks).
- **Target Backend:** `ort` (ONNX Runtime)
- [ ] Export ONNX weights for DWPose detector and estimator.
- [ ] Implement Rust pre-processing (letterbox resizing, normalization) and post-processing (keypoint NMS, confidence thresholding).
- [ ] Implement RGB skeleton visualization renderer in Rust (`imageproc`) for feeding into pose adapters.

### 3.4 MimicMotion & Champ — *Priority: Phase 2*
- **Sources:** [`Tencent/MimicMotion`](https://github.com/Tencent/MimicMotion), [`fudan-generative-vision/champ`](https://github.com/fudan-generative-vision/champ)
- **Architecture:** Confidence-aware pose guidance, SMPL-X 3D mesh rendering, DensePose UV maps.
- [ ] Implement confidence-weighted pose latent fusion in Rust.
- [ ] Implement SMPL-X parametric human mesh rasterizer or link via lightweight Rust WGPU pipeline.

### 3.5 Wan-Animate-2 (End-to-End Driving Character Animation & Viewpoint Control) — *Priority: Phase 1*
- **Source:** [`Wan-Video/Wan-Animate-2`](https://github.com/Wan-Video/Wan-Animate-2)
- **Architecture:** End-to-end character animation directly consuming driving videos in a redesigned Diffusion Transformer:
  - Eliminates intermediate keypoint/motion extractors (no DWPose or SMPL required).
  - Strong identity preservation via dense appearance feature cross-attention.
  - Text-driven viewpoint control (decouples camera angle from driving video perspective).
  - `Wan-Animate-2-Lite` distillation for real-time streaming animation.
- [ ] Implement direct driving-video frame encoder into Wan latent space in `candle`.
- [ ] Port cross-attention motion injection block coupling driving latents with source character latents.
- [ ] Implement text-driven viewpoint decoupling module (allows camera perspective alteration while following driving actor motion).
- [ ] Support `Wan-Animate-2 Distillation` (4-step/8-step) checkpoint in native Rust for sub-second streaming inference.

### 3.6 Wan-Dancer (Long-Duration Minute-Scale Music-to-Dance Generation) — *Priority: Phase 2*
- **Source:** [`Wan-Video/Wan-Dancer`](https://github.com/Wan-Video/Wan-Dancer)
- **Architecture:** Hierarchical minute-scale music-to-dance video generation surpassing the 20-second diffusion barrier. Generates stable 720p/30fps videos exceeding 1 minute across 5 dance genres.
- [ ] Implement full-track music feature extractor: audio beat tracking, onset envelope, and harmonic spectrogram features via `symphonia` + `rustfft`.
- [ ] Implement hierarchical keyframe planner (generating low-FPS global dance poses across the full 60+ seconds).
- [ ] Implement local temporal refinement DiT with dynamic frame rate adaptation via time-mapped RoPE embeddings.
- [ ] Port optical-flow motion continuity constraint and motion-speed control layer in native Rust.

---

## 4. 🎙️ Facial Expression & Audio-Driven Animation

### 4.1 LivePortrait (KwaiVGI) — *Priority: Phase 1 (Crucial for Acting/Selfie Videos)*
- **Source:** [`KwaiVGI/LivePortrait`](https://github.com/KwaiVGI/LivePortrait)
- **Architecture:** Implicit 3D keypoint extractor, head pose rotation matrix (yaw/pitch/roll), eye/lip expression deformation field.
- **Target Backend:** `ort` or `candle`
- [ ] Port Appearance Feature Extractor (F) to ONNX/`ort`.
- [ ] Port Motion Extractor (M) and canonical keypoint estimator ($x_c$).
- [ ] Port Expression and Pose deformation module: $x_i = R \cdot x_c + \delta_{exp} + t$.
- [ ] Implement 3D dense optical flow warping field and Spade decoder in native Rust / WGPU shader.
- [ ] Add real-time stitching and blending module (Poisson blending or seam feathering).

### 4.2 Hallo & Hallo2 — *Priority: Phase 2*
- **Source:** [`fudan-generative-vision/hallo`](https://github.com/fudan-generative-vision/hallo)
- **Architecture:** WavLM / Whisper audio encoder + hierarchical audio cross-attention into video latents.
- [ ] Implement audio feature extraction pipeline in Rust:
  - [ ] WAV decoding via `hound` or `symphonia`.
  - [ ] Mel-spectrogram computation in native Rust (`rustfft`).
  - [ ] Audio feature extraction using Whisper or WavLM via `candle-transformers`.
- [ ] Implement hierarchical audio-to-motion cross-attention layers.

### 4.3 EchoMimic — *Priority: Phase 3*
- **Source:** [`BadToBest/EchoMimic`](https://github.com/BadToBest/EchoMimic)
- **Architecture:** Dual audio + landmark conditioning without complex 3D morph targets.
- [ ] Implement simultaneous audio token + facial landmark tensor merger in Rust.

---

## 5. ⚡ Video Post-Processing (Interpolation & Upscaling)

### 5.1 RIFE (Real-Time Intermediate Flow Estimation) — *Priority: Phase 1*
- **Source:** [`hzwer/ECCV2022-RIFE`](https://github.com/hzwer/ECCV2022-RIFE)
- **Architecture:** IFNet (multi-scale optical flow estimation and backward warping).
- **Target Backend:** `ort` (ONNX) or pure `burn` / `wgpu` compute shader.
- [ ] Port IFNet architecture to ONNX / `ort` with TensorRT / DirectML / CoreML execution providers.
- [ ] Implement bilateral motion approximation and backward frame warper in Rust.
- [ ] Build multi-threaded frame queue: ingest 16/24 FPS video $\to$ output 60 FPS interpolated stream.

### 5.2 Real-ESRGAN Video & Real-CUGAN — *Priority: Phase 1*
- **Sources:** [`xinntao/Real-ESRGAN`](https://github.com/xinntao/Real-ESRGAN), [`bilibili/real-cugan`](https://github.com/bilibili/real-cugan)
- **Architecture:** Residual-in-Residual Dense Block (RRDBNet) / Conv2D super-resolution network.
- **Target Backend:** `ort` or `candle`
- [ ] Implement tiling inference to process large 1080p/4K video frames without GPU OOM (split into overlapping tiles, process, and blend).
- [ ] Integrate SIMD-accelerated color-space conversion (`RGB24` $\leftrightarrow$ `YUV420p`).

### 5.3 Microsoft DaVinci (Image/Video Enhancement & VSR Toolkit) — *Priority: Phase 1*
- **Source:** [`microsoft/DaVinci`](https://github.com/microsoft/DaVinci)
- **Architecture:** Intelligent image and video enhancement toolkit from Microsoft Research:
  - **General Image Super-Resolution (ISR):** Multi-scale restoration (2x, 3x, 4x) robust against complex unknown degradations (noise, motion blur, JPEG compression blocking artifacts).
  - **Compressed Video Super-Resolution (VSR):** Real-time video frame super-resolution specifically optimized for conference meetings, streaming, and talking-head portrait scenarios.
- **Target Backend:** `ort` (ONNX Runtime) with DirectML/CoreML/CUDA providers or `burn`
- [ ] Extract and convert DaVinci model checkpoints to standalone `.onnx` and `safetensors`.
- [ ] Implement native Rust pre/post-processing pipeline (color-space conversions, chunked streaming frames).
- [ ] Benchmark real-time performance on CPU and low-end GPUs for talking-head enhancement.
- [ ] Integrate into `higgsfield-portrait` and `higgsfield-postprocess` as a 1-click enhancement filter for AI-generated avatars.

---

## 6. 🧹 Dataset Curation, Splitting & Captioning

### 6.1 PySceneDetect Port (`scenedetect-rs`) — *Priority: Phase 1*
- **Source:** [`Breakthrough/PySceneDetect`](https://github.com/Breakthrough/PySceneDetect)
- **Architecture:** Content-aware threshold detector using HSV color histogram and edge difference metrics.
- [ ] Build native Rust CLI and library (`scenedetect-rs`).
- [ ] Demux video stream using `ffmpeg-next`.
- [ ] Implement HSV color-space delta calculation and fast threshold detection using SIMD.
- [ ] Export split scene timecodes in JSON, CSV, and CMX 3600 EDL.

### 6.2 Data-Juicer Video Operators — *Priority: Phase 2*
- **Source:** [`modelscope/data-juicer`](https://github.com/modelscope/data-juicer)
- [ ] Implement video aesthetic scoring filter using CLIP / LAION aesthetic predictor.
- [ ] Implement motion score filter (calculates average optical flow magnitude to reject static scenes).
- [ ] Implement text watermark and subtitle detector using OCR / edge clustering.

### 6.3 Dense Video Captioning (Qwen2-VL / LLaVA-Video) — *Priority: Phase 2*
- **Sources:** [`QwenLM/Qwen2-VL`](https://github.com/QwenLM/Qwen2-VL), [`LLaVA-VL/LLaVA-NeXT`](https://github.com/LLaVA-VL/LLaVA-NeXT)
- **Target Backend:** `candle-transformers`
- [ ] Port Qwen2-VL vision-language model in `candle`.
- [ ] Implement 3D patch merging and temporal window pooling.
- [ ] Build automated prompt generation script producing structured JSON:
  `{"subject": "...", "action": "...", "camera": "slow 360 orbit", "lighting": "..."}`.

---

## 7. 🚀 Inference Acceleration, Serving & Orchestration

### 7.1 FastVideo & Distillation Engine — *Priority: Phase 2*
- **Source:** [`hao-ai-lab/FastVideo`](https://github.com/hao-ai-lab/FastVideo)
- [ ] Implement sequence parallelism (RingAttention / DeepSpeed-Ulysses style) across multi-GPU in Rust.
- [ ] Implement 4-step / 8-step Rectified Flow step distillation in native Rust training loop.

### 7.2 Headless Execution Runtime (ComfyUI Workflow Engine) — *Priority: Phase 1*
- **Architecture:** Directed Acyclic Graph (DAG) executor in Rust that compiles ComfyUI-style node graphs into zero-overhead Rust execution chains.
- [ ] Build DAG scheduler using `petgraph` in Rust.
- [ ] Support caching of intermediate latent tensors between consecutive workflow executions.
- [ ] Support loading ComfyUI `.json` workflow formats directly into the Rust execution engine.

### 7.3 High-Throughput Serving & GPU Scheduler (`higgsfield-server`) — *Priority: Phase 1*
- **Target:** Production Axum HTTP / WebSocket server with priority queueing.
- [ ] Implement dynamic GPU batcher that aggregates requests across concurrent client connections.
- [ ] Implement preemption-resilient checkpointing for AWS/GCP spot GPU instances (saving step state to S3/R2).
- [ ] Implement streaming progress WebSockets emitting current step latents for mobile client real-time previews.

---

## 8. ⚙️ Hidden Low-Level Math & CUDA/Metal Libraries to Convert to Native Rust

These are critical under-the-hood C++/CUDA kernels and mathematical algorithms embedded inside Python repos that must be ported to native Rust to avoid performance bottlenecks or missing operator crashes.

### 8.1 4D/5D Grid Sampler & Backward Warper (`grid-sample-rs`) — *Priority: High*
- **Source:** Deep inside PyTorch core (`ATen/native/cuda/GridSampler.cu`)
- **Required By:** `LivePortrait`, `RIFE`, `MimicMotion`
- [ ] Implement 4D (`[N, C, H, W]`) and 5D (`[N, C, D, H, W]`) grid sampling in pure Rust.
- [ ] Support interpolation modes: Bilinear, Nearest, Bicubic.
- [ ] Support padding modes: `zeros`, `border`, and `reflection`.
- [ ] Write hand-tuned CUDA kernels via `cudarc` and Metal Shading Language (MSL) compute shaders.

### 8.2 Fast Softmax Splatting (`splatting-rs`) — *Priority: High*
- **Source:** [`sniklaus/softmax-splatting`](https://github.com/sniklaus/softmax-splatting)
- **Required By:** Advanced frame interpolation and occlusion-aware forward warping.
- [ ] Implement atomic forward-warping splatting kernel in CUDA / Metal.
- [ ] Support softmax depth-weighted aggregation to avoid visual tearing and pinhole artifacts.

### 8.3 TeaCache (Timestep Embedding Aware Caching) (`teacache-rs`) — *Priority: High*
- **Source:** [`ali-vilab/TeaCache`](https://github.com/ali-vilab/TeaCache)
- **Required By:** Fast video DiT inference (Wan2.1, HunyuanVideo, CogVideoX).
- [ ] Implement modulated timestep embedding L1-delta tracker in native Rust.
- [ ] Add skip-logic to transformer backbone loop when step delta falls below threshold $\delta_{thresh}$.
- [ ] Measure 2x–3x speedup on video diffusion generation with zero image degradation.

### 8.4 5-Point Umeyama Face Alignment & Similarity Transform (`umeyama-rs`) — *Priority: High*
- **Source:** [`deepinsight/insightface`](https://github.com/deepinsight/insightface) (`face_align.py`), `skimage.transform.SimilarityTransform`
- **Required By:** PuLID, InstantID, FaceID, LivePortrait.
- [ ] Implement closed-form Umeyama algorithm (SVD-based least squares estimation of scale, rotation, and translation) in pure Rust using `nalgebra`.
- [ ] Map 5 facial landmarks (eyes, nose, mouth corners) to canonical 112×112 and 512×512 reference coordinates.
- [ ] Eliminate the need for heavy OpenCV C++ bindings.

### 8.5 Seamless Poisson & Laplacian Pyramid Cloning (`poisson-blend-rs`) — *Priority: Medium*
- **Source:** LivePortrait `utils/cropper.py`, OpenCV `seamlessClone`
- **Required By:** Seamless portrait re-insertion into video backgrounds.
- [ ] Implement 2D Poisson boundary value solver using Conjugate Gradient in Rust.
- [ ] Implement Laplacian pyramid multi-band blending for seamless color and lighting transition.

### 8.6 SageAttention INT8/FP8 Quantized Attention (`sage-attn-rs`) — *Priority: High*
- **Source:** [`thu-ml/SageAttention`](https://github.com/thu-ml/SageAttention)
- **Required By:** Ultra-fast DiT self-attention and cross-attention blocks.
- [ ] Port smooth outlier suppression and INT8/FP8 matrix multiplications into PTX/CUDA kernels.
- [ ] Connect into `candle-nn` custom attention layer.

### 8.7 Plücker Camera Ray Geometry (`plucker-camera-rs`) — *Priority: High*
- **Source:** `CameraCtrl/cameractrl/models/pose_adaptor.py`, NeRF ray casting
- **Required By:** Precise 3D camera trajectory motion control.
- [ ] Implement camera pinhole model ray generator ($r = o + t \cdot d$).
- [ ] Compute 6D Plücker coordinates ($L = (d, o \times d)$) per pixel frame.
- [ ] Add trajectory presets: Orbit 360, Pan, Tilt, Roll, Dolly Zoom (Vertigo effect).

### 8.8 Spatio-Temporal 3D Patchify / Unpatchify Memory Shuffler — *Priority: Medium*
- **Source:** `diffusers` DiT models (`einops.rearrange`)
- **Required By:** Zero-allocation token packing for Video DiTs.
- [ ] Write zero-copy strided tensor view / CUDA kernel for 3D spacetime patch extraction: `[B, C, T, H, W] -> [B, (T/pt * H/ph * W/pw), (C * pt * ph * pw)]`.

### 8.9 Zero-Copy Hardware Video Demuxer (`zero-copy-video-rs`) — *Priority: Medium*
- **Source:** NVIDIA Video Codec SDK (NVDEC), Apple VideoToolbox
- **Required By:** High-throughput streaming and generation without PCIe memory stalls.
- [ ] Decode H.264/H.265 video packets directly into GPU VRAM buffers/textures (`AV_PIX_FMT_CUDA` / Metal texture surfaces).

---

## 9. 🎥 3D Camera Control, Lie Algebra & Scene Representations (3DGS)

### 9.1 Lie Group & Manifold Algebra on $\mathrm{SE}(3)$ / $\mathrm{SO}(3)$ (`se3-rs`) — *Priority: High*
- **Sources:** [`strasdat/Sophus`](https://github.com/strasdat/Sophus) (C++), [`artivis/manif`](https://github.com/artivis/manif) (C++), [`facebookresearch/theseus`](https://github.com/facebookresearch/theseus)
- **Architecture:** Special Euclidean Group $\mathrm{SE}(3)$ and Special Orthogonal Group $\mathrm{SO}(3)$ Lie theory.
- [ ] Implement matrix exponential and logarithmic maps: $T(t) = T_1 \cdot \exp(t \cdot \log(T_1^{-1} T_2))$.
- [ ] Port geodesic distance computation, tangent space Jacobians, and adjoint operations using `nalgebra`.
- [ ] Prevent matrix skewing and preserve rigidity during continuous camera keyframe interpolation.

### 9.2 $C^2$-Continuous $\mathrm{SE}(3)$ Spline Camera Trajectory Synthesizer (`cine-spline-rs`) — *Priority: High*
- **Sources:** [`borglab/gtsam`](https://github.com/borglab/gtsam), [`ethz-asl/curved_trajectory`](https://github.com/ethz-asl/curved_trajectory)
- **Architecture:** Cumulative cubic B-splines defined directly on $\mathrm{SE}(3)$ manifold.
- [ ] Implement jerk-free $C^2$-continuous acceleration curve calculation.
- [ ] Add Hollywood cinematic camera presets:
  - [ ] **360° Orbit:** Spherical spiral around target subject with adaptive look-at targeting.
  - [ ] **Vertigo / Dolly-Zoom:** Synchronized translation along optical $z$-axis with compensating focal length ($f_x, f_y$) variation.
  - [ ] **Crane / Jib Shot:** Parabolic height translation combined with downward pitch lock.
  - [ ] **Dutch Angle:** Smooth roll oscillation along optical axis.

### 9.3 Minimal Camera Solvers & Epipolar Geometry (`epipolar-rs`) — *Priority: Medium*
- **Sources:** [`PoseLib/PoseLib`](https://github.com/PoseLib/PoseLib), [`colmap/pycolmap`](https://github.com/colmap/pycolmap)
- **Architecture:** 5-point and 8-point minimal solvers for relative camera pose estimation.
- [ ] Implement Essential ($E = [t]_\times R$) and Fundamental ($F = K^{-T} E K^{-1}$) matrix computation.
- [ ] Add epipolar constraint validation to ensure novel view generation doesn't violate parallax physics.

### 9.4 Non-Linear Lens Distortion & Anamorphic Projection (`lens-optics-rs`) — *Priority: High*
- **Sources:** OpenCV Camera Calibration module (`cv::initUndistortRectifyMap`), COLMAP sensor models
- **Architecture:** Brown-Conrady polynomial radial/tangential distortion + anamorphic lens squish.
- [ ] Implement radial and tangential distortion formulas: $x' = x(1 + k_1 r^2 + k_2 r^4 + k_3 r^6) + 2 p_1 x y + p_2(r^2 + 2x^2)$.
- [ ] Implement $2\times$ horizontal anamorphic squeeze with oval bokeh distortion simulation.
- [ ] Support cinema sensor backplates (ARRI LF, RED Monstro, IMAX 70mm, Super 35).

### 9.5 Metric Depth Unprojection & 3D Point-Cloud Warper (`depth-warper-rs`) — *Priority: Phase 2*
- **Sources:** [`facebookresearch/pytorch3d`](https://github.com/facebookresearch/pytorch3d), [`hbb1/CamCo`](https://github.com/hbb1/CamCo)
- **Architecture:** Dense RGB-D point cloud projection and forward splatting.
- [ ] Back-project starting 2D image into 3D metric space using depth estimation maps (Depth Anything V2).
- [ ] Transform 3D point cloud by camera delta matrix $\Delta T \in \mathrm{SE}(3)$.
- [ ] Forward-splat transformed points into novel camera perspective to form motion guidance anchors.

### 9.6 3D Gaussian Splatting Real-Time Radiance Renderer (`gaussian-splatting-rs`) — *Priority: Phase 1*
- **Source:** [`graphdeco-inria/gaussian-splatting`](https://github.com/graphdeco-inria/gaussian-splatting)
- **Architecture:** 3D Gaussian scene representation & real-time differentiable rasterizer (`diff-gaussian-rasterization`).
- **Why it matters:** Enables instant 3D character avatars and real-time (>100 FPS) interactive camera sweeps prior to or in conjunction with video diffusion (e.g. LGM, TripoSR, SV3D, 4DGS).
- [ ] Implement tile-based Gaussian 2D screen projection and frustum culling in Rust (`cudarc` / Metal compute).
- [ ] Implement parallel GPU Radix sort on 64-bit Gaussian depth keys.
- [ ] Port front-to-back $\alpha$-blending accumulation shader: $C = \sum_{i \in \mathcal{N}} c_i \alpha_i \prod_{j=1}^{i-1} (1 - \alpha_j)$.
- [ ] Implement full Spherical Harmonics (degrees 0–3) for view-dependent directional radiance.
- [ ] Support fast binary `.ply` / `.splat` serialization and streaming web viewer.
- [ ] Support 4D Dynamic Gaussian Splatting (temporal deformation field for moving human bodies).

---

## 10. 🏛️ GraphDeco INRIA Ecosystem: 3D/4D Gaussian Radiance Fields & Neural Rendering

Complete native Rust conversion roadmap for all **22 repositories** from [`graphdeco-inria`](https://github.com/graphdeco-inria):

### 10.1 Core 3DGS & Differentiable Rasterization
* **10.1.1 [`graphdeco-inria/gaussian-splatting`](https://github.com/graphdeco-inria/gaussian-splatting)** (24k★):
  - [ ] Implement pure Rust training and optimization loop (Adam optimizer on 3D Gaussian parameters).
  - [ ] Adaptive density control: split large Gaussians and clone under-reconstructed Gaussians based on view-space positional gradient threshold $\nabla_{p} > \tau_{pos}$.
  - [ ] Spherical Harmonics (SH) degree 0 to 3 progressive activation scheduler.
* **10.1.2 [`graphdeco-inria/diff-gaussian-rasterization`](https://github.com/graphdeco-inria/diff-gaussian-rasterization)** (1.5k★):
  - [ ] Port forward & backward CUDA/C++ rasterization kernels to `cudarc` and Metal (MSL).
  - [ ] Implement tile-based bounding box intersections and shared-memory collaborative warp loading.
  - [ ] Exact analytical backward pass for gradients w.r.t. position, rotation quaternion, scale, opacity, and SH coefficients.

### 10.2 Massive Scale Hierarchies & LOD Streaming
* **10.2.1 [`graphdeco-inria/hierarchical-3d-gaussians`](https://github.com/graphdeco-inria/hierarchical-3d-gaussians)** (SIGGRAPH 2024, 1.4k★):
  - [ ] Implement hierarchical octree / cluster tree representation for city-scale 3D Gaussian datasets.
  - [ ] Level-of-Detail (LOD) selection based on projected screen-space footprint.
  - [ ] Asynchronous out-of-core streaming from disk / network cache into GPU VRAM.
* **10.2.2 [`graphdeco-inria/gaussian-hierarchy`](https://github.com/graphdeco-inria/gaussian-hierarchy)**:
  - [ ] Spatial clustering and bottom-up Gaussian merging algorithms in native Rust.
* **10.2.3 [`graphdeco-inria/hierarchy-rasterizer`](https://github.com/graphdeco-inria/hierarchy-rasterizer)**:
  - [ ] Multi-scale CUDA/Metal rasterizer executing dynamic LOD traversal without CPU stalls.
* **10.2.4 [`graphdeco-inria/hierarchy-viewer`](https://github.com/graphdeco-inria/hierarchy-viewer)**:
  - [ ] Native high-performance interactive desktop viewer using `winit` + `wgpu` or embedded WebAssembly/WebGL viewer.

### 10.3 Ray Tracing & Specular Reflections on 3D Gaussians
* **10.3.1 [`graphdeco-inria/gray`](https://github.com/graphdeco-inria/gray)** (GRay: Ray Tracing 3D Gaussians Near the Speed of Splats):
  - [ ] Construct hardware BVH (Bounding Volume Hierarchy) over oriented 3D Gaussian ellipsoids.
  - [ ] Port GPU ray-Gaussian intersection tests using DXR / Vulkan Ray Tracing / OptiX or custom CUDA traversal.
  - [ ] Real-time primary rays, secondary shadow rays, and ambient occlusion.
* **10.3.2 [`graphdeco-inria/editable-gaussian-reflections`](https://github.com/graphdeco-inria/editable-gaussian-reflections)**:
  - [ ] Raytrace physically-based specular reflections on glossy 3D Gaussian surfaces.
  - [ ] Interactive editing of surface roughness and reflection color parameters.

### 10.4 Immediate & Unposed Reconstruction
* **10.4.1 [`graphdeco-inria/on-the-fly-nvs`](https://github.com/graphdeco-inria/on-the-fly-nvs)** (SIGGRAPH 2025):
  - [ ] On-the-fly reconstruction pipeline from unposed, continuous video streams.
  - [ ] Simultaneous camera tracking and incremental Gaussian scene expansion in Rust.
* **10.4.2 [`graphdeco-inria/i3dgs`](https://github.com/graphdeco-inria/i3dgs)** (SIGGRAPH 2026):
  - [ ] Immediate 3D Gaussian Splat reconstruction from unordered image collections with global consistency.
  - [ ] Fast global alignment graph optimization in Rust.

### 10.5 Memory Reduction & Mobile Compression
* **10.5.1 [`graphdeco-inria/reduced-3dgs`](https://github.com/graphdeco-inria/reduced-3dgs)**:
  - [ ] Vector quantization (VQ) codebook generator for Gaussian scales, rotations, and SH colors.
  - [ ] Sensitivity-aware Gaussian pruning (prunes 50%–70% of Gaussians with $<0.1\text{dB}$ PSNR loss).
  - [ ] Compact binary `.splat` codec compressing 1GB PLY files down to $<50\text{MB}$ for instant mobile loading.

### 10.6 Relighting, PBR, & Material Decomposition
* **10.6.1 [`graphdeco-inria/gaussian-splatting-relighting`](https://github.com/graphdeco-inria/gaussian-splatting-relighting)** (EGSR 2024) & **[`controlnet-diffusers-relighting`](https://github.com/graphdeco-inria/controlnet-diffusers-relighting)**:
  - [ ] Diffusion-based multi-illumination synthesis pipeline for radiance fields.
  - [ ] Normal and albedo map extraction from 3D Gaussian fields.
* **10.6.2 [`graphdeco-inria/intr_decomp_3dgs`](https://github.com/graphdeco-inria/intr_decomp_3dgs)** (I3D 2026):
  - [ ] Intrinsic decomposition of 3D Gaussians into diffuse albedo, surface normals, and roughness channels.
  - [ ] Physically-based relighting with environment HDR maps in Rust.
* **10.6.3 [`graphdeco-inria/lighting-3d-generative-cars`](https://github.com/graphdeco-inria/lighting-3d-generative-cars)** (EG 2024):
  - [ ] Physically-based environment lighting for 3D generative vehicle models.
* **10.6.4 [`graphdeco-inria/svbrdf-evaluation`](https://github.com/graphdeco-inria/svbrdf-evaluation)**:
  - [ ] Spatially Varying Bidirectional Reflectance Distribution Function (SVBRDF) evaluation from generative models.

### 10.7 Spatio-Temporal Dynamics, Object Compositing & Tooling
* **10.7.1 [`graphdeco-inria/adaptive-spatio-temporal-gaussians`](https://github.com/graphdeco-inria/adaptive-spatio-temporal-gaussians)** (Eurographics 2026):
  - [ ] 4D Gaussian representation with temporal Fourier / polynomial trajectory basis for dynamic and oscillatory motion (fluttering clothing, hair, foliage).
* **10.7.2 [`graphdeco-inria/dot3d`](https://github.com/graphdeco-inria/dot3d)** (EGSR 2026):
  - [ ] Lighting-consistent 3D object / character insertion across heterogeneous radiance fields.
  - [ ] Spherical Harmonics ambient lighting harmonization.
* **10.7.3 [`graphdeco-inria/nerfshop`](https://github.com/graphdeco-inria/nerfshop)**:
  - [ ] Interactive space deformation: cage-based, skeleton-driven, and translation gizmo manipulations.
* **10.7.4 [`graphdeco-inria/gs-texturing`](https://github.com/graphdeco-inria/gs-texturing)** (EGSR 2025):
  - [ ] Content-aware 2D texture parameterization and projection onto 3D Gaussian surfaces.
* **10.7.5 [`graphdeco-inria/3dlayers`](https://github.com/graphdeco-inria/3dlayers)**:
  - [ ] Multi-layer semantic radiance scene decomposition (foreground character, midground props, background environment).
* **10.7.6 [`graphdeco-inria/graphdecoviewer`](https://github.com/graphdeco-inria/graphdecoviewer)**:
  - [ ] Native OpenGL / Vulkan / Metal interactive rendering viewer with camera trajectory recording.

---

## 11. 🎬 Maxon Computer Ecosystem: Cinematic Rigs, Shaders, Color Science & DCC Interop

Complete native Rust conversion roadmap for all **12 repositories** from [`Maxon-Computer`](https://github.com/Maxon-Computer):

### 11.1 Shading, Optics & Color Science (Redshift)
* **11.1.1 [`Maxon-Computer/Redshift-OSL-Shaders`](https://github.com/Maxon-Computer/Redshift-OSL-Shaders)** (399★):
  - [ ] Port Redshift Open Shading Language (OSL) procedural shader collection to Rust / WGPU / Slang / Metal Shading Language:
    - [ ] Procedural noise patterns (Perlin, Simplex, Worley, Voronoi, Flakes).
    - [ ] Thin-film optical iridescence and coating interference.
    - [ ] Lens distortion shaders (anamorphic streak flares, chromatic aberration, optical vignetting).
    - [ ] Camera projection mapping shaders (triplanar, cylindrical, spherical ray marching).
* **11.1.2 [`Maxon-Computer/Redshift-OCIO-Configs`](https://github.com/Maxon-Computer/Redshift-OCIO-Configs)**:
  - [ ] Implement OpenColorIO (OCIO) color management pipeline in pure Rust (`higgsfield-ocio`).
  - [ ] Support color-space transforms: ACEScg, ACES2065-1, Rec.709, sRGB, ARRI LogC3/LogC4, RED Log3G10.
  - [ ] Real-time 3D LUT (Look-Up Table) evaluation and filmic tonemapping for video outputs.

### 11.2 Cinematic Scene Graph & Camera Rigging (Cinema 4D)
* **11.2.1 [`Maxon-Computer/Cinema-4D-Cpp-API-Examples`](https://github.com/Maxon-Computer/Cinema-4D-Cpp-API-Examples)**:
  - [ ] Port C4D scene graph architecture into native Rust (`higgsfield-scene`):
    - [ ] Hierarchical DAG node tree with dirty flags and matrix propagation.
    - [ ] Procedural geometry generators (sweeps, lofts, clones).
    - [ ] Deformer pipeline (bend, twist, FFD, skin/bone skeletal weights).
    - [ ] Custom camera rigs (two-node look-at camera, crane rig, rail camera).
* **11.2.2 [`Maxon-Computer/Cinema-4D-Python-API-Examples`](https://github.com/Maxon-Computer/Cinema-4D-Python-API-Examples)** (160★):
  - [ ] Port animation track system: Hermite F-curves, keyframe tangent interpolation, auto-tangents, and cyclical post-extrapolation.
  - [ ] Implement take/variant management system (rendering different camera angles or lighting passes from one scene).
* **11.2.3 [`Maxon-Computer/Cinema-4D-DevKitchen-2018`](https://github.com/Maxon-Computer/Cinema-4D-DevKitchen-2018)** & **[`Cinema-4D-Coburg-Praxiswochen`](https://github.com/Maxon-Computer/Cinema-4D-Coburg-Praxiswochen)**:
  - [ ] Port advanced C4D C++ SDK examples: custom volumetric shaders, ray-tracing integration, and motion blur generation.

### 11.3 Digital Sculpting & Mesh Pipeline (ZBrush)
* **11.3.1 [`Maxon-Computer/ZBrush-Python-API-Examples`](https://github.com/Maxon-Computer/ZBrush-Python-API-Examples)**:
  - [ ] Port ZBrush automation workflows to native Rust:
    - [ ] Multi-resolution subdivision surface computation.
    - [ ] Automated displacement and tangent-space normal map baking.
    - [ ] Mesh decimation / quad-remeshing algorithms for generating low-poly proxies from 3D Gaussian / NeRF surfaces.

### 11.4 Motion Graphics Compositing & Tooling (Autograph & Studio Tools)
* **11.4.1 [`Maxon-Computer/AutographPythonDoc`](https://github.com/Maxon-Computer/AutographPythonDoc)**:
  - [ ] Port node-based 2D/3D motion graphics compositing engine:
    - [ ] Layer blending modes (Add, Multiply, Screen, Overlay, Soft Light).
    - [ ] Temporal keyframing and multi-pass compositor (integrating AI video plates with CGI renders).
* **11.4.2 [`Maxon-Computer/Cinema-4D-Visual-Studio-Code-Extension`](https://github.com/Maxon-Computer/Cinema-4D-Visual-Studio-Code-Extension)** & **[`Cinema-4D-Legacy-Visual-Studio-Code-Bridge-Plugin`](https://github.com/Maxon-Computer/Cinema-4D-Legacy-Visual-Studio-Code-Bridge-Plugin)**:
  - [ ] Implement JSON-RPC / WebSocket bridge for remote script execution and live scene reloading from external IDEs.
* **11.4.3 [`Maxon-Computer/Cinema-4D-Legacy-ResEdit`](https://github.com/Maxon-Computer/Cinema-4D-Legacy-ResEdit)**:
  - [ ] Parse and generate declarative C4D resource description layouts (`.res` / `.str`) for plugin UI definitions.
* **11.4.4 [`Maxon-Computer/Cinema-4D-Python-R25-Symbols-Finder`](https://github.com/Maxon-Computer/Cinema-4D-Python-R25-Symbols-Finder)**:
  - [ ] Port symbol mapper and API deprecation analyzer for DCC compatibility.

---

## 12. 💡 Blender Cycles Path Tracing Engine (Production Physics-Based Photorealism)

Complete native Rust conversion roadmap for **[`blender/cycles`](https://github.com/blender/cycles)**:

### 12.1 Core Monte Carlo Integrator & Path Tracer
* **Architecture:** Unidirectional Monte Carlo path tracer with Next Event Estimation (NEE) and Multiple Importance Sampling (MIS).
* [ ] Implement pure Rust path tracing integrator loop (`higgsfield-cycles` / `cycles-rs`):
  - [ ] Russian Roulette ray termination based on throughput luminance.
  - [ ] Light tree sampling (importance-sampled direct illumination for thousands of lights).
  - [ ] Volume scattering and absorption integrator (Henyey-Greenstein phase function for atmospheric haze, mist, god rays).
  - [ ] Subsurface Scattering (Random Walk SSS / Burley dipole) for photorealistic skin, eye cornea, and teeth in digital human generation.

### 12.2 BSDFs, Microfacets & Material Closures
* [ ] Implement Disney / Burley Principled BSDF in pure Rust:
  - [ ] GGX / Multiscatter GGX microfacet distribution with Smith shadowing/masking.
  - [ ] Anisotropic reflection and roughness rotation.
  - [ ] Sheen, clearcoat with custom absorption, and transmission with Fresnel IOR equations.
  - [ ] Thin-walled transmission for foliage and cloth simulation.

### 12.3 Multi-Backend Hardware Ray Tracing Acceleration
* [ ] Implement hardware-accelerated BVH traversal pipelines via Rust bindings:
  - [ ] **NVIDIA OptiX (`optix-rs`):** Hardware ray-tracing cores with motion blur BVHs and fast shader binding tables (SBT).
  - [ ] **Apple MetalRT (`metal-rs`):** `MTLAccelerationStructure` hardware ray tracing for M-series Apple Silicon.
  - [ ] **Intel Embree (`embree-rs`):** CPU / oneAPI BVH builder (SAH spatial partitioning) and packet ray stream traversal.
  - [ ] **Vulkan Ray Tracing (`ash` / `vulkano`):** Cross-platform Ray Tracing KHR extensions.

### 12.4 Cinematic Cameras & Temporal Motion Blur
* [ ] Implement realistic physical camera optics in Rust:
  - [ ] Thin-lens depth of field: variable aperture diameter, blades count, and polygonal bokeh geometry.
  - [ ] Multi-segment deformation motion blur (interpolating deforming mesh vertices across shutter open/close interval).
  - [ ] Camera transform motion blur (coupling rigid $\mathrm{SE}(3)$ camera moves with exposure duration).
  - [ ] Panoramic, fisheye (equidistant, equisolid, stereographic), and $360^\circ$ VR stereoscopic camera projections.

### 12.5 Multi-Pass AOV (Arbitrary Output Variables) & Optical Flow Vectors
* [ ] Implement multi-pass render buffer generation:
  - [ ] **Vector Pass:** Exact pixel-space motion vectors ($dx, dy$) between consecutive frames (directly injects ground-truth optical flow into RIFE and video diffusion).
  - [ ] **Depth & Normal Passes:** 32-bit floating point metric depth maps and world-space surface normals.
  - [ ] **Cryptomatte:** ID-based automatic object, material, and character matte segmentation.
  - [ ] **Albedo & Direct/Indirect Lighting Passes:** Clean separation for neural relighting and post-processing.

### 12.6 Neural Denoising & USD Hydra Integration
* [ ] Implement native OpenImageDenoise (OIDN) and OptiX AI Denoiser execution in Rust:
  - [ ] Multi-pass feature guidance (denoising with albedo + normal passes for crisp edge preservation).
* [ ] Build Hydra Render Delegate (`hdCycles-rs`) for direct integration with Universal Scene Description (USD) stages.

---

## 13. 🌐 Open Generative AI Studio & Multi-Provider Gateway

Complete native Rust conversion roadmap for **[`Anil-matcha/Open-Generative-AI`](https://github.com/Anil-matcha/Open-Generative-AI)** and its integration with [`open-generative-ai-rs`](../open-generative-ai-rs):

### 13.1 400+ Model Catalog Across 8 Generative Domains
* **Architecture:** Unified parameter, aspect ratio, and pricing schemas for 400+ commercial and open-weight models.
* [ ] Port model registry and capability matrices across all 8 domains:
  - [ ] **Text-to-Image (70+ models):** FLUX.1 Dev/Schnell, Seedream 5.0, Ideogram v2, Recraft v3, SDXL Turbo.
  - [ ] **Image-to-Image (70+ models):** FLUX Kontext Pro (up to 14-image reference conditioning), Clarity Upscaler v2.
  - [ ] **Text-to-Video (85+ models):** Kling v3 Pro, Sora 2 Turbo, Veo 3 Cinematic, Wan 2.1 / 2.6 T2V, Seedance 2.5 T2V.
  - [ ] **Image-to-Video (120+ models):** Kling v2.1 I2V, Veo 3 I2V, Wan 2.1 I2V.
  - [ ] **Video-to-Video (35+ models):** Vibe Motion v2, AI Clipping Pro (smart 9:16 reframe & active speaker tracking).
  - [ ] **Lip Sync & Talking Avatars (15 models):** Infinite Talk I2V, Wan 2.2 Speech-to-Video, SadTalker v2.
  - [ ] **Body Swap & Recast (3 models):** Subject Recast v1, Face Swap Pro, Virtual Try-on v2.
  - [ ] **Audio & Music (15+ models):** Text-to-Music v2, ElevenLabs Voice TTS.

### 13.2 Unified Multi-Provider Gateway & Dispatcher
* [ ] Implement multi-provider client dispatchers in pure Rust:
  - [ ] **Muapi.ai Client:** Multipart media upload, job queuing (`POST /api/v1/{endpoint}`), and exponential polling.
  - [ ] **Local Inference Adapter:** Native integration with local `sd.cpp` GGUF binaries and local Wan2GP endpoints.
  - [ ] **Higgsfield Native Provider:** First-class direct in-process binding to local `higgsfield-rs` execution pipelines.
  - [ ] **Deterministic Mock Engine:** Offline zero-cost dry-runs and regression tests.

### 13.3 DAG Workflow Execution Engine
* [ ] Implement multi-stage pipeline runner with variable interpolation and cycle detection:
  - [ ] Variable resolution engine supporting `{{steps.<id>.output}}` and `{{inputs.<key>}}`.
  - [ ] **AI Influencer Studio:** Character Anchor Portrait $\to$ Neural Voiceover $\to$ Lip Sync Talking Video.
  - [ ] **Popcorn Storyboard Studio:** Script Keyframe $\to$ 70mm Cinematic Motion $\to$ Original Soundtrack.
  - [ ] **Viral Shorts Engine:** Video Generation $\to$ Neural Speech $\to$ Lip Sync $\to$ 9:16 Reframe.

### 13.4 Embedded Axum Web Studio
* [ ] Single self-contained static binary ($<18\text{MB}$) serving the full dark-mode responsive studio at `http://127.0.0.1:8080/studio`:
  - [ ] Embedded web assets (`rust-embed` / `include_dir`).
  - [ ] Real-time job progress tracking via WebSockets.
  - [ ] Model Explorer with 1-click CLI and cURL command generation.

---

## 📦 Suggested Crate Layout

```text
higgsfield-rs/
├── Cargo.toml
├── crates/
│   ├── higgsfield-core/          # Common types: LatentTensor, CameraTrajectory, ModelSpec
│   ├── higgsfield-cuda/          # Low-level CUDA/Metal kernels (grid_sample, splatting, sage-attn)
│   ├── higgsfield-se3/           # Lie algebra SE(3)/SO(3) manifolds, exp/log maps, and geodesics
│   ├── higgsfield-spline/        # C^2-continuous B-splines & F-curves for cinematic camera paths
│   ├── higgsfield-lens/          # Brown-Conrady, Anamorphic 2x squeeze, optical distortion
│   ├── higgsfield-ocio/          # OpenColorIO color management (ACEScg, LogC, filmic tonemapping)
│   ├── higgsfield-osl/           # Procedural OSL-style shader runtime (thin-film, flakes, noise)
│   ├── higgsfield-scene/         # C4D-style scene graph DAG, procedural deformers & camera rigs
│   ├── higgsfield-cycles/        # Blender Cycles path tracer, Principled BSDF, Random Walk SSS & OIDN
│   ├── higgsfield-gaussian/      # 3D/4D Gaussian Splatting rasterizer, BVH raytracer (GRay) & compression
│   ├── higgsfield-gaussian-tree/ # Hierarchical LOD octree & streaming for massive scenes
│   ├── higgsfield-geom/          # Umeyama alignment, Poisson blending, Plücker ray math
│   ├── higgsfield-teacache/      # Timestep-aware latent cache for 2x-3x DiT speedup
│   ├── higgsfield-vae/           # 3D Spatio-Temporal VAEs (LTX, Wan, CogVideoX)
│   ├── higgsfield-dit/           # DiT backbones & transformer blocks (Candle/Burn)
│   ├── higgsfield-camera/        # Trajectory generators, camera extrinsics & epipolar solvers
│   ├── higgsfield-face/          # InsightFace, PuLID, InstantID (ORT + Candle)
│   ├── higgsfield-portrait/      # LivePortrait motion warper & expression retargeter
│   ├── higgsfield-postprocess/   # RIFE, Real-ESRGAN, and Microsoft DaVinci (ISR/VSR)
│   ├── higgsfield-scenedetect/   # High-speed scene cut detector (FFmpeg + SIMD)
│   ├── higgsfield-gateway/       # Multi-provider client & 400+ model catalog (Open Generative AI)
│   ├── higgsfield-workflow/      # Multi-stage DAG workflow engine (Influencer, Storyboard, Shorts)
│   ├── higgsfield-engine/        # ComfyUI-compatible DAG executor & pipeline runner
│   ├── higgsfield-server/        # Axum REST & WebSocket streaming server
│   └── higgsfield-cli/           # Unified CLI tool
```
