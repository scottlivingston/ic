//! Mouth and throat formant frequency modification
//!
//! SAM's voice can be altered by changing the frequencies of the
//! mouth formant (F1) and the throat formant (F2). Only the
//! vowel/diphthong and sonorant phonemes (5-29 and 48-53) are altered.

use super::tables::FREQUENCY_DATA;

/// Transform function for frequency modification
/// Uses wrapping arithmetic to match JavaScript behavior
#[inline]
fn trans(factor: u8, initial_frequency: u8) -> u8 {
    // JS: (((factor * initialFrequency) >> 8) & 0xFF) << 1
    // Use u16 for intermediate calculation to avoid overflow
    let product = (factor as u16) * (initial_frequency as u16);
    let shifted = (product >> 8) & 0xFF;
    (shifted as u8) << 1
}

/// Frequency arrays returned by set_mouth_throat
pub struct FreqData {
    pub freq1: [u8; 80],
    pub freq2: [u8; 80],
    pub freq3: [u8; 80],
}

/// Alters SAM's voice by changing the frequencies of the mouth formant (F1)
/// and the throat formant (F2).
///
/// # Arguments
/// * `mouth` - Mouth formant factor (0-255)
/// * `throat` - Throat formant factor (0-255)
///
/// # Returns
/// Three frequency arrays (freq1, freq2, freq3)
pub fn set_mouth_throat(mouth: u8, throat: u8) -> FreqData {
    // Extract individual frequency arrays from packed data
    let mut freq1 = [0u8; 80];
    let mut freq2 = [0u8; 80];
    let mut freq3 = [0u8; 80];

    for (i, &v) in FREQUENCY_DATA.iter().enumerate() {
        freq1[i] = (v & 0xFF) as u8;
        freq2[i] = ((v >> 8) & 0xFF) as u8;
        freq3[i] = ((v >> 16) & 0xFF) as u8;
    }

    // Recalculate formant frequencies 5..29 for vowels and diphthongs
    for pos in 5..30 {
        // Recalculate mouth frequency (F1)
        freq1[pos] = trans(mouth, freq1[pos]);
        // Recalculate throat frequency (F2)
        freq2[pos] = trans(throat, freq2[pos]);
    }

    // Recalculate formant frequencies 48..53 for sonorants
    for pos in 48..54 {
        // Recalculate F1 (mouth formant)
        freq1[pos] = trans(mouth, freq1[pos]);
        // Recalculate F2 (throat formant)
        freq2[pos] = trans(throat, freq2[pos]);
    }

    FreqData { freq1, freq2, freq3 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        // Default SAM values
        let freq = set_mouth_throat(128, 128);
        // Should not panic with default values
        assert_eq!(freq.freq1.len(), 80);
    }

    #[test]
    fn test_high_values() {
        // High values that caused crash in rustsam
        let freq = set_mouth_throat(220, 220);
        // Should not panic with high values
        assert_eq!(freq.freq1.len(), 80);
    }

    #[test]
    fn test_extreme_values() {
        // Maximum values
        let freq = set_mouth_throat(255, 255);
        assert_eq!(freq.freq1.len(), 80);

        // Minimum values
        let freq = set_mouth_throat(0, 0);
        assert_eq!(freq.freq1.len(), 80);
    }
}
