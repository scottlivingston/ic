//! Face Library - loads and manages face patterns
//!
//! Embedded faces (default, angry) are always available.
//! Users can add custom faces by placing 40x30 PNG images in ~/.config/ic/faces/

use bevy::prelude::*;
use image::GenericImageView;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::assets::{FACE_ANGRY, FACE_ANGRY_TALKING, FACE_DEFAULT, FACE_DEFAULT_TALKING};

pub const GRID_WIDTH: usize = 40;
pub const GRID_HEIGHT: usize = 30;

/// Face pattern data with resting and talking variants
#[derive(Clone)]
pub struct FaceData {
    pub resting: [[u8; GRID_WIDTH]; GRID_HEIGHT],
    pub talking: [[u8; GRID_WIDTH]; GRID_HEIGHT],
}

/// Resource containing all available faces
#[derive(Resource)]
pub struct FaceLibrary {
    faces: HashMap<String, FaceData>,
}

impl FaceLibrary {
    /// Load face library with embedded defaults + custom faces from disk
    pub fn load() -> Self {
        let mut faces = HashMap::new();

        // Insert embedded faces (always available)
        if let (Some(resting), Some(talking)) = (
            load_face_from_bytes(FACE_DEFAULT),
            load_face_from_bytes(FACE_DEFAULT_TALKING),
        ) {
            faces.insert("default".to_string(), FaceData { resting, talking });
        } else {
            panic!("Failed to load embedded default face");
        }

        if let (Some(resting), Some(talking)) = (
            load_face_from_bytes(FACE_ANGRY),
            load_face_from_bytes(FACE_ANGRY_TALKING),
        ) {
            faces.insert("angry".to_string(), FaceData { resting, talking });
        } else {
            warn!("Failed to load embedded angry face");
        }

        // Load custom faces from config directory
        if let Some(faces_dir) = get_faces_directory() {
            if faces_dir.exists() {
                load_custom_faces(&mut faces, &faces_dir);
            }
        }

        Self { faces }
    }

    /// Get a face by name, falling back to "default" if not found
    pub fn get(&self, name: &str) -> &FaceData {
        self.faces
            .get(name)
            .unwrap_or_else(|| self.faces.get("default").expect("default face must exist"))
    }

    /// List all available face names
    pub fn list_faces(&self) -> Vec<&str> {
        self.faces.keys().map(|s| s.as_str()).collect()
    }
}

fn get_faces_directory() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("ic").join("faces"))
}

fn load_custom_faces(faces: &mut HashMap<String, FaceData>, dir: &PathBuf) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    // Collect all PNG files and group by base name
    let mut file_groups: HashMap<String, (Option<PathBuf>, Option<PathBuf>)> = HashMap::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "png") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let (base_name, is_talking) = if let Some(base) = stem.strip_suffix("_talking") {
                    (base.to_string(), true)
                } else {
                    (stem.to_string(), false)
                };

                // Skip embedded face names
                if base_name == "default" || base_name == "angry" {
                    info!(
                        "Skipping {} - embedded faces cannot be overridden",
                        path.display()
                    );
                    continue;
                }

                let group = file_groups.entry(base_name).or_insert((None, None));
                if is_talking {
                    group.1 = Some(path);
                } else {
                    group.0 = Some(path);
                }
            }
        }
    }

    // Load each face group
    for (name, (resting_path, talking_path)) in file_groups {
        let resting = resting_path.as_ref().and_then(|p| load_face_image(p));
        let talking = talking_path.as_ref().and_then(|p| load_face_image(p));

        match (resting, talking) {
            (Some(r), Some(t)) => {
                info!("Loaded custom face: {}", name);
                faces.insert(
                    name,
                    FaceData {
                        resting: r,
                        talking: t,
                    },
                );
            }
            (Some(r), None) => {
                // Use resting for both if no talking variant
                info!("Loaded custom face: {} (no talking variant)", name);
                faces.insert(
                    name.clone(),
                    FaceData {
                        resting: r,
                        talking: r,
                    },
                );
            }
            (None, Some(t)) => {
                // Use talking for both if no resting variant
                info!("Loaded custom face: {} (no resting variant)", name);
                faces.insert(
                    name,
                    FaceData {
                        resting: t,
                        talking: t,
                    },
                );
            }
            (None, None) => {
                // Both failed to load
            }
        }
    }
}

fn load_face_image(path: &PathBuf) -> Option<[[u8; GRID_WIDTH]; GRID_HEIGHT]> {
    let img = image::open(path).ok()?;
    convert_image_to_pattern(img, Some(path.display().to_string()))
}

fn load_face_from_bytes(bytes: &[u8]) -> Option<[[u8; GRID_WIDTH]; GRID_HEIGHT]> {
    let img = image::load_from_memory(bytes).ok()?;
    convert_image_to_pattern(img, None)
}

fn convert_image_to_pattern(
    img: image::DynamicImage,
    path_for_warning: Option<String>,
) -> Option<[[u8; GRID_WIDTH]; GRID_HEIGHT]> {
    // Check dimensions
    if img.width() != GRID_WIDTH as u32 || img.height() != GRID_HEIGHT as u32 {
        if let Some(path) = path_for_warning {
            warn!(
                "Face image {} has wrong dimensions ({}x{}, expected {}x{})",
                path,
                img.width(),
                img.height(),
                GRID_WIDTH,
                GRID_HEIGHT
            );
        }
        return None;
    }

    let mut pattern = [[0u8; GRID_WIDTH]; GRID_HEIGHT];

    for (x, y, pixel) in img.pixels() {
        let [r, g, b, a] = pixel.0;

        // Pixel is lit if:
        // - Alpha > 127 (not transparent) AND
        // - Brightness > 127 (not dark)
        let brightness = (r as u16 + g as u16 + b as u16) / 3;
        let is_lit = a > 127 && brightness > 127;

        pattern[y as usize][x as usize] = if is_lit { 1 } else { 0 };
    }

    Some(pattern)
}
