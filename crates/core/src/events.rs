// Core event system for Gaucho Tracker
// This provides a common event system that different components can use

use std::sync::{Arc, Mutex};

/// Event type for the tracker system
#[derive(Debug, Clone)]
pub enum TrackerEvent {
    /// A step has been triggered (track_idx, step_idx)
    StepTriggered(usize, usize),
    
    /// BPM has been changed
    BpmChanged(u32),
    
    /// Playback state changed (is_playing)
    PlaybackStateChanged(bool),
    
    /// Pattern changed
    PatternChanged,
    
    /// Track volume changed (track_idx, volume)
    TrackVolumeChanged(usize, f32),
}

/// A simple event bus implementation
pub struct EventBus {
    listeners: Arc<Mutex<Vec<Box<dyn Fn(&TrackerEvent) + Send + Sync>>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Subscribe to events
    pub fn subscribe<F>(&self, listener: F) -> usize
    where
        F: Fn(&TrackerEvent) + Send + Sync + 'static,
    {
        let mut listeners = self.listeners.lock().unwrap();
        let id = listeners.len();
        listeners.push(Box::new(listener));
        id
    }
    
    /// Emit an event to all listeners
    pub fn emit(&self, event: TrackerEvent) {
        let listeners = self.listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener(&event);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// A reference counted, thread-safe event bus
pub type SharedEventBus = Arc<EventBus>;

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            listeners: Arc::clone(&self.listeners),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[test]
    fn test_event_bus() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        
        // Subscribe to events
        let counter_clone = Arc::clone(&counter);
        bus.subscribe(move |event| {
            if let TrackerEvent::StepTriggered(_, _) = event {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        
        // Emit some events
        bus.emit(TrackerEvent::StepTriggered(0, 0));
        bus.emit(TrackerEvent::BpmChanged(120));
        bus.emit(TrackerEvent::StepTriggered(1, 2));
        
        // Check counter was incremented only for StepTriggered events
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
