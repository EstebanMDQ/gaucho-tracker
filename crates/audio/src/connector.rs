// Audio connector module
// Handles integration between sequencer and audio playback

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use log::{debug, info};
use crossbeam_channel::{bounded, Sender};
use std::collections::VecDeque;

use crate::{AudioError, SamplePlayer, SampleEffect};
use sequencer::TriggerEvent;
use project::model::Track;
use core::TrackerEvent;

/// Audio connector that receives trigger events from the sequencer
/// and manages the sample player
/// Commands for the audio system
#[derive(Debug, Clone)]
enum AudioCommand {
    TriggerSample(usize, usize),
    SetTrackVolume(usize, f32),
    StopAll,
    Deactivate,
    Initialize(Vec<Track>),
    ConfigureEffects(Vec<EffectConfig>),
}

#[derive(Debug, Clone)]
pub struct EffectConfig {
    pub track_idx: usize,
    pub effect: SampleEffect,
}

pub struct AudioConnector {
    /// Sample player instance (shared, thread-safe)
    // player: Arc<Mutex<SamplePlayer>>,
    
    /// Project sample directory
    #[allow(dead_code)]
    sample_dir: PathBuf,
    
    /// Whether the connector is currently active (thread-safe flag)
    active: Arc<Mutex<bool>>,
    
    /// Command queue for deferred processing (thread-safe queue)
    #[allow(dead_code)]
    command_queue: Arc<Mutex<VecDeque<AudioCommand>>>,
    
    /// Event bus subscription ID
    #[allow(dead_code)]
    subscription_id: Option<usize>,
    
    /// Channel sender for sending audio events to the audio thread
    message_sender: Sender<AudioCommand>,
    
    /// Background audio thread handle
    _audio_thread: Option<JoinHandle<()>>,
}

impl AudioConnector {
    /// Create a new audio connector
    pub fn new(sample_dir: impl AsRef<Path>) -> Result<Self, AudioError> {
        // let player = SamplePlayer::new(sample_dir.as_ref())?;  // <-- Just keep player here
        let sample_dir_clone = sample_dir.as_ref().to_path_buf();
        let (sender, receiver) = bounded::<AudioCommand>(100);
    
        let active = Arc::new(Mutex::new(false));
        let thread_active = active.clone();
    
        let audio_thread = thread::spawn(move || {
            debug!("Audio processing thread started");
            let mut player = match SamplePlayer::new(&sample_dir_clone) {
                Ok(player) => player,
                Err(err) => {
                    debug!("Failed to initialize audio player in thread: {:?}", err);
                    return; // Exit the thread early
                }
            };
            while let Ok(message) = receiver.recv() {
                match message {
                    AudioCommand::TriggerSample(track_idx, step_idx) => {
                        if !*thread_active.lock().unwrap() {
                            continue;
                        }
                        let trigger = TriggerEvent { track_idx, step_idx };
                        if let Err(err) = player.process_trigger(&trigger) {
                            debug!("Error processing trigger: {:?}", err);
                        }
                    },
                    AudioCommand::SetTrackVolume(track_idx, volume) => {
                        if let Err(err) = player.set_track_volume(track_idx, volume) {
                            debug!("Error setting track volume: {:?}", err);
                        }
                    },
                    AudioCommand::StopAll => {
                        player.stop_all();
                    },
                    AudioCommand::Deactivate => {
                        *thread_active.lock().unwrap() = false;
                        player.stop_all();
                        debug!("Audio thread deactivated");
                        break; // Exit the thread
                    },
                    AudioCommand::Initialize(tracks) => {
                        if let Err(err) = player.initialize_with_tracks(&tracks) {
                            debug!("Error initializing tracks: {:?}", err);
                        }
                    },
                    AudioCommand::ConfigureEffects(effects) => {
                        let effects_count = effects.len();
                        for effect_config in effects {
                            player.processor.add_effect(effect_config.track_idx, effect_config.effect);
                        }
                        debug!("Applied {} effects", effects_count);
                    },
                }
            }
    
            debug!("Audio processing thread stopped");
        });
    
        Ok(Self {
            sample_dir: sample_dir.as_ref().to_path_buf(),
            active,
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
            subscription_id: None,
            message_sender: sender,
            _audio_thread: Some(audio_thread),
        })
    }
    // pub fn new(sample_dir: impl AsRef<Path>) -> Result<Self, AudioError> {
    //     // Initialize the sample player
    //     // let player = SamplePlayer::new(sample_dir.as_ref())?;
    //     // let player_arc = Arc::new(Mutex::new(player));
        

    //     // Create a channel for audio messages
    //     let (sender, receiver) = bounded::<AudioCommand>(100);
        
    //     // Create a reference to the player for the audio thread
    //     // let thread_player = player_arc.clone();
    //     let active = Arc::new(Mutex::new(false));
    //     let thread_active = active.clone();

