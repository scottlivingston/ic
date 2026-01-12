# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

IC is a Rust/Bevy application recreating the IC helper robot from the horror game ROUTINE. It's a speaking robotic face with CRT effects, designed for Raspberry Pi kiosk displays with web browser control.

**Stack:** Rust 2024 edition, Bevy 0.17 (ECS game engine), Axum 0.8 (HTTP server), rodio (audio), SAM TTS engine

## Build Commands

```bash
mise run dev        # Development mode with RUST_BACKTRACE=1
mise run build      # Release build for local machine
mise run build-pi   # Cross-compile for Raspberry Pi (aarch64-unknown-linux-gnu)
mise run build-ui   # Build admin web UI (Svelte → src/assets/)
```

Requires [mise](https://mise.jdx.dev/) task runner.

## Architecture

### Plugin Structure (Bevy ECS)

```
main.rs → Bevy App
├── ServerPlugin (server.rs)     # Axum HTTP server on :3000
├── CrtPlugin (crt/mod.rs)       # Post-processing CRT shader
├── FacePlugin (face.rs)         # Sprite-based animated face
└── AudioPlugin (audio/mod.rs)   # SAM TTS and audio playback
```

### Threading Model

- **Bevy Main Thread:** Game loop, rendering, ECS systems
- **Tokio Server Thread:** HTTP requests via mpsc channel to Bevy
- **Audio Playback Thread:** rodio device management
- **SAM TTS Thread:** Speech synthesis (isolated to prevent blocking)

Communication between web server and Bevy uses `mpsc::Sender<ServerCommand>` passed through Axum state.

### Data Flow

```
POST /api/speak → Axum → mpsc channel → SayEvent → SAM TTS → Audio Playback
                                                         → Face Animation (mouth syncs)
```

### Key Files

- `src/events.rs` - Bevy events: SayEvent, EffectsEvent, VolumeEvent
- `src/sam/` - Complete SAM TTS engine (text→phonemes→u8 PCM at 22050Hz)
- `src/crt/shaders/crt_sdf.wgsl` - Single-pass WGSL shader with SDF-based glow
- `admin/` - Svelte source for web UI (build output goes to `src/assets/`)
- `src/assets/` - Built admin UI embedded in binary via `include_str!()`/`include_bytes!()`

## REST API (port 3000)

- `GET /api/health` - Health check
- `POST /api/speak` - `{"msg": "text"}`
- `POST /api/effects` - CRT settings (glow, scanlines, flicker, curvature, grid)
- `POST /api/volume` - `{"volume": 0.0-1.0}`
- `GET /` - Admin web UI

## SAM TTS Voice Parameters

Located in `src/audio/sam.rs`:
```rust
const SAM_SPEED: u8 = 140;   // Higher = faster
const SAM_PITCH: u8 = 60;    // Lower = higher voice
const SAM_MOUTH: u8 = 220;   // F1 formant
const SAM_THROAT: u8 = 220;  // F2 formant
```

## Release Profile

Binary size optimized for Pi Zero 2 W: LTO enabled, stripped, single codegen unit, opt-level "s", panic abort.
