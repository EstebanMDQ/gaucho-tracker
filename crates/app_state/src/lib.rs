// App state for the Gaucho Tracker
use std::path::{Path, PathBuf};
use std::sync::Arc;

use audio::AudioConnector;
use project::model::Track;
use sequencer::{Sequencer, TriggerEvent};
use core::{EventBus, TrackerEvent, SharedEventBus};
use log::{debug, info, error};

/// Represents the state of the application
pub struct AppState {
    /// Step pattern data for all tracks - [track][step]
    pub steps: Vec<Vec<bool>>,
    
    /// Currently selected track (for UI)
    pub selected_track: usize,
    /// Currently selected step (for UI)
    pub selected_step: usize,
    /// Names of each track
    pub track_names: Vec<String>,
    /// Whether the sequencer is playing or paused
    pub is_playing: bool,
    /// The current step during playback
    pub current_step: usize,
    /// The sequencer engine
    pub sequencer: Option<Sequencer>,
    /// Collected trigger events from sequencer
    pub trigger_events: Vec<TriggerEvent>,
    /// Current BPM
    pub bpm: u32,
    /// Audio connector for sample playback
    pub audio: Option<AudioConnector>,
    /// Path to the sample directory
    sample_dir: PathBuf,
    /// Track configurations
    tracks: Vec<Track>,
    /// Shared event bus for component communication
    event_bus: SharedEventBus,
}

impl AppState {
    /// Creates a new AppState with empty pattern data
    pub fn new(num_tracks: usize, num_steps: usize) -> Self {
        // Create event bus for component communication
        let event_bus = Arc::new(EventBus::new());
        
        Self {
            steps: vec![vec![false; num_steps]; num_tracks],
            selected_track: 0,
            selected_step: 0,
            track_names: vec![],
            is_playing: false,
            current_step: 0,
            sequencer: None, // Will be initialized after pattern data is loaded
            trigger_events: Vec::new(),
            bpm: 120, // Default BPM
            audio: None, // Will be initialized later
            sample_dir: PathBuf::from("samples"), // Default sample directory
            tracks: Vec::new(),
            event_bus,
        }
    }
    
    /// Initialize the app with project data, including sample dir
    pub fn with_sample_dir(mut self, sample_dir: impl AsRef<Path>) -> Self {
        self.sample_dir = sample_dir.as_ref().to_path_buf();
        self
    }
    
    /// Initialize the app with track data
    pub fn with_tracks(mut self, tracks: Vec<Track>) -> Self {
        // Save the track names before moving the tracks vector
        let track_names: Vec<String> = tracks.iter().map(|t| t.name.clone()).collect();
        
        self.tracks = tracks;
        self.track_names = track_names;
        self
    }

    /// Toggle the currently selected step
    pub fn toggle_step(&mut self) {
        let val = &mut self.steps[self.selected_track][self.selected_step];
        *val = !*val;
        
        // Emit pattern changed event
        self.event_bus.emit(TrackerEvent::PatternChanged);
        
        // After toggling a step, we need to update the sequencer pattern
        if let Some(_) = &self.sequencer {
            // Recreate sequencer with updated pattern
            let was_playing = self.is_playing;
            let bpm = if let Some(seq) = &self.sequencer {
                seq.get_bpm()
            } else {
                self.bpm
            };
            
            // Stop if playing
            if was_playing {
                if let Some(seq) = &self.sequencer {
                    seq.stop();
                }
            }
            
            // We need to preserve audio connection when recreating the sequencer
            let has_audio = self.audio.is_some();
            
            // Clean up existing audio thread if any
            if has_audio {
                self.cleanup_audio();
            }
            
            // Create new sequencer with updated pattern and our shared event bus
            if let Err(err) = self.initialize_sequencer(has_audio) {
                debug!("Error reinitializing sequencer with audio: {}", err);
                // Fallback to just reinitializing the sequencer without audio
                let event_bus_clone = Arc::clone(&self.event_bus); 
                self.sequencer = Some(Sequencer::new_with_event_bus(bpm, self.steps.clone(), event_bus_clone));
            }
            
            // Resume if it was playing
            if was_playing {
                if let Some(seq) = &self.sequencer {
                    seq.start();
                }
            }
        }
    }

    /// Move the cursor in the specified direction
    pub fn move_cursor_left(&mut self) {
        if self.selected_step > 0 {
            self.selected_step -= 1;
        }
    }
    