    //     // Spawn a thread that will process audio messages
    //     let audio_thread = thread::spawn(move || {
    //         debug!("Audio processing thread started");
    //         let player = SamplePlayer::new(sample_dir.as_ref())?;
    //         while let Ok(message) = receiver.recv() {
    //             // Process messages based on their type
    //             match message {
    //                 AudioCommand::TriggerSample(track_idx, step_idx) => {
    //                     if !*thread_active.lock().unwrap() {
    //                         continue;
    //                     }
                        
    //                     let trigger = TriggerEvent { track_idx, step_idx };
    //                     // if let Ok(mut player) = thread_player.lock() {
    //                     //     if let Err(err) = player.process_trigger(&trigger) {
    //                     //         debug!("Error processing trigger: {:?}", err);
    //                     //     }
    //                     // }
    //                     if let Err(err) = player.process_trigger(&trigger) {
    //                        debug!("Error processing trigger: {:?}", err);
    //                     }
    //                 },
    //                 AudioCommand::SetTrackVolume(track_idx, volume) => {
    //                     // if let Ok(mut player) = thread_player.lock() {
    //                     //     if let Err(err) = player.set_track_volume(track_idx, volume) {
    //                     //         debug!("Error setting track volume: {:?}", err);
    //                     //     }
    //                     // }
    //                     if let Err(err) = player.set_track_volume(track_idx, volume) {
    //                         debug!("Error setting track volume: {:?}", err);
    //                     }
    //                 },
    //                 AudioCommand::StopAll => {
    //                     // if let Ok(mut player) = thread_player.lock() {
    //                     //     player.stop_all();
    //                     // }
    //                     player.stop_all();
    //                 },
    //                 AudioCommand::Deactivate => {
    //                     *thread_active.lock().unwrap() = false;
    //                     // if let Ok(mut player) = thread_player.lock() {
    //                     //     player.stop_all();
    //                     // }
    //                     player.stop_all();
    //                     debug!("Audio thread deactivated");
    //                     break; // Exit the thread
    //                 },
    //                 AudioCommand::Initialize(tracks) => {
    //                     if let Err(err) = player.initialize_with_tracks(&tracks) {
    //                         debug!("Error initializing tracks: {:?}", err);
    //                     }
    //                 },
    //                 AudioCommand::ConfigureEffects(effects) => {
    //                     for effect_config in effects {
    //                         player.processor.add_effect(effect_config.track_idx, effect_config.effect);
    //                     }
    //                     debug!("Applied {} effects", effects.len());
    //                 },
    //             }
    //         }
            
    //         debug!("Audio processing thread stopped");
    //     });
        
    //     Ok(Self {
    //         // player: player_arc,
    //         sample_dir: sample_dir.as_ref().to_path_buf(),
    //         active,
    //         command_queue: Arc::new(Mutex::new(VecDeque::new())),
    //         subscription_id: None,
    //         message_sender: sender,
    //         _audio_thread: Some(audio_thread),
    //     })
    // }
    
    /// Initialize the connector with tracks and samples
    // pub fn initialize(&self, tracks: &[Track]) -> Result<(), AudioError> {
    //     info!("Initializing audio connector with {} tracks", tracks.len());
    //     let mut player = self.player.lock().unwrap();
        
    //     // Initialize the sample player with track data
    //     player.initialize_with_tracks(tracks)?;
        
    //     // Mark the connector as active
    //     *self.active.lock().unwrap() = true;
        
    //     Ok(())
    // }

    pub fn initialize(&self, tracks: &[Track]) -> Result<(), AudioError> {
        info!("Initializing audio connector with {} tracks", tracks.len());
    
        // Clone tracks because we're sending them into the thread
        let tracks_clone = tracks.to_vec();
    
        if let Err(_) = self.message_sender.send(AudioCommand::Initialize(tracks_clone)) {
            return Err(AudioError::PlaybackError("Failed to send initialize command to audio thread".into()));
        }
    
        *self.active.lock().unwrap() = true;
    
        Ok(())
    }

