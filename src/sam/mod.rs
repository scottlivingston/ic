//! SAM (Software Automatic Mouth) - Text-to-speech synthesis
//!
//! This is a Rust port of the sam-js library, providing accurate parameter handling
//! for mouth, throat, speed, and pitch settings.
//!
//! # Pipeline
//! ```text
//! Text → [Reciter] → Phonemes → [Parser] → Parsed Data → [Renderer] → Audio Samples
//! ```

pub mod constants;
pub mod parser;
pub mod reciter;
pub mod renderer;

// Re-export main types and functions for convenience
pub use parser::ParsedPhoneme;

/// Convert text to phonemes using the reciter
///
/// # Arguments
/// * `text` - English text to convert
///
/// # Returns
/// Result containing phoneme string or error message
pub fn text_to_phonemes(text: &str) -> Result<String, String> {
    reciter::text_to_phonemes(text).ok_or_else(|| "Failed to convert text to phonemes".to_string())
}

/// Parse phonemes into data suitable for rendering
///
/// # Arguments
/// * `phonemes` - Phoneme string from reciter or manual input
///
/// # Returns
/// Result containing parsed phoneme data or error message
pub fn parse_phonemes(phonemes: &str) -> Result<Vec<ParsedPhoneme>, String> {
    parser::parse(phonemes).ok_or_else(|| "Failed to parse phonemes".to_string())
}

/// Render parsed phonemes to audio samples
///
/// # Arguments
/// * `phonemes` - Parsed phoneme data
/// * `pitch` - Voice pitch (default 64, lower = higher voice)
/// * `mouth` - Mouth formant (default 128, affects F1)
/// * `throat` - Throat formant (default 128, affects F2)
/// * `speed` - Speech speed (default 72, higher = faster)
/// * `singmode` - Enable monotone pitch for singing
///
/// # Returns
/// Audio samples as unsigned 8-bit PCM at 22050 Hz
pub fn render(
    phonemes: &[ParsedPhoneme],
    pitch: u8,
    mouth: u8,
    throat: u8,
    speed: u8,
    singmode: bool,
) -> Vec<u8> {
    renderer::render(
        phonemes,
        Some(pitch),
        Some(mouth),
        Some(throat),
        Some(speed),
        singmode,
    )
}

/// Convert text directly to audio samples (convenience function)
///
/// # Arguments
/// * `text` - English text to speak
/// * `pitch` - Voice pitch (default 64)
/// * `mouth` - Mouth formant (default 128)
/// * `throat` - Throat formant (default 128)
/// * `speed` - Speech speed (default 72)
///
/// # Returns
/// Result containing audio samples or error message
#[allow(dead_code)]
pub fn say(text: &str, pitch: u8, mouth: u8, throat: u8, speed: u8) -> Result<Vec<u8>, String> {
    let phonemes_str = text_to_phonemes(text)?;
    let parsed = parse_phonemes(&phonemes_str)?;
    Ok(render(&parsed, pitch, mouth, throat, speed, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        // Test the full pipeline with default settings
        let result = say("HELLO", 64, 128, 128, 72);
        assert!(result.is_ok());
        let samples = result.unwrap();
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_extreme_mouth_throat() {
        // This is the key test - rustsam crashes with these values
        let result = say("TEST", 60, 220, 220, 140);
        assert!(result.is_ok());
        let samples = result.unwrap();
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_target_voice_params() {
        // Target voice: speed=140, pitch=60, mouth=220, throat=220
        let result = say("HELLO WORLD", 60, 220, 220, 140);
        assert!(result.is_ok());
        let samples = result.unwrap();
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_reciter() {
        let result = text_to_phonemes("HELLO");
        assert!(result.is_ok());
        let phonemes = result.unwrap();
        assert!(!phonemes.is_empty());
    }

    #[test]
    fn test_parser() {
        // Test with raw phoneme input
        let result = parse_phonemes("HEHLOW");
        assert!(result.is_ok());
    }

    #[test]
    fn test_long_phrases() {
        // These long phrases previously caused distortion or silence due to buffer overflow
        let phrases = [
            "Follow me to the check-in terminal and get ready for an adventure in The Mall!",
            "You need to sign in using this terminal!",
            "Welcome to the Mall Station!",
            "Oh my. What a day!",
        ];

        for phrase in phrases {
            let result = say(phrase, 60, 220, 220, 140);
            assert!(result.is_ok(), "Failed on phrase: {}", phrase);
            let samples = result.unwrap();
            assert!(!samples.is_empty(), "Empty samples for phrase: {}", phrase);
            // Long phrases should produce significant audio
            assert!(samples.len() > 1000, "Too few samples for phrase: {} (got {})", phrase, samples.len());
        }
    }
}
