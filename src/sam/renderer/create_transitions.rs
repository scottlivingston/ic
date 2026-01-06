//! Create transitions between phonemes

use super::create_frames::PhonemeTuple;
use super::tables::{BLEND_RANK, IN_BLEND_LENGTH, OUT_BLEND_LENGTH};

/// Create smooth transitions between phonemes
///
/// Linear transitions are created to smoothly connect each phoneme.
/// The transition is spread between the ending frames of the old phoneme
/// (outBlendLength) and the beginning frames of the new phoneme (inBlendLength).
///
/// # Arguments
/// * `pitches` - Pitch values for each frame
/// * `frequency` - Frequency values for each formant
/// * `amplitude` - Amplitude values for each formant
/// * `tuples` - Phoneme tuples
///
/// # Returns
/// Total number of frames
pub fn create_transitions(
    pitches: &mut [u8],
    frequency: &mut [Vec<u8>; 3],
    amplitude: &mut [Vec<u8>; 3],
    tuples: &[PhonemeTuple],
) -> usize {
    // Collect all tables for indexed access
    // 0=pitches, 1=freq1, 2=freq2, 3=freq3, 4=amp1, 5=amp2, 6=amp3

    let read = |table: usize, pos: usize, pitches: &[u8], frequency: &[Vec<u8>; 3], amplitude: &[Vec<u8>; 3]| -> i32 {
        match table {
            0 => pitches.get(pos).copied().unwrap_or(0) as i32,
            1 => frequency[0].get(pos).copied().unwrap_or(0) as i32,
            2 => frequency[1].get(pos).copied().unwrap_or(0) as i32,
            3 => frequency[2].get(pos).copied().unwrap_or(0) as i32,
            4 => amplitude[0].get(pos).copied().unwrap_or(0) as i32,
            5 => amplitude[1].get(pos).copied().unwrap_or(0) as i32,
            6 => amplitude[2].get(pos).copied().unwrap_or(0) as i32,
            _ => 0,
        }
    };

    let write = |table: usize, pos: usize, val: i32, pitches: &mut [u8], frequency: &mut [Vec<u8>; 3], amplitude: &mut [Vec<u8>; 3]| {
        let val = (val & 0xFF) as u8;
        match table {
            0 => {
                if pos < pitches.len() {
                    pitches[pos] = val;
                }
            }
            1 => {
                if pos < frequency[0].len() {
                    frequency[0][pos] = val;
                }
            }
            2 => {
                if pos < frequency[1].len() {
                    frequency[1][pos] = val;
                }
            }
            3 => {
                if pos < frequency[2].len() {
                    frequency[2][pos] = val;
                }
            }
            4 => {
                if pos < amplitude[0].len() {
                    amplitude[0][pos] = val;
                }
            }
            5 => {
                if pos < amplitude[1].len() {
                    amplitude[1][pos] = val;
                }
            }
            6 => {
                if pos < amplitude[2].len() {
                    amplitude[2][pos] = val;
                }
            }
            _ => {}
        }
    };

    // Linearly interpolate values
    let interpolate = |width: i32, table: usize, frame: usize, change: i32, pitches: &mut [u8], frequency: &mut [Vec<u8>; 3], amplitude: &mut [Vec<u8>; 3]| {
        if width <= 0 {
            return;
        }

        let sign = change < 0;
        let remainder = change.abs() % width;
        let div = change / width;

        let mut error = 0i32;
        let mut pos = width;
        let mut frame = frame;

        while pos > 1 {
            pos -= 1;
            let mut val = read(table, frame, pitches, frequency, amplitude) + div;
            error += remainder;

            if error >= width {
                error -= width;
                if sign {
                    val -= 1;
                } else if val != 0 {
                    val += 1;
                }
            }

            frame += 1;
            write(table, frame, val, pitches, frequency, amplitude);
        }
    };

    let mut boundary = 0usize;

    for pos in 0..tuples.len().saturating_sub(1) {
        let (phoneme, length, _) = tuples[pos];
        let (next_phoneme, next_length, _) = tuples[pos + 1];

        // Get the ranking of each phoneme
        let rank = BLEND_RANK.get(phoneme).copied().unwrap_or(0);
        let next_rank = BLEND_RANK.get(next_phoneme).copied().unwrap_or(0);

        let (out_blend_frames, in_blend_frames) = if rank == next_rank {
            // Same rank, use out blend lengths from each phoneme
            (
                OUT_BLEND_LENGTH.get(phoneme).copied().unwrap_or(0) as usize,
                OUT_BLEND_LENGTH.get(next_phoneme).copied().unwrap_or(0) as usize,
            )
        } else if rank < next_rank {
            // Next phoneme is stronger, use its blend lengths
            (
                IN_BLEND_LENGTH.get(next_phoneme).copied().unwrap_or(0) as usize,
                OUT_BLEND_LENGTH.get(next_phoneme).copied().unwrap_or(0) as usize,
            )
        } else {
            // Current phoneme is stronger, use its blend lengths (swapped)
            (
                OUT_BLEND_LENGTH.get(phoneme).copied().unwrap_or(0) as usize,
                IN_BLEND_LENGTH.get(phoneme).copied().unwrap_or(0) as usize,
            )
        };

        boundary += length;

        let trans_end = boundary + in_blend_frames;
        let trans_start = boundary.saturating_sub(out_blend_frames);
        let trans_length = out_blend_frames + in_blend_frames;

        // Check if transition length is valid (not too large)
        if ((trans_length.wrapping_sub(2)) & 128) == 0 {
            // Pitch interpolation from middle of current phoneme to middle of next
            let cur_width = length / 2;
            let next_width = next_length / 2;

            let pitch_end = boundary + next_width;
            let pitch_start = boundary.saturating_sub(cur_width);

            if pitch_end < pitches.len() && pitch_start < pitches.len() {
                let pitch_change = pitches[pitch_end] as i32 - pitches[pitch_start] as i32;
                interpolate(
                    (cur_width + next_width) as i32,
                    0,
                    trans_start,
                    pitch_change,
                    pitches,
                    frequency,
                    amplitude,
                );
            }

            // Interpolate frequency and amplitude tables
            for table in 1..7 {
                let value = read(table, trans_end, pitches, frequency, amplitude)
                    - read(table, trans_start, pitches, frequency, amplitude);
                interpolate(
                    trans_length as i32,
                    table,
                    trans_start,
                    value,
                    pitches,
                    frequency,
                    amplitude,
                );
            }
        }
    }

    // Add the length of last phoneme
    if let Some(&(_, last_length, _)) = tuples.last() {
        boundary + last_length
    } else {
        boundary
    }
}