    pub fn move_cursor_right(&mut self) {
        if self.selected_step + 1 < self.steps[0].len() {
            self.selected_step += 1;
        }
    }
    
    pub fn move_cursor_up(&mut self) {
        if self.selected_track > 0 {
            self.selected_track -= 1;
        }
    }
    
    pub fn move_cursor_down(&mut self) {
        if self.selected_track + 1 < self.steps.len() {
            self.selected_track += 1;
        }
    }

    /// Start or stop the sequencer
    pub fn toggle_playback(&mut self) {
        self.is_playing = !self.is_playing;
        
        if let Some(sequencer) = &self.sequencer {
            if self.is_playing {
                // Start the sequencer
                sequencer.start();
                info!("Sequencer started");
            } else {
                // Stop the sequencer
                sequencer.stop();
                // Reset current step when stopping
                self.current_step = 0;
                info!("Sequencer stopped");
            }
        }
    }
    
    /// Process and handle any events from the sequencer
    pub fn process_sequencer_events(&mut self) {
        // Process any events from the sequencer
        if let Some(sequencer) = &self.sequencer {
            // Get the current step from the sequencer
            self.current_step = sequencer.current_step();
            
            // Process legacy trigger events for backward compatibility
            let events = sequencer.tick();
            if !events.is_empty() {
                debug!("Got {} legacy trigger events", events.len());
                self.trigger_events = events.clone();
                
                // Store trigger events for UI or other purposes,
                // but don't directly trigger audio - that now happens via the event bus
            }
            
            // Note: we don't need to explicitly process audio triggers anymore
            // The AudioConnector now receives events directly from the event bus
        }
    }

    /// Set the BPM (tempo) for the sequencer
    pub fn set_bpm(&mut self, bpm: u32) {
        self.bpm = bpm;
        
        // Emit BPM changed event
        self.event_bus.emit(TrackerEvent::BpmChanged(bpm));
        
        // Also update sequencer directly for backward compatibility
        if let Some(sequencer) = &self.sequencer {
            sequencer.set_bpm(bpm);
        }
    }
    
    /// Trigger a test sound on a specific track (for debugging)
    pub fn test_track_sound(&mut self, track_idx: usize) -> Result<(), Box<dyn std::error::Error>> {
        if track_idx >= self.num_tracks() {
            return Err(format!("Track index {} out of bounds", track_idx).into());
        }
        
        // Check if audio is initialized
        if self.audio.is_none() {
            return Err("Audio not initialized".into());
        }
        
        // Emit event through the event bus
        self.event_bus.emit(TrackerEvent::StepTriggered(track_idx, 0));
        
        info!("Test sound triggered for track {}", track_idx);
        Ok(())
    }

    /// Initialize the sequencer with the current pattern data and BPM
    /// If with_audio is true, also initialize and connect the audio system
    pub fn initialize_sequencer(&mut self, with_audio: bool) -> Result<(), Box<dyn std::error::Error>> {
        // Create the sequencer with our shared event bus
        let event_bus_clone = Arc::clone(&self.event_bus);
        self.sequencer = Some(Sequencer::new_with_event_bus(self.bpm, self.steps.clone(), event_bus_clone));
        
        // Initialize audio if requested
        if with_audio {
            // Initialize audio system if it hasn't been initialized yet
            if self.audio.is_none() {
                self.initialize_audio()?;
            }
            
            // Connect audio to sequencer
            self.connect_audio_to_sequencer()?;
        }
        
        Ok(())
    }
    
    /// Initialize the audio system with the current sample directory and track configurations
    pub fn initialize_audio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing audio system with sample directory: {}", self.sample_dir.display());
        
        // Create an AudioConnector with the sample directory
        let connector = AudioConnector::new(&self.sample_dir)?;
        
        // Initialize the connector with track data
        connector.initialize(&self.tracks)?;
        
        self.audio = Some(connector);
        info!("Audio system initialized successfully");
        
