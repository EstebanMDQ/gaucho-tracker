// This module will contain code for integrating the sequencer with other components
use log::{debug};

/// This enum represents the possible destinations for sequencer events
#[derive(Debug, Clone, Copy)]
pub enum EventDestination {
    /// UI related events (highlighting current step, etc.)
    UI,
    /// Audio/sampler related events (playing sounds)
    Sampler,
    /// MIDI output events
    MIDI,
}

/// Represents a connection to another component
pub struct Connection {
    destination: EventDestination,
    active: bool,
}

impl Connection {
    pub fn new(destination: EventDestination) -> Self {
        Self {
            destination,
            active: false,
        }
    }
    
    pub fn activate(&mut self) {
        self.active = true;
        debug!("Connection to {:?} activated", self.destination);
    }
    
    pub fn deactivate(&mut self) {
        self.active = false;
        debug!("Connection to {:?} deactivated", self.destination);
    }
    
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Future: Add functions to manage connections between sequencer and other components
pub fn setup_ui_connection() -> Connection {
    Connection::new(EventDestination::UI)
}

pub fn setup_sampler_connection() -> Connection {
    Connection::new(EventDestination::Sampler)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection() {
        let mut conn = Connection::new(EventDestination::UI);
        assert_eq!(conn.is_active(), false);
        
        conn.activate();
        assert_eq!(conn.is_active(), true);
        
        conn.deactivate();
        assert_eq!(conn.is_active(), false);
    }
}
