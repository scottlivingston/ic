//! Asset generator for pre-rendered face images
//!
//! Run with: cargo run --features generate-assets
//!
//! Generates PNG assets for each face state with high-quality 2-pass Gaussian blur.
//! These pre-rendered assets are used by SpriteFacePlugin for efficient rendering
//! on devices with software rendering (like Pi Zero 2 W).

#![cfg(feature = "generate-assets")]

use image::{Rgba, RgbaImage};
use std::fs;
use std::path::Path;

// Grid dimensions matching pixel_face
const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 30;
const PIXEL_SIZE: u32 = 16;
const OUTPUT_WIDTH: u32 = GRID_WIDTH as u32 * PIXEL_SIZE;
const OUTPUT_HEIGHT: u32 = GRID_HEIGHT as u32 * PIXEL_SIZE;

// Colors from shader
const FACE_COLOR: [u8; 4] = [204, 255, 238, 255]; // #ccffee
const GLOW_COLOR: [u8; 4] = [0, 255, 170, 255]; // #00ffaa

// Blur parameters
const GLOW_BLUR_RADIUS: u32 = 32;
const GLOW_INTENSITY: f32 = 0.8;

// Scanline parameters
const SCANLINE_SPACING: u32 = 4; // Every 4th pixel
const SCANLINE_OPACITY: f32 = 0.5;

// ============================================================================
// Face Patterns (from pixel_face/mod.rs)
// ============================================================================

#[rustfmt::skip]
const FACE_DEFAULT: [[u8; GRID_WIDTH]; GRID_HEIGHT] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,1,1,1,1,1,1,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
];

#[rustfmt::skip]
const FACE_TALKING: [[u8; GRID_WIDTH]; GRID_HEIGHT] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,1,1,1,1,1,1,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,1,1,1,1,1,1,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
];

#[rustfmt::skip]
const FACE_ANGRY: [[u8; GRID_WIDTH]; GRID_HEIGHT] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
];

#[rustfmt::skip]
const FACE_ANGRY_TALKING: [[u8; GRID_WIDTH]; GRID_HEIGHT] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
];

// ============================================================================
// Image Generation
// ============================================================================

/// Render a face pattern to an RGBA image
fn render_face_base(pattern: &[[u8; GRID_WIDTH]; GRID_HEIGHT]) -> RgbaImage {
    let mut img = RgbaImage::new(OUTPUT_WIDTH, OUTPUT_HEIGHT);

    for row in 0..GRID_HEIGHT {
        for col in 0..GRID_WIDTH {
            if pattern[row][col] != 0 {
                // Fill the virtual pixel
                let x_start = col as u32 * PIXEL_SIZE;
                let y_start = row as u32 * PIXEL_SIZE;

                for dy in 0..PIXEL_SIZE {
                    for dx in 0..PIXEL_SIZE {
                        img.put_pixel(x_start + dx, y_start + dy, Rgba(FACE_COLOR));
                    }
                }
            }
        }
    }

    img
}

// ============================================================================
// Gaussian Blur
// ============================================================================

/// Generate a 1D Gaussian kernel
fn gaussian_kernel(radius: u32) -> Vec<f32> {
    let sigma = radius as f32 / 3.0;
    let size = (radius * 2 + 1) as usize;
    let mut kernel = Vec::with_capacity(size);
    let mut sum = 0.0;

    for i in 0..size {
        let x = i as f32 - radius as f32;
        let weight = (-x * x / (2.0 * sigma * sigma)).exp();
        kernel.push(weight);
        sum += weight;
    }

    // Normalize
    for w in &mut kernel {
        *w /= sum;
    }

    kernel
}

/// Apply horizontal Gaussian blur pass
fn blur_horizontal(src: &RgbaImage, kernel: &[f32]) -> RgbaImage {
    let (width, height) = src.dimensions();
    let radius = (kernel.len() / 2) as i32;
    let mut dst = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut a = 0.0f32;

            for (i, &weight) in kernel.iter().enumerate() {
                let sample_x = (x as i32 + i as i32 - radius).clamp(0, width as i32 - 1) as u32;
                let pixel = src.get_pixel(sample_x, y);

                r += pixel[0] as f32 * weight;
                g += pixel[1] as f32 * weight;
                b += pixel[2] as f32 * weight;
                a += pixel[3] as f32 * weight;
            }

            dst.put_pixel(
                x,
                y,
                Rgba([r as u8, g as u8, b as u8, a as u8]),
            );
        }
    }

    dst
}

