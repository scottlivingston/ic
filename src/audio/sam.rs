use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use rustsam::{parser, reciter, renderer};

// SAM voice settings - higher pitch = higher voice
const SAM_SPEED: u8 = 110; // Slower = more robotic (original IC used 140 in sam-js)
const SAM_PITCH: u8 = 38; // Lower = higher voice
const SAM_MOUTH: u8 = 180; // Affects formants
const SAM_THROAT: u8 = 180; // Affects formants

/// Handle to communicate with the SAM thread
/// Wrapped in Mutex to satisfy Bevy's Sync requirement for Resources
pub struct SamHandle {
    request_tx: Mutex<Sender<String>>,
    response_rx: Mutex<Receiver<Result<Vec<u8>, String>>>,
}

impl SamHandle {
    /// Spawn a dedicated thread for all SAM operations
    pub fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<String>();
        let (response_tx, response_rx) = mpsc::channel::<Result<Vec<u8>, String>>();

        thread::spawn(move || {
            // Process requests on this thread
            while let Ok(text) = request_rx.recv() {
                let result = generate_speech_internal(&text);
                if response_tx.send(result).is_err() {
                    break;
                }
            }
        });

        Self {
            request_tx: Mutex::new(request_tx),
            response_rx: Mutex::new(response_rx),
        }
    }

    /// Generate SAM audio for the given text
    /// Returns unsigned 8-bit mono PCM at 22050 Hz
    pub fn generate_speech(&self, text: &str) -> Result<Vec<u8>, String> {
        let tx = self.request_tx.lock().unwrap();
        if tx.send(text.to_string()).is_err() {
            return Err("SAM thread died".to_string());
        }
        drop(tx);

        let rx = self.response_rx.lock().unwrap();
        rx.recv().map_err(|_| "SAM thread died".to_string())?
    }
}

fn generate_speech_internal(text: &str) -> Result<Vec<u8>, String> {
    // Step 1: Convert text to phonemes
    let phonemes = reciter::text_to_phonemes(text)
        .map_err(|e| format!("Failed to convert text to phonemes: {:?}", e))?;

    // Step 2: Parse phonemes
    let parsed = parser::parse_phonemes(&phonemes)
        .map_err(|e| format!("Failed to parse phonemes: {:?}", e))?;

    // Step 3: Render audio
    // Parameters: phonemes, speed, pitch, mouth, throat, sing_mode
    // Returns Vec<u8> directly (unsigned 8-bit PCM)
    let samples = renderer::render(&parsed, SAM_PITCH, SAM_MOUTH, SAM_THROAT, SAM_SPEED, false);

    Ok(samples)
}
