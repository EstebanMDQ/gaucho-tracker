// App state for the Gaucho Tracker
use sequencer::{Sequencer, TriggerEvent};
use log::{debug, info};

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
}

impl AppState {
    /// Creates a new AppState with empty pattern data
    pub fn new(num_tracks: usize, num_steps: usize) -> Self {
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
        }
    }

    /// Toggle the currently selected step
    pub fn toggle_step(&mut self) {
        let val = &mut self.steps[self.selected_track][self.selected_step];
        *val = !*val;
        
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
            
            // Create new sequencer with updated pattern
            self.sequencer = Some(Sequencer::new(bpm, self.steps.clone()));
            
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
            
            // Process trigger events
            let events = sequencer.tick();
            if !events.is_empty() {
                debug!("Got {} trigger events", events.len());
                self.trigger_events = events;
                
                // You could add sound triggering here in the future
                // For now, just log the triggers
                for event in &self.trigger_events {
                    debug!("Trigger: track {} at step {}", event.track_idx, event.step_idx);
                }
            }
        }
    }

    /// Set the BPM (tempo) for the sequencer
    pub fn set_bpm(&mut self, bpm: u32) {
        self.bpm = bpm;
        if let Some(sequencer) = &self.sequencer {
            sequencer.set_bpm(bpm);
        }
    }

    /// Initialize the sequencer with the current pattern data and BPM
    pub fn initialize_sequencer(&mut self) {
        self.sequencer = Some(Sequencer::new(self.bpm, self.steps.clone()));
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
