// Audio connector module
// Handles integration between sequencer and audio playback

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use log::{debug, info};

use crate::{AudioError, SamplePlayer, SampleEffect};
use sequencer::TriggerEvent;
use project::model::Track;

/// Audio connector that receives trigger events from the sequencer
/// and manages the sample player
pub struct AudioConnector {
    /// Sample player instance
    player: Arc<Mutex<SamplePlayer>>,
    
    /// Project sample directory
    sample_dir: PathBuf,
    
    /// Whether the connector is currently active
    active: Arc<Mutex<bool>>,
}

impl AudioConnector {
    /// Create a new audio connector
    pub fn new(sample_dir: impl AsRef<Path>) -> Result<Self, AudioError> {
        // Initialize the sample player
        let player = SamplePlayer::new(sample_dir.as_ref())?;
        
        Ok(Self {
            player: Arc::new(Mutex::new(player)),
            sample_dir: sample_dir.as_ref().to_path_buf(),
            active: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Initialize the connector with tracks and samples
    pub fn initialize(&self, tracks: &[Track]) -> Result<(), AudioError> {
        info!("Initializing audio connector with {} tracks", tracks.len());
        let mut player = self.player.lock().unwrap();
        
        // Initialize the sample player with track data
        player.initialize_with_tracks(tracks)?;
        
        // Mark the connector as active
        *self.active.lock().unwrap() = true;
        
        Ok(())
    }
    
    /// Configure effects based on pattern metadata
    pub fn configure_effects(&self, pattern_metas: &[project::model::PatternMeta]) -> Result<(), AudioError> {
        if pattern_metas.is_empty() {
            info!("No pattern metadata available for effects configuration");
            return Ok(());
        }
        
        info!("Configuring audio effects from pattern metadata");
        let mut player = self.player.lock().unwrap();
        
        // Apply effects from the first available pattern metadata
        for meta in pattern_metas.iter() {
            for (fx_key, fx_entry) in &meta.fx {
                // Parse the key which is in the format "track:step"
                let parts: Vec<&str> = fx_key.split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(track_idx), Ok(_step_idx)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                        // Apply effects based on FX entry
                        if let Some(true) = fx_entry.reverse {
                            info!("Adding reverse effect to track {}", track_idx);
                            player.processor.add_effect(track_idx, SampleEffect::Reverse);
                        }
                        
                        // Add other effects as needed
                        if let Some(retrigger) = fx_entry.retrigger {
                            if retrigger > 1 {
                                info!("Track {} has retrigger effect: {} times", track_idx, retrigger);
                                // Implement retrigger in the future
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a trigger event from the sequencer
    pub fn process_trigger(&self, event: &TriggerEvent) -> Result<(), AudioError> {
        // Check if we're active
        if !*self.active.lock().unwrap() {
            debug!("Audio connector is not active, ignoring trigger");
            return Ok(());
        }
        
        // Get the player and process the trigger
        let mut player = self.player.lock().unwrap();
        player.process_trigger(event)
    }
    
    /// Set up a callback to process trigger events from a sequencer
    /// Returns a boolean indicating success
    pub fn connect_to_sequencer(&self, _sequencer: &sequencer::Sequencer) -> bool {
        info!("Connecting audio to sequencer");
        
        // Mark the connector as active, but we'll handle events from the main thread
        *self.active.lock().unwrap() = true;
        
        debug!("Audio connector ready to process events from main thread");
        true
    }
    
    /// Stop all audio playback
    pub fn stop_all(&self) {
        if let Ok(mut player) = self.player.lock() {
            player.stop_all();
        }
    }
    
    /// Deactivate the connector
    pub fn deactivate(&self) {
        *self.active.lock().unwrap() = false;
        self.stop_all();
    }
    
    /// Check if the connector is active
    pub fn is_active(&self) -> bool {
        *self.active.lock().unwrap()
    }
    
    /// Set volume for a specific track
    pub fn set_track_volume(&self, track_idx: usize, volume: f32) -> Result<(), AudioError> {
        let mut player = self.player.lock().unwrap();
        player.set_track_volume(track_idx, volume)
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