/// Apply vertical Gaussian blur pass
fn blur_vertical(src: &RgbaImage, kernel: &[f32]) -> RgbaImage {
    let (width, height) = src.dimensions();
    let radius = (kernel.len() / 2) as i32;
    let mut dst = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut a = 0.0f32;

            for (i, &weight) in kernel.iter().enumerate() {
                let sample_y = (y as i32 + i as i32 - radius).clamp(0, height as i32 - 1) as u32;
                let pixel = src.get_pixel(x, sample_y);

                r += pixel[0] as f32 * weight;
                g += pixel[1] as f32 * weight;
                b += pixel[2] as f32 * weight;
                a += pixel[3] as f32 * weight;
            }

            dst.put_pixel(
                x,
                y,
                Rgba([r as u8, g as u8, b as u8, a as u8]),
            );
        }
    }

    dst
}

/// Apply 2-pass Gaussian blur
fn gaussian_blur(src: &RgbaImage, radius: u32) -> RgbaImage {
    let kernel = gaussian_kernel(radius);
    let horizontal = blur_horizontal(src, &kernel);
    blur_vertical(&horizontal, &kernel)
}

/// Generate glow layer from base image
fn generate_glow_layer(base: &RgbaImage, blur_radius: u32, intensity: f32) -> RgbaImage {
    let (width, height) = base.dimensions();

    // Create a version with glow color instead of face color
    let mut glow_base = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let pixel = base.get_pixel(x, y);
            if pixel[3] > 0 {
                // Use glow color with original alpha
                glow_base.put_pixel(
                    x,
                    y,
                    Rgba([GLOW_COLOR[0], GLOW_COLOR[1], GLOW_COLOR[2], pixel[3]]),
                );
            }
        }
    }

    // Apply blur
    let blurred = gaussian_blur(&glow_base, blur_radius);

    // Apply intensity scaling to alpha
    let mut result = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let pixel = blurred.get_pixel(x, y);
            let scaled_alpha = ((pixel[3] as f32) * intensity).min(255.0) as u8;
            result.put_pixel(
                x,
                y,
                Rgba([pixel[0], pixel[1], pixel[2], scaled_alpha]),
            );
        }
    }

    result
}

/// Generate scanlines overlay texture
fn generate_scanlines(width: u32, height: u32, spacing: u32, opacity: f32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    let alpha = (255.0 * opacity) as u8;

    for y in 0..height {
        // Every Nth row is a dark scanline
        if y % spacing == 0 {
            for x in 0..width {
                // Black with alpha for darkening effect
                img.put_pixel(x, y, Rgba([0, 0, 0, alpha]));
            }
        }
        // Other rows are transparent (default)
    }

    img
}

/// Generate grid overlay texture
fn generate_grid(width: u32, height: u32, cell_size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    // Cyan/teal grid color matching CrtPlugin shader: #00ffaa
    let line_color = Rgba([0u8, 255u8, 170u8, 255u8]);

    // Draw vertical lines
    for x in (0..width).step_by(cell_size as usize) {
        for y in 0..height {
            img.put_pixel(x, y, line_color);
        }
    }

    // Draw horizontal lines
    for y in (0..height).step_by(cell_size as usize) {
        for x in 0..width {
            img.put_pixel(x, y, line_color);
        }
    }

    img
}

// ============================================================================
// Main Entry Point
// ============================================================================

/// Face state configuration for asset generation
struct FaceState {
    name: &'static str,
    pattern: &'static [[u8; GRID_WIDTH]; GRID_HEIGHT],
}

const FACE_STATES: &[FaceState] = &[
    FaceState {
        name: "default",
        pattern: &FACE_DEFAULT,
    },
    FaceState {
        name: "talking",
        pattern: &FACE_TALKING,
    },
    FaceState {
        name: "angry",
        pattern: &FACE_ANGRY,
    },
    FaceState {
        name: "angry_talking",
        pattern: &FACE_ANGRY_TALKING,
    },
];

