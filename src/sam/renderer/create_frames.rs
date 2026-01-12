//! Create frames from phonemes

use super::tables::{AMPLITUDE_DATA, SAMPLED_CONSONANT_FLAGS, STRESS_PITCH};
use crate::sam::constants::{PHONEME_PERIOD, PHONEME_QUESTION};

const RISING_INFLECTION: i16 = 255;
const FALLING_INFLECTION: i16 = 1;

/// Phoneme tuple: (phoneme_index, length, stress)
pub type PhonemeTuple = (usize, usize, usize);

/// Frame data output from create_frames
pub struct FrameData {
    pub pitches: Vec<u8>,
    pub frequency: [Vec<u8>; 3],
    pub amplitude: [Vec<u8>; 3],
    pub sampled_consonant_flag: Vec<u8>,
}

/// Create a rising or falling inflection 30 frames prior to the given position
fn add_inflection(inflection: i16, pos: usize, pitches: &mut [u8]) {
    let end = pos;
    let mut pos = if pos < 30 { 0 } else { pos - 30 };

    // Skip past any 127 values (invalid pitch markers)
    while pos < pitches.len() && pitches[pos] == 127 {
        pos += 1;
    }

    if pos >= pitches.len() {
        return;
    }

    let mut a = pitches[pos] as i16;

    while pos != end && pos < pitches.len() {
        // Add the inflection direction
        a = a.wrapping_add(inflection);

        // Set the inflection
        pitches[pos] = (a & 0xFF) as u8;

        // Skip past any 255 values
        pos += 1;
        while pos < pitches.len() && pos != end && pitches[pos] == 255 {
            pos += 1;
        }
    }
}

/// Create frames from phoneme data
///
/// The length parameter in the list corresponds to the number of frames
/// to expand the phoneme to. At the default speed, each frame represents
/// about 10 milliseconds of time.
///
/// # Arguments
/// * `pitch` - Base pitch value
/// * `tuples` - Phoneme tuples: (phoneme_index, length, stress)
/// * `freq_data` - Frequency data arrays (freq1, freq2, freq3)
///
/// # Returns
/// FrameData containing pitches, frequencies, amplitudes, and sampled consonant flags
pub fn create_frames(pitch: u8, tuples: &[PhonemeTuple], freq_data: &[Vec<u8>; 3]) -> FrameData {
    let mut pitches = Vec::new();
    let mut frequency = [Vec::new(), Vec::new(), Vec::new()];
    let mut amplitude = [Vec::new(), Vec::new(), Vec::new()];
    let mut sampled_consonant_flag = Vec::new();

    for &(phoneme, length, stress) in tuples.iter() {
        // Check for punctuation and add inflection
        if phoneme == PHONEME_PERIOD {
            add_inflection(FALLING_INFLECTION, pitches.len(), &mut pitches);
        } else if phoneme == PHONEME_QUESTION {
            add_inflection(RISING_INFLECTION, pitches.len(), &mut pitches);
        }

        // Get the stress amount (more stress = higher pitch)
        let phase1 = if stress < STRESS_PITCH.len() {
            STRESS_PITCH[stress]
        } else {
            0
        };

        // Copy from the source to the frames list for each frame
        for _ in 0..length {
            if phoneme < freq_data[0].len() {
                frequency[0].push(freq_data[0][phoneme]); // F1 frequency
                frequency[1].push(freq_data[1][phoneme]); // F2 frequency
                frequency[2].push(freq_data[2][phoneme]); // F3 frequency
            } else {
                frequency[0].push(0);
                frequency[1].push(0);
                frequency[2].push(0);
            }

            if phoneme < AMPLITUDE_DATA.len() {
                let amp = AMPLITUDE_DATA[phoneme];
                amplitude[0].push((amp & 0xFF) as u8); // F1 amplitude
                amplitude[1].push(((amp >> 8) & 0xFF) as u8); // F2 amplitude
                amplitude[2].push(((amp >> 16) & 0xFF) as u8); // F3 amplitude
            } else {
                amplitude[0].push(0);
                amplitude[1].push(0);
                amplitude[2].push(0);
            }

            if phoneme < SAMPLED_CONSONANT_FLAGS.len() {
                sampled_consonant_flag.push(SAMPLED_CONSONANT_FLAGS[phoneme]);
            } else {
                sampled_consonant_flag.push(0);
            }

            // Pitch = base pitch + stress adjustment
            pitches.push(pitch.wrapping_add(phase1));
        }
    }

    FrameData {
        pitches,
        frequency,
        amplitude,
        sampled_consonant_flag,
    }
}
