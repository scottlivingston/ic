<p align="center">
  <img src="ic.png" alt="IC" width="600">
</p>

# IC

A recreation of the IC helper robot from the horror game [ROUTINE](https://store.steampowered.com/app/606160/ROUTINE/).

## Features

- Speaking robotic face with text-to-speech
- Authentic CRT effects (glow, scanlines, flicker, curvature)
- Control via web browser
- Runs on desktop and Raspberry Pi

## Requirements

- [Rust](https://rustup.rs/)
- [mise](https://mise.jdx.dev/) (task runner)

## Quick Start

```bash
# Clone and enter directory
git clone <repo-url>
cd ic

# Install tools
mise install

# Run
mise run dev
```

Open the control panel at http://localhost:3000/admin

## Using IC

### Control Panel

Access the control panel at `http://localhost:3000/admin` to:

- Click preset phrases to make IC speak
- Type custom text in the input field
- Toggle CRT effects on/off
- Adjust effect intensity with sliders

### Effects

| Effect    | Description                      |
| --------- | -------------------------------- |
| Glow      | Bloom effect around bright areas |
| Scanlines | Horizontal CRT scan lines        |
| Flicker   | Random screen flicker            |
| Curvature | CRT barrel distortion            |
| Grid      | Pixel grid overlay               |

## Building

| Command             | Description        |
| ------------------- | ------------------ |
| `mise run dev`      | Development mode   |
| `mise run build`    | Release build      |
| `mise run build-pi` | Raspberry Pi build |

## API

For integration with other applications:

| Endpoint       | Method | Description                               |
| -------------- | ------ | ----------------------------------------- |
| `/api/speak`   | POST   | Make IC say something (`{"msg": "text"}`) |
| `/api/effects` | POST   | Update CRT effects                        |
| `/api/ws`      | GET    | WebSocket for real-time sync              |