/// Generate all face assets
pub fn generate_all_assets() {
    let output_dir = Path::new("src/assets/sprites");

    // Create output directory
    fs::create_dir_all(output_dir).expect("Failed to create assets/sprites directory");

    println!("Generating face assets to {:?}", output_dir);
    println!("  Resolution: {}x{}", OUTPUT_WIDTH, OUTPUT_HEIGHT);
    println!("  Pixel size: {}", PIXEL_SIZE);
    println!("  Blur radius: {}", GLOW_BLUR_RADIUS);

    // Generate individual face images first (needed for atlas)
    let mut base_images = Vec::new();
    let mut glow_images = Vec::new();

    for state in FACE_STATES {
        println!("\nGenerating {} face...", state.name);

        // Generate base image
        let base = render_face_base(state.pattern);
        let base_path = output_dir.join(format!("{}_base.png", state.name));
        base.save(&base_path)
            .unwrap_or_else(|e| panic!("Failed to save {}: {}", base_path.display(), e));
        println!("  Saved: {}", base_path.display());

        // Generate glow layer
        let glow = generate_glow_layer(&base, GLOW_BLUR_RADIUS, GLOW_INTENSITY);
        let glow_path = output_dir.join(format!("{}_glow.png", state.name));
        glow.save(&glow_path)
            .unwrap_or_else(|e| panic!("Failed to save {}: {}", glow_path.display(), e));
        println!("  Saved: {}", glow_path.display());

        base_images.push(base);
        glow_images.push(glow);
    }

    // Generate face atlases (2x2 grid: 1280x960)
    generate_face_atlases(output_dir, &base_images, &glow_images);

    // Generate scanlines overlay (only need one)
    println!("\nGenerating scanlines overlay...");
    let scanlines = generate_scanlines(OUTPUT_WIDTH, OUTPUT_HEIGHT, SCANLINE_SPACING, SCANLINE_OPACITY);
    let scanlines_path = output_dir.join("scanlines.png");
    scanlines
        .save(&scanlines_path)
        .expect("Failed to save scanlines.png");
    println!("  Saved: {}", scanlines_path.display());

    // Generate grid overlay (40px cells to match CrtPlugin shader)
    println!("\nGenerating grid overlay...");
    let grid = generate_grid(OUTPUT_WIDTH, OUTPUT_HEIGHT, 40);
    let grid_path = output_dir.join("grid.png");
    grid.save(&grid_path).expect("Failed to save grid.png");
    println!("  Saved: {}", grid_path.display());

    println!("\nAsset generation complete!");
    println!("Generated {} face states + atlases + scanlines + grid overlays", FACE_STATES.len());
}

/// Generate face atlases by combining individual face images into 2x2 grids
/// Layout (index mapping):
///   0 = default (top-left)     1 = talking (top-right)
///   2 = angry (bottom-left)    3 = angry_talking (bottom-right)
fn generate_face_atlases(output_dir: &Path, base_images: &[RgbaImage], glow_images: &[RgbaImage]) {
    use image::imageops;

    const ATLAS_WIDTH: u32 = OUTPUT_WIDTH * 2;  // 1280
    const ATLAS_HEIGHT: u32 = OUTPUT_HEIGHT * 2; // 960

    println!("\nGenerating face atlases ({}x{})...", ATLAS_WIDTH, ATLAS_HEIGHT);

    // Create base atlas
    let mut base_atlas = RgbaImage::new(ATLAS_WIDTH, ATLAS_HEIGHT);
    for (i, img) in base_images.iter().enumerate() {
        let col = (i % 2) as u32;
        let row = (i / 2) as u32;
        let x = col * OUTPUT_WIDTH;
        let y = row * OUTPUT_HEIGHT;
        imageops::overlay(&mut base_atlas, img, x as i64, y as i64);
    }
    let base_atlas_path = output_dir.join("face_base_atlas.png");
    base_atlas.save(&base_atlas_path).expect("Failed to save face_base_atlas.png");
    println!("  Saved: {}", base_atlas_path.display());

    // Create glow atlas
    let mut glow_atlas = RgbaImage::new(ATLAS_WIDTH, ATLAS_HEIGHT);
    for (i, img) in glow_images.iter().enumerate() {
        let col = (i % 2) as u32;
        let row = (i / 2) as u32;
        let x = col * OUTPUT_WIDTH;
        let y = row * OUTPUT_HEIGHT;
        imageops::overlay(&mut glow_atlas, img, x as i64, y as i64);
    }
    let glow_atlas_path = output_dir.join("face_glow_atlas.png");
    glow_atlas.save(&glow_atlas_path).expect("Failed to save face_glow_atlas.png");
    println!("  Saved: {}", glow_atlas_path.display());
}
