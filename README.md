# ALICE-Video-SaaS

Video codec SaaS built on the ALICE-Video engine. Provides hardware-accelerated video encoding, decoding, transcoding, and metadata extraction via REST API.

## Architecture

```
Client --> API Gateway (8110) --> Core Engine (8115)
```

- **API Gateway**: Authentication, rate limiting, request proxying
- **Core Engine**: Codec pipeline, transcoding scheduler, container muxer

## Features

- H.264, H.265/HEVC, AV1, VP9 encoding and decoding
- Hardware acceleration (NVENC, VideoToolbox, VAAPI)
- Adaptive bitrate ladder generation
- Container format support (MP4, MKV, WebM, HLS, DASH)
- Frame-accurate trimming and concatenation
- Thumbnail and sprite sheet extraction
- HDR to SDR tone mapping

## API Endpoints

### Core Engine (port 8115)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check with uptime and stats |
| POST | `/api/v1/video/encode` | Encode raw or compressed video |
| POST | `/api/v1/video/decode` | Decode video to raw frames |
| POST | `/api/v1/video/transcode` | Transcode between codecs/containers |
| POST | `/api/v1/video/info` | Extract video metadata |
| GET | `/api/v1/video/stats` | Operational statistics |

### API Gateway (port 8110)

Proxies all `/api/v1/*` routes to the Core Engine with JWT/API-Key auth and token-bucket rate limiting.

## Quick Start

```bash
# Core Engine
cd services/core-engine
VIDEO_ADDR=0.0.0.0:8115 cargo run --release

# API Gateway
cd services/api-gateway
GATEWAY_ADDR=0.0.0.0:8110 CORE_ENGINE_URL=http://localhost:8115 cargo run --release
```

## Example Request

```bash
curl -X POST http://localhost:8115/api/v1/video/transcode \
  -H "Content-Type: application/json" \
  -d '{"input_b64":"...","input_codec":"h264","output_codec":"av1","crf":28,"container":"mp4"}'
```

## License

AGPL-3.0-or-later. SaaS operators must publish complete service source code under AGPL-3.0.
