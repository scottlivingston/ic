//! SAM Renderer - converts parsed phoneme data to audio samples

mod create_frames;
mod create_transitions;
mod output_buffer;
mod process_frames;
mod set_mouth_throat;
pub mod tables;

use create_frames::create_frames;
use create_transitions::create_transitions;
use output_buffer::OutputBuffer;
use process_frames::process_frames;
use set_mouth_throat::set_mouth_throat;

/// Amplitude rescale table (decibels to linear scale)
const AMPLITUDE_RESCALE: [u8; 16] = [
    0x00, 0x01, 0x02, 0x02, 0x02, 0x03, 0x03, 0x04,
    0x04, 0x05, 0x06, 0x08, 0x09, 0x0B, 0x0D, 0x0F,
];

/// Render phonemes to audio samples
///
/// # Arguments
/// * `phonemes` - Array of (phoneme_index, length, stress) tuples
/// * `pitch` - Base pitch (default: 64)
/// * `mouth` - Mouth formant (default: 128)
/// * `throat` - Throat formant (default: 128)
/// * `speed` - Speech speed (default: 72)
/// * `singmode` - Enable sing mode (monotone pitch)
///
/// # Returns
/// Audio samples as unsigned 8-bit PCM
pub fn render(
    phonemes: &[(usize, usize, usize)],
    pitch: Option<u8>,
    mouth: Option<u8>,
    throat: Option<u8>,
    speed: Option<u8>,
    singmode: bool,
) -> Vec<u8> {
    let pitch = pitch.unwrap_or(64);
    let mouth = mouth.unwrap_or(128);
    let throat = throat.unwrap_or(128);
    let speed = speed.unwrap_or(72);

    // Set mouth and throat formants
    let freq_data = set_mouth_throat(mouth, throat);

    // Convert frequency data to the format expected by create_frames
    let freq_arrays = [
        freq_data.freq1.to_vec(),
        freq_data.freq2.to_vec(),
        freq_data.freq3.to_vec(),
    ];

    // Create frames from phonemes
    let mut frame_data = create_frames(pitch, phonemes, &freq_arrays);

    // Create transitions between phonemes
    let total_frames = create_transitions(
        &mut frame_data.pitches,
        &mut frame_data.frequency,
        &mut frame_data.amplitude,
        phonemes,
    );

    // Apply pitch contour (unless in sing mode)
    if !singmode {
        for i in 0..frame_data.pitches.len() {
            // Subtract half the frequency of formant 1 to add variety
            let f1 = frame_data.frequency[0].get(i).copied().unwrap_or(0);
            frame_data.pitches[i] = frame_data.pitches[i].wrapping_sub(f1 >> 1);
        }
    }

    // Rescale amplitude from decibels to linear
    for i in (0..frame_data.amplitude[0].len()).rev() {
        for j in 0..3 {
            let amp = frame_data.amplitude[j][i] as usize;
            frame_data.amplitude[j][i] = if amp < AMPLITUDE_RESCALE.len() {
                AMPLITUDE_RESCALE[amp]
            } else {
                0
            };
        }
    }

    // Calculate buffer size
    // Reserve 176.4 * speed samples (= 8 * speed ms) for each frame
    let total_length: usize = phonemes.iter().map(|p| p.1).sum();
    let buffer_size = ((176.4 * total_length as f64 * speed as f64) as usize).max(1);

    // Create output buffer and process frames
    let mut output = OutputBuffer::new(buffer_size);

    process_frames(
        &mut output,
        total_frames,
        speed,
        &frame_data.frequency,
        &frame_data.pitches,
        &frame_data.amplitude,
        &frame_data.sampled_consonant_flag,
    );

    output.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty() {
        let result = render(&[], None, None, None, None, false);
        assert!(result.is_empty() || result.len() <= 1);
    }

    #[test]
    fn test_render_with_high_mouth_throat() {
        // This should not panic with high mouth/throat values
        let phonemes = vec![(5, 10, 0)]; // IY phoneme
        let result = render(&phonemes, Some(60), Some(220), Some(220), Some(140), false);
        assert!(!result.is_empty());
    }
}
