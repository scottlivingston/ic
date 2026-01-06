//! Process frames to generate audio output

use super::output_buffer::OutputBuffer;
use super::tables::{SAMPLED_CONSONANT_VALUES_0, SAMPLE_TABLE};
use std::f32::consts::PI;

/// Calculate sine value for audio synthesis
#[inline]
fn sinus(x: u8) -> i32 {
    (f32::sin(2.0 * PI * (x as f32 / 256.0)) * 127.0) as i32
}

/// Render a sampled consonant
fn render_sample(
    output: &mut OutputBuffer,
    last_sample_offset: u8,
    consonant_flag: u8,
    pitch: u8,
) -> u8 {
    // Mask low three bits and subtract 1 to get conversion value
    let kind = ((consonant_flag & 7) as usize).saturating_sub(1);

    // Determine sample page (256-byte section)
    let sample_page = (kind * 256) & 0xFFFF;
    let mut off = consonant_flag & 248;

    let render_sample_inner = |output: &mut OutputBuffer, off: u8, index1: usize, value1: u8, index0: usize, value0: u8| {
        let sample_idx = sample_page + (off as usize);
        if sample_idx >= SAMPLE_TABLE.len() {
            return;
        }
        let mut sample = SAMPLE_TABLE[sample_idx];
        for _ in 0..8 {
            if (sample & 128) != 0 {
                output.write(index1, value1);
            } else {
                output.write(index0, value0);
            }
            sample <<= 1;
        }
    };

    if off == 0 {
        // Voiced phoneme: Z*, ZH, V*, DH
        let mut phase1 = ((pitch >> 4) ^ 255) & 0xFF;
        let mut current_off = last_sample_offset;

        loop {
            render_sample_inner(output, current_off, 3, 26, 4, 6);
            current_off = current_off.wrapping_add(1);
            phase1 = phase1.wrapping_add(1);
            if phase1 == 0 {
                break;
            }
        }
        return current_off;
    }

    // Unvoiced
    off = off ^ 255;
    let value0 = SAMPLED_CONSONANT_VALUES_0.get(kind).copied().unwrap_or(0);

    let mut current_off = off;
    loop {
        render_sample_inner(output, current_off, 2, 5, 1, value0);
        current_off = current_off.wrapping_add(1);
        if current_off == 0 {
            break;
        }
    }

    last_sample_offset
}

/// Process frames to generate audio output
///
/// In traditional vocal synthesis, the glottal pulse drives filters which
/// are attenuated to the frequencies of the formants.
///
/// SAM generates these formants directly with sine and rectangular waves.
/// To simulate them being driven by the glottal pulse, the waveforms are
/// reset at the beginning of each glottal pulse.
pub fn process_frames(
    output: &mut OutputBuffer,
    mut frame_count: usize,
    speed: u8,
    frequency: &[Vec<u8>; 3],
    pitches: &[u8],
    amplitude: &[Vec<u8>; 3],
    sampled_consonant_flag: &[u8],
) {
    let mut speed_counter = speed as usize;
    let mut phase1: u32 = 0;
    let mut phase2: u32 = 0;
    let mut phase3: u32 = 0;
    let mut last_sample_offset: u8 = 0;
    let mut pos: usize = 0;

    let mut glottal_pulse = pitches.get(0).copied().unwrap_or(0) as i32;
    let mut mem38 = (glottal_pulse as f32 * 0.75) as i32;

    while frame_count > 0 {
        let flags = sampled_consonant_flag.get(pos).copied().unwrap_or(0);

        // Unvoiced sampled phoneme?
        if (flags & 248) != 0 {
            let pitch = pitches.get(pos & 0xFF).copied().unwrap_or(0);
            last_sample_offset = render_sample(output, last_sample_offset, flags, pitch);
            // Skip ahead two in the phoneme buffer
            pos += 2;
            frame_count = frame_count.saturating_sub(2);
            speed_counter = speed as usize;
        } else {
            // Generate formant synthesis
            let mut ary = [0u8; 5];
            let mut p1: u32 = phase1 * 256;
            let mut p2: u32 = phase2 * 256;
            let mut p3: u32 = phase3 * 256;

            for k in 0..5 {
                let sp1 = sinus(((p1 >> 8) & 0xFF) as u8);
                let sp2 = sinus(((p2 >> 8) & 0xFF) as u8);
                let rp3: i32 = if ((p3 >> 8) & 0xFF) < 129 { -0x70 } else { 0x70 };

                let amp0 = (amplitude[0].get(pos).copied().unwrap_or(0) & 0x0F) as i32;
                let amp1 = (amplitude[1].get(pos).copied().unwrap_or(0) & 0x0F) as i32;
                let amp2 = (amplitude[2].get(pos).copied().unwrap_or(0) & 0x0F) as i32;

                let sin1 = sp1 * amp0;
                let sin2 = sp2 * amp1;
                let rect = rp3 * amp2;

                let mut mux = sin1 + sin2 + rect;
                mux /= 32;
                mux += 128; // Go from signed to unsigned amplitude

                ary[k] = mux.clamp(0, 255) as u8;

                let freq0 = frequency[0].get(pos).copied().unwrap_or(0) as u32;
                let freq1 = frequency[1].get(pos).copied().unwrap_or(0) as u32;
                let freq2 = frequency[2].get(pos).copied().unwrap_or(0) as u32;

                p1 += freq0 * 256 / 4;
                p2 += freq1 * 256 / 4;
                p3 += freq2 * 256 / 4;
            }

            output.write_ary(0, ary);

            speed_counter -= 1;
            if speed_counter == 0 {
                pos += 1;
                frame_count -= 1;
                if frame_count == 0 {
                    return;
                }
                speed_counter = speed as usize;
            }

            glottal_pulse -= 1;

            if glottal_pulse != 0 {
                // Not finished with a glottal pulse
                mem38 -= 1;

                // Within the first 75% of the glottal pulse?
                // Is the count non-zero and the sampled flag is zero?
                if mem38 != 0 || flags == 0 {
                    // Update the phase of the formants
                    let freq0 = frequency[0].get(pos).copied().unwrap_or(0) as u32;
                    let freq1 = frequency[1].get(pos).copied().unwrap_or(0) as u32;
                    let freq2 = frequency[2].get(pos).copied().unwrap_or(0) as u32;

                    phase1 = phase1.wrapping_add(freq0);
                    phase2 = phase2.wrapping_add(freq1);
                    phase3 = phase3.wrapping_add(freq2);
                    continue;
                }

                // Voiced sampled phonemes interleave the sample with the glottal pulse
                let pitch = pitches.get(pos & 0xFF).copied().unwrap_or(0);
                last_sample_offset = render_sample(output, last_sample_offset, flags, pitch);
            }
        }

        // Reset for new glottal pulse
        glottal_pulse = pitches.get(pos).copied().unwrap_or(0) as i32;
        mem38 = (glottal_pulse as f32 * 0.75) as i32;

        // Reset the formant wave generators
        phase1 = 0;
        phase2 = 0;
        phase3 = 0;
    }
}
