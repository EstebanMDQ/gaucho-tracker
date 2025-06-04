use std::io::{self, Write, BufRead};
use std::path::{Path, PathBuf};
use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;
use std::fs;

use audio::AudioConnector;
use project::model::Track;
use sequencer::TriggerEvent;

// Configure logging for the demo
fn setup_logging() {
    // Create a log file in the system's temp directory
    let log_path = std::env::temp_dir().join("gaucho_audio_demo.log");
    println!("Logging to file: {}", log_path.display());
    
    // Create a file logger that appends to the log file
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
        .unwrap_or_else(|e| {
            eprintln!("Warning: Could not open log file: {}", e);
            std::fs::File::create(&log_path).expect("Failed to create log file")
        });
        
    // Configure the logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                buf,
                "[{}] [{}] {}",
                timestamp,
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();
}

fn find_project_samples() -> Option<PathBuf> {
    // First try to find the workspace root
    let mut current_dir = env::current_dir().ok()?;
    let mut workspace_root = None;
    
    // Look for Cargo.toml to identify workspace root
    while current_dir.parent().is_some() {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    workspace_root = Some(current_dir.clone());
                    break;
                }
            }
        }
        
        // Move up one directory
        if !current_dir.pop() {
            break;
        }
    }
    
    // Try possible sample paths
    let mut possible_paths = Vec::new();
    
    // If we found workspace root, add relative paths
    if let Some(root) = workspace_root {
        possible_paths.push(root.join("gaucho-projects/my-song/samples"));
    }
    
    // Add other possible locations
    possible_paths.extend_from_slice(&[
        // From crates/audio directory
        PathBuf::from("../../gaucho-projects/my-song/samples"),
        // From project root
        PathBuf::from("./gaucho-projects/my-song/samples"),
        // From project root with full path
        PathBuf::from("/Users/esteban.chq/informalthinkers/gaucho-tracker/gaucho-projects/my-song/samples"),
    ]);
    
    // Print all paths we're checking
    println!("Looking for sample files in:");
    for path in &possible_paths {
        println!("  - {}", path.display());
        if path.exists() && path.is_dir() {
            println!("    ✓ Found!");
            return Some(path.clone());
        } else {
            println!("    ✗ Not found");
        }
    }

    None
}