    pub fn configure_effects(&self, pattern_metas: &[project::model::PatternMeta]) -> Result<(), AudioError> {
        if pattern_metas.is_empty() {
            info!("No pattern metadata available for effects configuration");
            return Ok(());
        }
        
        info!("Configuring audio effects from pattern metadata");
    
        let mut effect_configs = Vec::new();
    
        for meta in pattern_metas.iter() {
            for (fx_key, fx_entry) in &meta.fx {
                // Parse the key which is in the format "track:step"
                let parts: Vec<&str> = fx_key.split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(track_idx), Ok(_step_idx)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                        if let Some(true) = fx_entry.reverse {
                            info!("Adding reverse effect to track {}", track_idx);
                            effect_configs.push(EffectConfig {
                                track_idx,
                                effect: SampleEffect::Reverse,
                            });
                        }
    
                        if let Some(retrigger) = fx_entry.retrigger {
                            if retrigger > 1 {
                                info!("Track {} has retrigger effect: {} times", track_idx, retrigger);
                                // Future: Add retrigger effect here
                            }
                        }
                    }
                }
            }
        }
    
        if effect_configs.is_empty() {
            return Ok(());
        }
    
        if let Err(_) = self.message_sender.send(AudioCommand::ConfigureEffects(effect_configs)) {
            return Err(AudioError::PlaybackError("Failed to send configure effects command to audio thread".into()));
        }
    
        Ok(())
    }

    /// Configure effects based on pattern metadata
    // pub fn configure_effects(&self, pattern_metas: &[project::model::PatternMeta]) -> Result<(), AudioError> {
    //     if pattern_metas.is_empty() {
    //         info!("No pattern metadata available for effects configuration");
    //         return Ok(());
    //     }
        
    //     info!("Configuring audio effects from pattern metadata");
    //     let player = self.player.lock().unwrap();
        
    //     // Apply effects from the first available pattern metadata
    //     for meta in pattern_metas.iter() {
    //         for (fx_key, fx_entry) in &meta.fx {
    //             // Parse the key which is in the format "track:step"
    //             let parts: Vec<&str> = fx_key.split(':').collect();
    //             if parts.len() == 2 {
    //                 if let (Ok(track_idx), Ok(_step_idx)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
    //                     // Apply effects based on FX entry
    //                     if let Some(true) = fx_entry.reverse {
    //                         info!("Adding reverse effect to track {}", track_idx);
    //                         player.processor.add_effect(track_idx, SampleEffect::Reverse);
    //                     }
                        
    //                     // Add other effects as needed
    //                     if let Some(retrigger) = fx_entry.retrigger {
    //                         if retrigger > 1 {
    //                             info!("Track {} has retrigger effect: {} times", track_idx, retrigger);
    //                             // Implement retrigger in the future
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
        
    //     Ok(())
    // }
    
    /// Process a trigger event from the sequencer
    pub fn process_trigger(&self, event: &TriggerEvent) -> Result<(), AudioError> {
        // Check if we're active
        if !*self.active.lock().unwrap() {
            debug!("Audio connector is not active, ignoring trigger");
            return Ok(());
        }
        
        // Send a message to the audio thread
        if let Err(_) = self.message_sender.send(AudioCommand::TriggerSample(
            event.track_idx, 
            event.step_idx
        )) {
            return Err(AudioError::PlaybackError("Failed to send trigger to audio thread".into()));
        }
        
        Ok(())
    }
    
    /// Set up a callback to process trigger events from a sequencer
    /// Returns a boolean indicating success
    pub fn connect_to_sequencer(&self, sequencer: &sequencer::Sequencer) -> bool {
        info!("Connecting audio to sequencer");
        
        // Mark the connector as active
        *self.active.lock().unwrap() = true;
        
        // Get the event bus from the sequencer
        let event_bus = sequencer.get_event_bus().clone();
        
        // Create a channel to the audio thread
        let sender = self.message_sender.clone();
        
        // Connect to the event bus
        event_bus.subscribe(move |event| {
            match event {
                TrackerEvent::StepTriggered(track_idx, step_idx) => {
                    // Send trigger message to audio thread
                    let _ = sender.send(AudioCommand::TriggerSample(*track_idx, *step_idx));
                },
                TrackerEvent::TrackVolumeChanged(track_idx, volume) => {
                    // Send volume change message to audio thread
                    let _ = sender.send(AudioCommand::SetTrackVolume(*track_idx, *volume));
                },
                _ => {
                    // Ignore other events
                }
            }
        });
        
        debug!("Audio connector ready to process events from sequencer");
        true
    }
    
    /// Stop all audio playback
    pub fn stop_all(&self) {
        let _ = self.message_sender.send(AudioCommand::StopAll);
    }
    
    /// Deactivate the connector
    pub fn deactivate(&self) {
        *self.active.lock().unwrap() = false;
        let _ = self.message_sender.send(AudioCommand::Deactivate);
    }
    
    /// Check if the connector is active
    pub fn is_active(&self) -> bool {
        *self.active.lock().unwrap()
    }
    
    /// Set volume for a specific track
    pub fn set_track_volume(&self, track_idx: usize, volume: f32) -> Result<(), AudioError> {
        if let Err(_) = self.message_sender.send(AudioCommand::SetTrackVolume(track_idx, volume)) {
            return Err(AudioError::PlaybackError("Failed to send volume change to audio thread".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use project::model::Track;
    use sequencer::TriggerEvent;
    
    #[test]
    fn test_connector_creation() {
        // This test won't actually interact with audio devices
        // We just ensure it can be created without errors
        if let Ok(connector) = AudioConnector::new(Path::new("./samples")) {
            assert!(!connector.is_active());
        }
    }
    
    /// Creates a temporary WAV file that can be used for testing.
    fn create_test_wav_file(dir: &Path, name: &str) -> Result<PathBuf, std::io::Error> {
        let path = dir.join(name);
        
        // Create a simple WAV file with minimal headers to be a valid file
        let wav_header: [u8; 44] = [
            // RIFF header
            b'R', b'I', b'F', b'F',
            // File size - 8 (36 bytes for rest of the header + no data)
            36, 0, 0, 0,
            // WAVE header
            b'W', b'A', b'V', b'E',
            // fmt chunk marker
            b'f', b'm', b't', b' ',
            // Length of fmt data (16 bytes)
            16, 0, 0, 0,
            // Format type (1 = PCM)
            1, 0,
            // Channels (1 = mono)
            1, 0,
            // Sample rate (44100Hz)
            0x44, 0xAC, 0, 0,
            // Byte rate (44100 * 1 * 16/8)
            0x88, 0x58, 0x01, 0,
            // Block align ((16/8) * 1)
            2, 0,
            // Bits per sample (16)
            16, 0,
            // data chunk marker
            b'd', b'a', b't', b'a',
            // Data size (0 bytes)
            0, 0, 0, 0
        ];
        
        let mut file = File::create(&path)?;
        file.write_all(&wav_header)?;
        
        Ok(path)
    }
    
    /// Creates a temporary test environment with sample files
    fn setup_test_environment() -> (tempfile::TempDir, PathBuf, Vec<Track>) {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let samples_dir = temp_dir.path().join("samples");
        fs::create_dir_all(&samples_dir).expect("Failed to create samples directory");
        
        // Create test WAV files
        let _kick_path = create_test_wav_file(&samples_dir, "kick.wav")
            .expect("Failed to create test kick.wav");
        let _snare_path = create_test_wav_file(&samples_dir, "snare.wav")
            .expect("Failed to create test snare.wav");
        let _hihat_path = create_test_wav_file(&samples_dir, "hihat.wav")
            .expect("Failed to create test hihat.wav");
        
        // Create track configurations
        let tracks = vec![
            Track {
                name: "Kick".to_string(),
                sample: "kick.wav".to_string(),
                volume: 1.0,
            },
            Track {
                name: "Snare".to_string(),
                sample: "snare.wav".to_string(),
                volume: 0.8,
            },
            Track {
                name: "HiHat".to_string(),
                sample: "hihat.wav".to_string(),
                volume: 0.7,
            }
        ];
        
        (temp_dir, samples_dir, tracks)
    }
    
    #[test]
    fn test_connector_lifecycle() {
        let (_temp_dir, samples_dir, tracks) = setup_test_environment();
        
        // Test lifecycle: create -> initialize -> process triggers -> deactivate
        if let Ok(connector) = AudioConnector::new(&samples_dir) {
            // Test initial state
            assert!(!connector.is_active());
            
            // Test initialization
            let result = connector.initialize(&tracks);
            if result.is_ok() {
                assert!(connector.is_active());
                
                // Test trigger processing
                let event = TriggerEvent { track_idx: 0, step_idx: 0 };
                let _ = connector.process_trigger(&event);
                
                // Test volume control
                let _ = connector.set_track_volume(0, 0.75);
                
                // Test deactivation
                connector.deactivate();
                assert!(!connector.is_active());
                
                // Test that trigger processing does nothing when inactive
                let result = connector.process_trigger(&event);
                assert!(result.is_ok());
            }
        }
    }
    
    #[test]
    fn test_process_trigger_with_invalid_track() {
        let (_temp_dir, samples_dir, tracks) = setup_test_environment();
        
        if let Ok(connector) = AudioConnector::new(&samples_dir) {
            let _ = connector.initialize(&tracks);
            
            // Test with a track index that doesn't exist
            let event = TriggerEvent { track_idx: 999, step_idx: 0 };
            let result = connector.process_trigger(&event);
            
            // Should succeed without error (just won't play anything)
            assert!(result.is_ok());
        }
    }
    
    #[test]
    fn test_connector_sequencer_integration() {
        let (_temp_dir, samples_dir, tracks) = setup_test_environment();
        
        if let Ok(connector) = AudioConnector::new(&samples_dir) {
            let _ = connector.initialize(&tracks);
            
            // Create a minimal sequencer for testing
            let dummy_pattern = vec![vec![true, false]];
            let dummy_sequencer = sequencer::Sequencer::new(120, dummy_pattern);
            
            // Test connecting to sequencer
            let success = connector.connect_to_sequencer(&dummy_sequencer);
            assert!(success);
            assert!(connector.is_active());
            
            // Clean up
            connector.deactivate();
        }
    }
}