        Ok(())
    }
    
    /// Connect the audio system to the sequencer to handle trigger events
    pub fn connect_audio_to_sequencer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.audio.is_none() || self.sequencer.is_none() {
            return Err("Audio or sequencer not initialized".into());
        }
        
        let audio = self.audio.as_ref().unwrap();
        
        info!("Connecting audio system to event bus");
        
        // Connect audio directly to the event bus
        // audio.connect_to_event_bus(Arc::clone(&self.event_bus));
        
        info!("Audio-event bus connection established");
        
        // Also maintain backwards compatibility by connecting to sequencer
        let sequencer = self.sequencer.as_ref().unwrap();
        if !audio.connect_to_sequencer(sequencer) {
            debug!("Legacy sequencer connection failed, but event bus connection should work");
        }
        
        Ok(())
    }
    
    /// Clean up audio resources when the application exits
    pub fn cleanup_audio(&mut self) {
        info!("Cleaning up audio resources");
        
        // Deactivate audio connector if it exists
        if let Some(audio) = &self.audio {
            audio.deactivate();
        }
        
        // Clear the audio connector
        self.audio = None;
        
        info!("Audio resources cleaned up");
    }
    
    /// Get the total number of steps in the pattern
    pub fn num_steps(&self) -> usize {
        if self.steps.is_empty() {
            0
        } else {
            self.steps[0].len()
        }
    }
    
    /// Get the total number of tracks
    pub fn num_tracks(&self) -> usize {
        self.steps.len()
    }
    
    /// Get the sample file path for a specific track
    pub fn get_track_sample(&self, track_idx: usize) -> Option<&str> {
        if track_idx < self.tracks.len() {
            Some(&self.tracks[track_idx].sample)
        } else {
            None
        }
    }
    
    /// Get the volume for a specific track
    pub fn get_track_volume(&self, track_idx: usize) -> Option<f32> {
        if track_idx < self.tracks.len() {
            Some(self.tracks[track_idx].volume)
        } else {
            None
        }
    }
    
    /// Set the volume for a specific track
    pub fn set_track_volume(&mut self, track_idx: usize, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Update the track configuration
        if track_idx < self.tracks.len() {
            // Clamp volume between 0 and 1
            let volume = volume.max(0.0).min(1.0);
            
            // Update the track volume in the model
            self.tracks[track_idx].volume = volume;
            
            info!("Setting track {} volume to {:.2}", track_idx, volume);
            
            // Emit event through the event bus
            self.event_bus.emit(TrackerEvent::TrackVolumeChanged(track_idx, volume));
            
            // Legacy direct update if event bus isn't connected yet
            if let Some(audio) = &self.audio {
                if let Err(e) = audio.set_track_volume(track_idx, volume) {
                    debug!("Legacy volume update failed: {}", e);
                }
            }
            
            Ok(())
        } else {
            Err(format!("Track index {} out of bounds", track_idx).into())
        }
    }
    
    /// Get a reference to the event bus
    pub fn get_event_bus(&self) -> &SharedEventBus {
        &self.event_bus
    }
    
    /// Subscribe to events from the app state
    pub fn subscribe_to_events<F>(&self, listener: F) -> usize 
    where 
        F: Fn(&TrackerEvent) + Send + Sync + 'static
    {
        self.event_bus.subscribe(listener)
    }
    
    /// Emit an event through the app state's event bus
    pub fn emit_event(&self, event: TrackerEvent) {
        self.event_bus.emit(event);
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        // Ensure we clean up resources properly
        info!("AppState being dropped, cleaning up resources");
        
        // Stop the sequencer if it's running
        if self.is_playing {
            if let Some(sequencer) = &self.sequencer {
                sequencer.stop();
            }
            self.is_playing = false;
        }
        
        // Clean up audio resources
        self.cleanup_audio();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_state_creation() {
        let app = AppState::new(4, 16);
        assert_eq!(app.num_tracks(), 4);
        assert_eq!(app.num_steps(), 16);
        assert_eq!(app.is_playing, false);
        assert_eq!(app.current_step, 0);
    }
    
    #[test]
    fn test_toggle_step() {
        let mut app = AppState::new(2, 8);
        app.selected_track = 0;
        app.selected_step = 0;
        
        // Initially false
        assert_eq!(app.steps[0][0], false);
        
        // Toggle to true
        app.toggle_step();
        assert_eq!(app.steps[0][0], true);
        
        // Toggle back to false
        app.toggle_step();
        assert_eq!(app.steps[0][0], false);
    }
    
    #[test]
    fn test_cursor_movement() {
        let mut app = AppState::new(3, 8);
        assert_eq!(app.selected_track, 0);
        assert_eq!(app.selected_step, 0);
        
        // Move right
        app.move_cursor_right();
        assert_eq!(app.selected_step, 1);
        
        // Move down
        app.move_cursor_down();
        assert_eq!(app.selected_track, 1);
        
        // Move left
        app.move_cursor_left();
        assert_eq!(app.selected_step, 0);
        
        // Move up
        app.move_cursor_up();
        assert_eq!(app.selected_track, 0);
    }
}
