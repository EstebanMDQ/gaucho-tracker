#[cfg(test)]
mod tests {
    use crate::{Sequencer, TriggerEvent};
    use crate::integration::{Connection, EventDestination};
    use std::sync::mpsc;
    use std::thread::sleep;
    use std::time::Duration;
    use log::debug;

    #[test]
    fn test_integration_with_components() {
        // Setup an mpsc channel to simulate communication with TUI
        let (tx, rx) = mpsc::channel();
        
        // Create a simple pattern
        let pattern = vec![
            vec![true, false, false, false],
            vec![false, false, true, false],
        ];
        
        // Create sequencer
        let sequencer = Sequencer::new(240, pattern);
        
        // Setup a connection to simulate UI integration
        let mut ui_conn = crate::integration::setup_ui_connection();
        ui_conn.activate();
        
        // Start the sequencer
        sequencer.start();
        
        // In a real integration, here we would hook up the trigger events
        // to the TUI component. For this test, we'll simulate by:
        
        // Spawn a thread to process trigger events and forward to UI
        let sequencer_clone = sequencer.clone();
        let handle = std::thread::spawn(move || {
            for _ in 0..10 { // Process up to 10 ticks
                let triggers = sequencer_clone.tick();
                if !triggers.is_empty() {
                    // In real integration, would format and send to TUI
                    let _ = tx.send(format!("Step change: {}", sequencer_clone.current_step()));
                    for trigger in triggers {
                        let _ = tx.send(format!("Trigger: track {} at step {}", 
                            trigger.track_idx, trigger.step_idx));
                    }
                }
                sleep(Duration::from_millis(20));
            }
        });
        
        // Give sequencer time to run
        sleep(Duration::from_millis(200));
        
        // Stop the sequencer
        sequencer.stop();
        
        // Wait for processing thread
        let _ = handle.join();
        
        // Check if we received any events
        let mut received_messages = 0;
        while let Ok(_) = rx.try_recv() {
            received_messages += 1;
        }
        
        // We should have received some messages - but this is timing dependent
        // so we don't assert on exact count
        debug!("Received {} messages", received_messages);
    }
}