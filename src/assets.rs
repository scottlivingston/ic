//! Centralized asset embedding for the IC application.
//!
//! All static assets embedded in the binary are defined here.

// Admin web UI (served via HTTP)
pub const ADMIN_HTML: &str = include_str!("assets/admin/index.html");
pub const ADMIN_JS: &str = include_str!("assets/admin/admin.js");
pub const ADMIN_CSS: &str = include_str!("assets/admin/admin.css");
pub const ADMIN_FONT: &[u8] = include_bytes!("assets/admin/videotype.ttf");

// Face images (used by FaceLibrary and HTTP API)
pub const FACE_DEFAULT: &[u8] = include_bytes!("assets/faces/default.png");
pub const FACE_DEFAULT_TALKING: &[u8] = include_bytes!("assets/faces/default_talking.png");
pub const FACE_ANGRY: &[u8] = include_bytes!("assets/faces/angry.png");
pub const FACE_ANGRY_TALKING: &[u8] = include_bytes!("assets/faces/angry_talking.png");