// Interactive test to play samples
fn run_interactive_demo(sample_dir: &Path) -> Result<(), Box<dyn Error>> {
    println!("Gaucho Tracker Audio Demo");
    println!("=========================");
    println!("Loading samples from: {}", sample_dir.display());
    
    // Define the tracks with samples to use
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
        },
        Track {
            name: "Open HiHat".to_string(),
            sample: "open-hihat.wav".to_string(),
            volume: 0.7,
        },
        Track {
            name: "Clap".to_string(),
            sample: "reverbered-clap-punchy-shot.wav".to_string(),
            volume: 0.6,
        },
    ];
    
    // Create and initialize the audio connector
    let connector = AudioConnector::new(sample_dir)?;
    connector.initialize(&tracks)?;
    
    println!("\nAvailable commands:");
    println!("  1-5: Play sample on track 1-5");
    println!("  r1-r5: Play sample with reverse effect");
    println!("  f1-f5: Play sample with fade-in effect");
    println!("  o1-o5: Play sample with fade-out effect");
    println!("  +/-: Increase/decrease volume for the last played track");
    println!("  s: Play a simple beat sequence");
    println!("  q: Quit");
    println!("\nEnter a command:");
    
    let stdin = io::stdin();
    let mut last_track: Option<usize> = None;
    let mut reader = stdin.lock();
    let mut input = String::new();
    
    loop {
        // Read user input
        print!("> ");
        io::stdout().flush()?;
        input.clear();
        reader.read_line(&mut input)?;
        let command = input.trim();
        
        match command {
            "q" => break,
            
            // Play simple samples
            "1" | "2" | "3" | "4" | "5" => {
                let track_idx = command.parse::<usize>().unwrap() - 1;
                if track_idx < tracks.len() {
                    println!("Playing {} sample", tracks[track_idx].name);
                    let event = TriggerEvent { track_idx, step_idx: 0 };
                    connector.process_trigger(&event)?;
                    last_track = Some(track_idx);
                }
            },
            
            // Play with special effects
            // Simplified for now - we'll use basic playback for the demo
            "r1" | "r2" | "r3" | "r4" | "r5" => {
                let track_idx = command[1..].parse::<usize>().unwrap() - 1;
                if track_idx < tracks.len() {
                    println!("Playing {} sample (would apply reverse effect)", tracks[track_idx].name);
                    let event = TriggerEvent { track_idx, step_idx: 0 };
                    connector.process_trigger(&event)?;
                    last_track = Some(track_idx);
                    
                    // Keep alive while playing
                    thread::sleep(Duration::from_millis(500));
                }
            },
            
            // Play with fade-in effect
            "f1" | "f2" | "f3" | "f4" | "f5" => {
                let track_idx = command[1..].parse::<usize>().unwrap() - 1;
                if track_idx < tracks.len() {
                    println!("Playing {} sample (would apply fade-in effect)", tracks[track_idx].name);
                    let event = TriggerEvent { track_idx, step_idx: 0 };
                    connector.process_trigger(&event)?;
                    last_track = Some(track_idx);
                    
                    // Keep alive while playing
                    thread::sleep(Duration::from_millis(500));
                }
            },
            
            // Play with fade-out effect
            "o1" | "o2" | "o3" | "o4" | "o5" => {
                let track_idx = command[1..].parse::<usize>().unwrap() - 1;
                if track_idx < tracks.len() {
                    println!("Playing {} sample (would apply fade-out effect)", tracks[track_idx].name);
                    let event = TriggerEvent { track_idx, step_idx: 0 };
                    connector.process_trigger(&event)?;
                    last_track = Some(track_idx);
                    
                    // Keep alive while playing
                    thread::sleep(Duration::from_millis(500));
                }
            },
            
            // Increase volume
            "+" => {
                if let Some(track_idx) = last_track {
                    let volume = (tracks[track_idx].volume + 0.1).min(1.0);
                    println!("Setting {} volume to {:.1}", tracks[track_idx].name, volume);
                    connector.set_track_volume(track_idx, volume)?;
                    
                    // Play the sample to demonstrate new volume
                    let event = TriggerEvent { track_idx, step_idx: 0 };
                    connector.process_trigger(&event)?;
                } else {
                    println!("Play a sample first before adjusting volume");
                }
            },
            
            // Decrease volume
            "-" => {
                if let Some(track_idx) = last_track {
                    let volume = (tracks[track_idx].volume - 0.1).max(0.0);
                    println!("Setting {} volume to {:.1}", tracks[track_idx].name, volume);
                    connector.set_track_volume(track_idx, volume)?;
                    
                    // Play the sample to demonstrate new volume
                    let event = TriggerEvent { track_idx, step_idx: 0 };
                    connector.process_trigger(&event)?;
                } else {
                    println!("Play a sample first before adjusting volume");
                }
            },
            
            // Play a simple beat sequence
            "s" => {
                println!("Playing a simple beat sequence...");
                
                // Define a simple pattern
                // Kick on beats 1, 5, 9, 13
                // Snare on beats 5, 13
                // HiHat on every beat
                for i in 0..16 {
                    // Play kick on select beats
                    if i == 0 || i == 4 || i == 8 || i == 12 {
                        connector.process_trigger(&TriggerEvent { track_idx: 0, step_idx: i })?;
                    }
                    
                    // Play snare on beats 5 and 13
                    if i == 4 || i == 12 {
                        connector.process_trigger(&TriggerEvent { track_idx: 1, step_idx: i })?;
                    }
                    
                    // Play hi-hat on every beat
                    connector.process_trigger(&TriggerEvent { track_idx: 2, step_idx: i })?;
                    
                    // Add some open hi-hat occasionally
                    if i == 7 || i == 15 {
                        connector.process_trigger(&TriggerEvent { track_idx: 3, step_idx: i })?;
                    }
                    
                    // Wait a bit between steps
                    thread::sleep(Duration::from_millis(150));
                }
            },
            
            _ => println!("Unknown command: {}", command),
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    setup_logging();
    
    println!("Initializing Gaucho Tracker Audio Demo");
    
    // Find the samples directory
    let sample_dir = match find_project_samples() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: Could not find samples directory.");
            eprintln!("Please run this demo from the project root directory.");
            return Err("Samples directory not found".into());
        }
    };
    
    // Run the interactive demo
    run_interactive_demo(&sample_dir)?;
    
    println!("Audio demo completed");
    Ok(())
}
