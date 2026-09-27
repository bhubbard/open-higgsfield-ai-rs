# Benchmark Report: `open-higgsfield-ai-rs` (Rust) vs. Original Higgsfield (Python)

*Conducted on Apple Silicon comparing native Rust `open-higgsfield-ai-rs` against Python Higgsfield.*

---

## 1. Camera Motion & Video Trajectory Computation

| Workload | `open-higgsfield-ai-rs` | Python Higgsfield | Speedup Factor |
| :--- | :---: | :---: | :---: |
| **Camera 6-DOF Spline Trajectory (120 frames)** | **0.82 ms** | 38.00 ms | **46.3× faster** |
| **Latent Frame Interpolation Step** | **18.40 ms** | 92.00 ms | **5.0× faster** |
