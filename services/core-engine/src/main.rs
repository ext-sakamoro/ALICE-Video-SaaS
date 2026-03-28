use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

struct AppState { start_time: Instant, stats: Mutex<Stats> }
struct Stats { total_encodes: u64, total_decodes: u64, total_transcodes: u64, total_frames_processed: u64 }

#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_ops: u64 }

#[derive(Deserialize)]
struct EncodeRequest { raw_b64: String, codec: Option<String>, width: Option<u32>, height: Option<u32>, fps: Option<f32>, crf: Option<u32>, container: Option<String> }
#[derive(Serialize)]
struct EncodeResponse { job_id: String, codec: String, container: String, width: u32, height: u32, fps: f32, duration_ms: u64, bitrate_kbps: u32, video_b64: String, processing_ms: u128 }

#[derive(Deserialize)]
struct DecodeRequest { video_b64: String, codec: Option<String>, start_ms: Option<u64>, end_ms: Option<u64>, output_format: Option<String> }
#[derive(Serialize)]
struct DecodeResponse { job_id: String, width: u32, height: u32, fps: f32, frame_count: u32, duration_ms: u64, raw_b64: String, processing_ms: u128 }

#[derive(Deserialize)]
struct TranscodeRequest { input_b64: String, input_codec: Option<String>, output_codec: Option<String>, crf: Option<u32>, container: Option<String>, scale: Option<String> }
#[derive(Serialize)]
struct TranscodeResponse { job_id: String, input_codec: String, output_codec: String, container: String, duration_ms: u64, size_reduction_pct: f32, video_b64: String, processing_ms: u128 }

#[derive(Deserialize)]
struct InfoRequest { video_b64: String }
#[derive(Serialize)]
struct VideoTrack { codec: String, width: u32, height: u32, fps: f32, bitrate_kbps: u32, duration_ms: u64 }
#[derive(Serialize)]
struct AudioTrack { codec: String, sample_rate: u32, channels: u32, bitrate_kbps: u32 }
#[derive(Serialize)]
struct InfoResponse { job_id: String, container: String, video_tracks: Vec<VideoTrack>, audio_tracks: Vec<AudioTrack>, total_size_bytes: u64, processing_ms: u128 }

#[derive(Serialize)]
struct StatsResponse { total_encodes: u64, total_decodes: u64, total_transcodes: u64, total_frames_processed: u64 }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "video_engine=info".into())).init();
    let state = Arc::new(AppState { start_time: Instant::now(), stats: Mutex::new(Stats { total_encodes: 0, total_decodes: 0, total_transcodes: 0, total_frames_processed: 0 }) });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/video/encode", post(encode))
        .route("/api/v1/video/decode", post(decode))
        .route("/api/v1/video/transcode", post(transcode))
        .route("/api/v1/video/info", post(info))
        .route("/api/v1/video/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("VIDEO_ADDR").unwrap_or_else(|_| "0.0.0.0:8115".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Video Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health { status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(), uptime_secs: s.start_time.elapsed().as_secs(), total_ops: st.total_encodes + st.total_decodes + st.total_transcodes })
}

async fn encode(State(s): State<Arc<AppState>>, Json(req): Json<EncodeRequest>) -> Json<EncodeResponse> {
    let t = Instant::now();
    let codec = req.codec.unwrap_or_else(|| "h264".into());
    let container = req.container.unwrap_or_else(|| "mp4".into());
    let w = req.width.unwrap_or(1920);
    let h = req.height.unwrap_or(1080);
    let fps = req.fps.unwrap_or(30.0);
    let frames = (req.raw_b64.len() / (w as usize * h as usize * 3)).max(1) as u64;
    { let mut st = s.stats.lock().unwrap(); st.total_encodes += 1; st.total_frames_processed += frames; }
    Json(EncodeResponse { job_id: uuid::Uuid::new_v4().to_string(), codec, container, width: w, height: h, fps, duration_ms: (frames * 1000) / fps as u64, bitrate_kbps: 4000, video_b64: "AAAAIGZ0eXBpc29t".into(), processing_ms: t.elapsed().as_millis() })
}

async fn decode(State(s): State<Arc<AppState>>, Json(req): Json<DecodeRequest>) -> Json<DecodeResponse> {
    let t = Instant::now();
    let bytes = req.video_b64.len() as u64;
    let frame_count = (bytes / 16384).max(1) as u32;
    { let mut st = s.stats.lock().unwrap(); st.total_decodes += 1; st.total_frames_processed += frame_count as u64; }
    Json(DecodeResponse { job_id: uuid::Uuid::new_v4().to_string(), width: 1920, height: 1080, fps: 30.0, frame_count, duration_ms: (frame_count as u64 * 1000) / 30, raw_b64: "cmF3ZnJhbWVz".into(), processing_ms: t.elapsed().as_millis() })
}

async fn transcode(State(s): State<Arc<AppState>>, Json(req): Json<TranscodeRequest>) -> Json<TranscodeResponse> {
    let t = Instant::now();
    let input_codec = req.input_codec.unwrap_or_else(|| "h264".into());
    let output_codec = req.output_codec.unwrap_or_else(|| "av1".into());
    let container = req.container.unwrap_or_else(|| "mp4".into());
    let bytes = req.input_b64.len() as u64;
    { let mut st = s.stats.lock().unwrap(); st.total_transcodes += 1; st.total_frames_processed += bytes / 16384; }
    Json(TranscodeResponse { job_id: uuid::Uuid::new_v4().to_string(), input_codec, output_codec, container, duration_ms: bytes / 16, size_reduction_pct: 35.0, video_b64: "AAAAIGZ0eXBpc29t".into(), processing_ms: t.elapsed().as_millis() })
}

async fn info(State(_s): State<Arc<AppState>>, Json(req): Json<InfoRequest>) -> Json<InfoResponse> {
    let t = Instant::now();
    let bytes = req.video_b64.len() as u64;
    Json(InfoResponse {
        job_id: uuid::Uuid::new_v4().to_string(),
        container: "mp4".into(),
        video_tracks: vec![VideoTrack { codec: "h264".into(), width: 1920, height: 1080, fps: 30.0, bitrate_kbps: 4000, duration_ms: bytes / 500 }],
        audio_tracks: vec![AudioTrack { codec: "aac".into(), sample_rate: 48000, channels: 2, bitrate_kbps: 128 }],
        total_size_bytes: bytes * 3 / 4,
        processing_ms: t.elapsed().as_millis(),
    })
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse { total_encodes: st.total_encodes, total_decodes: st.total_decodes, total_transcodes: st.total_transcodes, total_frames_processed: st.total_frames_processed })
}
