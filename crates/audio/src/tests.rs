//! Test suite for the audio crate functionality

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;
    use project::model::Track;
    use sequencer::TriggerEvent;
    use crate::{SamplePlayer, AudioConnector, SampleEffect, SampleProcessor, AudioError};

    /// Creates a temporary WAV file that can be used for testing.
    /// Returns the path to the file and a cleanup function.
    fn create_test_wav_file(dir: &Path, name: &str) -> Result<PathBuf, std::io::Error> {
        let path = dir.join(name);
        
        // Create a simple WAV file with minimal headers to be a valid file
        // This is a minimal valid WAV with 0.1s of silence
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
        let kick_path = create_test_wav_file(&samples_dir, "kick.wav")
            .expect("Failed to create test kick.wav");
        let snare_path = create_test_wav_file(&samples_dir, "snare.wav")
            .expect("Failed to create test snare.wav");
        let hihat_path = create_test_wav_file(&samples_dir, "hihat.wav")
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
    fn test_sample_processor() {
        let mut processor = SampleProcessor::new();
        
        // Test adding effects
        processor.add_effect(0, SampleEffect::Reverse);
        processor.add_effect(0, SampleEffect::FadeIn(0.5));
        processor.add_effect(1, SampleEffect::FadeOut(0.2));
        
        // Test clearing effects
        processor.clear_effects(0);
        
        // No assertions needed as we're just testing the API doesn't panic
    }
    
    #[test]
    fn test_audio_connector_creation() {
        let (temp_dir, samples_dir, _) = setup_test_environment();
        
        // Test creating an audio connector
        let connector_result = AudioConnector::new(&samples_dir);
        
        // We might not have audio hardware, so handle both cases
        match connector_result {
            Ok(connector) => {
                assert!(!connector.is_active());
                
                // Test deactivating an inactive connector (should be a no-op)
                connector.deactivate();
                assert!(!connector.is_active());
            },
            Err(e) => {
                // On CI systems without audio, this is expected
                println!("Note: AudioConnector creation failed: {}", e);
                // No panic, test passes
            }
        }
    }
    
    #[test]
    fn test_initialize_with_tracks() {
        let (temp_dir, samples_dir, tracks) = setup_test_environment();
        
        // Test creating an audio connector
        if let Ok(connector) = AudioConnector::new(&samples_dir) {
            // Initialize with tracks
            let result = connector.initialize(&tracks);
            
            // Allow for systems without audio hardware
            match result {
                Ok(_) => {
                    // Test that the connector is now active
                    assert!(connector.is_active());
                    
                    // Test trigger processing (should be a no-op without real hardware)
                    let event = TriggerEvent { track_idx: 0, step_idx: 0 };
                    let _ = connector.process_trigger(&event);
                    
                    // Test setting volume (again, no real assertions but should not panic)
                    let _ = connector.set_track_volume(0, 0.5);
                    
                    // Test stopping all playback
                    connector.stop_all();
                    
                    // Test deactivating
                    connector.deactivate();
                    assert!(!connector.is_active());
                },
                Err(e) => {
                    println!("Note: Track initialization failed: {}", e);
                    // No panic, test passes
                }
            }
        }
    }
    
    #[test]
    fn test_connecting_to_sequencer() {
        let (temp_dir, samples_dir, tracks) = setup_test_environment();
        
        if let Ok(connector) = AudioConnector::new(&samples_dir) {
            // We can't create an actual sequencer here for unit testing,
            // but we can test the connection setup logic using the trait object pattern.
            
            // This won't be used but satisfies the API
            let dummy_sequencer = sequencer::Sequencer::new(120, vec![vec![false, false]]);
            
            // Test connection API
            let connected = connector.connect_to_sequencer(&dummy_sequencer);
            assert!(connected);
            assert!(connector.is_active());
        }
    }
    
    #[test]
    fn test_error_handling() {
        let (temp_dir, samples_dir, _) = setup_test_environment();
        
        // Test error handling when a sample is not found
        if let Ok(mut player) = SamplePlayer::new(&samples_dir) {
            let result = player.load_sample("nonexistent", "nonexistent.wav");
            assert!(result.is_err());
            
            // Extract and check the error type
            match result {
                Err(AudioError::SampleLoadError(_, _)) => {
                    // This is the expected error type
                },
                Err(e) => {
                    panic!("Expected SampleLoadError, got {:?}", e);
                }
                _ => {
                    panic!("Expected an error but got Ok");
                }
            }
        }
    }
}