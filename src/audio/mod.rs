pub mod sam;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use bevy::log::{error, info};
use bevy::prelude::*;
use rodio::{OutputStreamBuilder, Sink};

use crate::face::SpeakingState;
use crate::events::{SayEvent, VolumeEvent};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        // Create SAM handle (spawns dedicated SAM thread)
        let sam_handle = sam::SamHandle::new();

        // Create audio output
        let audio_state = AudioState::new(sam_handle);

        app.insert_resource(audio_state)
            .add_systems(Update, (handle_say_events, handle_volume_events, check_speech_finished));
    }
}

#[derive(Resource)]
pub struct AudioState {
    /// Shared state to track if speech is currently playing
    is_playing: Arc<Mutex<bool>>,
    /// Shared volume level (0.0 to 1.0)
    volume: Arc<Mutex<f32>>,
    /// Channel to send audio data to the playback thread
    audio_sender: std::sync::mpsc::Sender<Vec<u8>>,
    /// Handle to the SAM TTS thread
    sam_handle: sam::SamHandle,
}

impl AudioState {
    pub fn new(sam_handle: sam::SamHandle) -> Self {
        let is_playing = Arc::new(Mutex::new(false));
        let is_playing_clone = is_playing.clone();
        let volume = Arc::new(Mutex::new(0.04f32)); // 20% on admin slider (scaled by 0.2)
        let volume_clone = volume.clone();

        let (audio_sender, audio_receiver) = std::sync::mpsc::channel::<Vec<u8>>();

        // Spawn audio playback thread
        thread::spawn(move || {
            // Create audio output stream (must be on this thread)
            let Ok(stream) = OutputStreamBuilder::open_default_stream() else {
                error!("Failed to create audio output stream");
                return;
            };

            let sink = Sink::connect_new(&stream.mixer());

            loop {
                // Wait for audio data
                match audio_receiver.recv() {
                    Ok(audio_data) => {
                        // Convert u8 PCM to f32 samples (SAM outputs unsigned 8-bit)
                        let samples: Vec<f32> = audio_data
                            .iter()
                            .map(|&b| (b as f32 - 128.0) / 128.0)
                            .collect();

                        // Create rodio source from samples
                        let source = rodio::buffer::SamplesBuffer::new(1, 22050, samples);

                        // Apply current volume setting
                        let vol = *volume_clone.lock().unwrap();
                        sink.set_volume(vol);

                        *is_playing_clone.lock().unwrap() = true;
                        sink.append(source);

                        // Poll for completion while updating volume in real-time
                        while !sink.empty() {
                            let vol = *volume_clone.lock().unwrap();
                            sink.set_volume(vol);
                            thread::sleep(Duration::from_millis(100));
                        }
                        *is_playing_clone.lock().unwrap() = false;
                    }
                    Err(_) => {
                        // Channel closed, exit thread
                        break;
                    }
                }
            }
        });

        Self {
            is_playing,
            volume,
            audio_sender,
            sam_handle,
        }
    }

    pub fn generate_and_play(&self, text: &str) -> Result<(), String> {
        let audio_data = self.sam_handle.generate_speech(text)?;
        // Set is_playing BEFORE sending to avoid race condition with check_speech_finished
        *self.is_playing.lock().unwrap() = true;
        self.audio_sender.send(audio_data).map_err(|e| {
            // Reset is_playing if send fails
            *self.is_playing.lock().unwrap() = false;
            format!("Audio thread died: {}", e)
        })
    }

    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock().unwrap()
    }

    pub fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume.clamp(0.0, 1.0);
    }
}

fn handle_say_events(
    mut say_events: MessageReader<SayEvent>,
    audio_state: Res<AudioState>,
    mut speaking_state: ResMut<SpeakingState>,
) {
    for event in say_events.read() {
        info!("Processing Say event: {}", event.msg);

        // Generate and play SAM audio
        match audio_state.generate_and_play(&event.msg) {
            Ok(()) => {
                speaking_state.start_speaking();
            }
            Err(e) => {
                error!("Failed to generate speech: {}", e);
            }
        }
    }
}

fn check_speech_finished(
    audio_state: Res<AudioState>,
    mut speaking_state: ResMut<SpeakingState>,
) {
    // Check if we were speaking and audio has finished
    if speaking_state.speaking && !audio_state.is_playing() {
        speaking_state.stop_speaking();
    }
}

fn handle_volume_events(
    mut volume_events: MessageReader<VolumeEvent>,
    audio_state: Res<AudioState>,
) {
    for event in volume_events.read() {
        info!("Setting volume to {:.0}%", event.volume * 100.0);
        audio_state.set_volume(event.volume);
    }
}
