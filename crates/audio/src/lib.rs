// audio module
mod connector;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use log::{debug, error, info};
use sequencer::TriggerEvent;
use project::model::Track;
use rodio::source::Source;

// Re-export important types
pub use crate::connector::AudioConnector;

/// Error types for the audio system
#[derive(Debug)]
pub enum AudioError {
    InitializationError(String),
    SampleLoadError(String, String),
    PlaybackError(String),
    SampleNotFound(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitializationError(msg) => write!(f, "Failed to initialize audio output: {}", msg),
            Self::SampleLoadError(sample, msg) => write!(f, "Failed to load sample {}: {}", sample, msg),
            Self::PlaybackError(msg) => write!(f, "Playback error: {}", msg),
            Self::SampleNotFound(msg) => write!(f, "Sample not found: {}", msg),
            Self::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for AudioError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AudioError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

/// Represents an audio sample in memory
pub struct Sample {
    /// The name of the sample
    pub name: String,
    
    /// The raw audio data
    data: Vec<u8>,
    
    /// File path for debugging/reference
    #[allow(dead_code)]
    path: PathBuf,
}

/// Audio player for sample playback
pub struct SamplePlayer {
    /// Output stream and handle for audio playback
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    
    /// Map of track indexes to sinks
    track_sinks: HashMap<usize, Sink>,
    
    /// Map of track indexes to sample indexes
    track_to_sample: HashMap<usize, usize>,
    
    /// Loaded samples
    samples: Vec<Sample>,
    
    /// Track configurations for volume
    tracks: HashMap<usize, Track>,
    
    /// Base directory for sample files
    sample_dir: PathBuf,
    
    /// Whether the player is active
    active: bool,
    
    /// Sample processor for effects
    processor: SampleProcessor,
}

impl SamplePlayer {
    /// Create a new SamplePlayer
    pub fn new(sample_dir: impl AsRef<Path>) -> Result<Self, AudioError> {
        // Initialize audio output stream
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::InitializationError(e.to_string()))?;
            
        Ok(Self {
            _stream: stream,
            stream_handle,
            track_sinks: HashMap::new(),
            track_to_sample: HashMap::new(),
            samples: Vec::new(),
            tracks: HashMap::new(),
            sample_dir: sample_dir.as_ref().to_path_buf(),
            active: false,
            processor: SampleProcessor::new(),
        })
    }
    
    /// Load a sample into memory
    pub fn load_sample(&mut self, name: &str, file_path: &str) -> Result<usize, AudioError> {
        let path = self.sample_dir.join(file_path);
        debug!("Loading sample '{}' from {}", name, path.display());
        
        // Open the file
        let file = File::open(&path)
            .map_err(|e| AudioError::SampleLoadError(
                file_path.to_string(), 
                format!("Failed to open file: {}", e)
            ))?;
        
        // Read the entire file to memory
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        std::io::copy(&mut reader, &mut buffer)
            .map_err(|e| AudioError::SampleLoadError(
                file_path.to_string(), 
                format!("Failed to read file: {}", e)
            ))?;
            
        // Store the sample
        let sample_idx = self.samples.len();
        self.samples.push(Sample {
            name: name.to_string(),
            data: buffer,
            path,
        });
        
        Ok(sample_idx)
    }
    
    /// Initialize the player with track configurations
    pub fn initialize_with_tracks(&mut self, tracks: &[Track]) -> Result<(), AudioError> {
        debug!("Initializing SamplePlayer with {} tracks", tracks.len());
        
        // Clear existing data
        self.track_sinks.clear();
        self.track_to_sample.clear();
        self.tracks.clear();
        
        // Load samples for each track
        for (track_idx, track) in tracks.iter().enumerate() {
            debug!("Setting up track {}: '{}' with sample '{}'", 
                  track_idx, track.name, track.sample);
            
            // Load the sample if it hasn't been loaded already
            let sample_idx = self.find_or_load_sample(&track.name, &track.sample)?;
            
            // Create a sink for this track
            let sink = Sink::try_new(&self.stream_handle)
                .map_err(|e| AudioError::InitializationError(
                    format!("Failed to create sink for track {}: {}", track_idx, e)
                ))?;
                
            // Set the track's volume
            sink.set_volume(track.volume);
                
            // Store the mappings
            self.track_sinks.insert(track_idx, sink);
            self.track_to_sample.insert(track_idx, sample_idx);
            self.tracks.insert(track_idx, track.clone());
        }
        
        self.active = true;
        info!("SamplePlayer initialized with {} tracks and {} samples", 
             tracks.len(), self.samples.len());
             
        Ok(())
    }
    
    /// Find a sample by name or load it if not found
    fn find_or_load_sample(&mut self, name: &str, file_path: &str) -> Result<usize, AudioError> {
        // Check if we already have this sample
        if let Some((idx, _)) = self.samples.iter().enumerate()
            .find(|(_, s)| s.name == name) {
            return Ok(idx);
        }
        
        // If not, load it
        self.load_sample(name, file_path)
    }
    
    /// Process trigger events from the sequencer
    pub fn process_trigger(&mut self, event: &TriggerEvent) -> Result<(), AudioError> {
        if !self.active {
            return Ok(());
        }
        
        // Check if we have a mapping for this track
        let track_idx = event.track_idx;
        if !self.track_to_sample.contains_key(&track_idx) {
            debug!("No sample mapping for track {}", track_idx);
            return Ok(());
        }
        
        // Get the sample
        let sample_idx = self.track_to_sample[&track_idx];
        if sample_idx >= self.samples.len() {
            return Err(AudioError::SampleNotFound(
                format!("Sample index {} out of bounds", sample_idx)
            ));
        }
        
        // Play the sample
        self.play_sample(track_idx, sample_idx)
    }
    
    /// Play a specific sample on a specific track
    pub fn play_sample(&mut self, track_idx: usize, sample_idx: usize) -> Result<(), AudioError> {
        debug!("Playing sample {} on track {}", sample_idx, track_idx);
        
        // Get the sink for this track
        let sink = match self.track_sinks.get_mut(&track_idx) {
            Some(sink) => sink,
            None => {
                // Create a new sink if one doesn't exist
                let sink = Sink::try_new(&self.stream_handle)
                    .map_err(|e| AudioError::PlaybackError(format!(
                        "Failed to create sink for track {}: {}", track_idx, e
                    )))?;
                self.track_sinks.insert(track_idx, sink);
                self.track_sinks.get_mut(&track_idx).unwrap()
            }
        };
        
        // Get the sample data
        let sample = &self.samples[sample_idx];
        
        // Check if the sink is already playing something
        if !sink.empty() {
            // Stop any current playback on this track
            sink.stop();
        }
        
        // Force resetting the sink to ensure clean playback
        sink.clear();
        
        // Create a cursor for the sample data
        let cursor = Cursor::new(sample.data.clone());
        
        // Decode the sample
        let source = match Decoder::new(cursor) {
            Ok(source) => source,
            Err(e) => {
                error!("Failed to decode sample {}: {}", sample.name, e);
                return Err(AudioError::PlaybackError(format!(
                    "Failed to decode sample {}: {}", sample.name, e
                )));
            }
        };
        
        // Apply effects using the processor
        let processed_source = match self.processor.process_sample(track_idx, source) {
            Ok(src) => src,
            Err(e) => {
                error!("Failed to process effects for sample {}: {}", sample.name, e);
                return Err(e);
            }
        };
        
        // Play the processed sample
        sink.append(processed_source);
        sink.play(); // Explicitly set to play mode
        
        // Set the volume for this track
        if let Some(track) = self.tracks.get(&track_idx) {
            sink.set_volume(track.volume);
            debug!("Set volume for track {} to {}", track_idx, track.volume);
        }
        
        // This is the critical point where the audio should play
        info!("Sample playback started for track {} ({})", track_idx, sample.name);
        
        // Check if the sink is detached or paused (this can cause silent playback)
        if sink.is_paused() {
            debug!("Warning: Sink is paused for track {} - unpausing", track_idx);
            sink.play(); // Make sure playback starts
        }
        
        Ok(())
    }
    
    /// Stop all playback
    pub fn stop_all(&mut self) {
        for (_, sink) in &self.track_sinks {
            sink.stop();
        }
    }
    
    /// Set volume for a specific track (0.0 to 1.0)
    pub fn set_track_volume(&mut self, track_idx: usize, volume: f32) -> Result<(), AudioError> {
        let sink = self.track_sinks.get(&track_idx).ok_or_else(|| {
            AudioError::PlaybackError(format!("Track {} not found", track_idx))
        })?;
        
        sink.set_volume(volume);
        Ok(())
    }
    
    /// Check if the player is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Effects that can be applied to samples
#[derive(Debug, Clone)]
pub enum SampleEffect {
    /// Play the sample in reverse
    Reverse,
    
    /// Apply fade in effect (seconds)
    FadeIn(f32),
    
    /// Apply fade out effect (seconds)
    FadeOut(f32),
    
    /// Play only a portion of the sample (start_fraction, end_fraction)
    Partial(f32, f32),
}

/// Sample processor for audio effects
pub struct SampleProcessor {
    effects: HashMap<usize, Vec<SampleEffect>>,
}

impl SampleProcessor {
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
        }
    }
    
    /// Add an effect to a track
    pub fn add_effect(&mut self, track_idx: usize, effect: SampleEffect) {
        let track_effects = self.effects.entry(track_idx).or_insert_with(Vec::new);
        track_effects.push(effect);
    }
    
    /// Remove all effects from a track
    pub fn clear_effects(&mut self, track_idx: usize) {
        self.effects.remove(&track_idx);
    }
    
    /// Get all effects for a track
    pub fn get_effects(&self, track_idx: &usize) -> Option<&Vec<SampleEffect>> {
        self.effects.get(track_idx)
    }
    
    /// Process a sample using the registered effects for a track
    /// This returns a processed source ready for playback
    pub fn process_sample(&self, track_idx: usize, source: Decoder<Cursor<Vec<u8>>>) 
        -> Result<Box<dyn rodio::Source<Item = i16> + Send>, AudioError> {
        
        // If no effects registered for this track, return as-is
        if !self.effects.contains_key(&track_idx) {
            return Ok(Box::new(source));
        }
        
        // Get the effects for this track
        let effects = self.effects.get(&track_idx).unwrap();
        
        // Save initial source properties before any processing
        let channels = source.channels();
        let sample_rate = source.sample_rate();
        
        // Apply each effect in order
        let mut processed: Box<dyn rodio::Source<Item = i16> + Send> = Box::new(source);
        
        for effect in effects {
            match effect {
                SampleEffect::Reverse => {
                    // Collect source into a buffer and reverse it
                    let collected: Vec<i16> = processed.collect();
                    let mut reversed = collected;
                    reversed.reverse();
                    
                    // Create a new source from the reversed buffer
                    processed = Box::new(rodio::buffer::SamplesBuffer::new(
                        channels,
                        sample_rate,
                        reversed
                    ));
                },
                SampleEffect::FadeIn(duration_secs) => {
                    // Convert seconds to samples
                    let fade_samples = (sample_rate as f32 * duration_secs * channels as f32) as usize;
                    // Collect and recreate to avoid borrowing issues
                    let collected: Vec<i16> = processed.collect();
                    let fade_source = FadeIn::new(
                        rodio::buffer::SamplesBuffer::new(channels, sample_rate, collected),
                        fade_samples
                    );
                    processed = Box::new(fade_source);
                },
                SampleEffect::FadeOut(duration_secs) => {
                    // Convert seconds to samples
                    let fade_samples = (sample_rate as f32 * duration_secs * channels as f32) as usize;
                    // Collect and recreate to avoid borrowing issues
                    let collected: Vec<i16> = processed.collect();
                    let fade_source = FadeOut::new(
                        rodio::buffer::SamplesBuffer::new(channels, sample_rate, collected),
                        fade_samples
                    );
                    processed = Box::new(fade_source);
                },
                SampleEffect::Partial(start_frac, end_frac) => {
                    // Validate fractions
                    let start = start_frac.max(0.0).min(1.0);
                    let end = end_frac.max(start).min(1.0);
                    
                    // Collect into buffer for length calculation
                    let collected: Vec<i16> = processed.collect();
                    let total_samples = collected.len();
                    let start_sample = (total_samples as f32 * start) as usize;
                    let end_sample = (total_samples as f32 * end) as usize;
                    
                    // Create a partial sample buffer
                    let partial: Vec<i16> = collected[start_sample..end_sample].to_vec();
                    processed = Box::new(rodio::buffer::SamplesBuffer::new(
                        channels,
                        sample_rate,
                        partial
                    ));
                }
            }
        }
        
        Ok(processed)
    }
}

/// Effect that applies a fade-in to a source
pub struct FadeIn<S> {
    source: S,
    fade_samples: usize,
    current_sample: usize,
}

impl<S> FadeIn<S> {
    fn new(source: S, fade_samples: usize) -> Self {
        Self {
            source,
            fade_samples,
            current_sample: 0,
        }
    }
}

impl<S> Iterator for FadeIn<S>
where
    S: Iterator<Item = i16>,
{
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next().map(|sample| {
            if self.current_sample < self.fade_samples {
                let factor = self.current_sample as f32 / self.fade_samples as f32;
                self.current_sample += 1;
                (sample as f32 * factor) as i16
            } else {
                sample
            }
        })
    }
}

impl<S> rodio::Source for FadeIn<S>
where
    S: rodio::Source<Item = i16>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.source.total_duration()
    }
}

/// Effect that applies a fade-out to a source
pub struct FadeOut<S> {
    source: S,
    fade_samples: usize,
    total_samples: Option<usize>,
    current_sample: usize,
}

impl<S> FadeOut<S>
where
    S: rodio::Source<Item = i16> + Iterator<Item = i16>,
{
    fn new(source: S, fade_samples: usize) -> Self {
        Self {
            source,
            fade_samples,
            total_samples: None,
            current_sample: 0,
        }
    }
}

impl<S> Iterator for FadeOut<S>
where
    S: rodio::Source<Item = i16> + Iterator<Item = i16>,
{
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.total_samples.is_none() {
            // Determine the total number of samples if not already known
            if let Some(duration) = self.source.total_duration() {
                let sample_rate = self.source.sample_rate();
                let channels = self.source.channels() as u32;
                let total_samples = (duration.as_secs_f32() * sample_rate as f32 * channels as f32) as usize;
                self.total_samples = Some(total_samples);
            }
        }
        
        self.source.next().map(|sample| {
            if let Some(total) = self.total_samples {
                if self.current_sample >= total.saturating_sub(self.fade_samples) {
                    let remaining = total.saturating_sub(self.current_sample);
                    let factor = remaining as f32 / self.fade_samples as f32;
                    self.current_sample += 1;
                    return (sample as f32 * factor) as i16;
                }
            }
            self.current_sample += 1;
            sample
        })
    }
}

impl<S> rodio::Source for FadeOut<S>
where
    S: rodio::Source<Item = i16>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.source.total_duration()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;
    use project::model::Track;
    
    // Basic test to ensure the module compiles
    #[test]
    fn test_sample_player_creation() {
        // This test may fail on systems without audio
        // Support, so we'll just check if it can be created
        if let Ok(_player) = SamplePlayer::new(Path::new("./samples")) {
            // Player created successfully
        }
    }
    
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
    fn test_error_handling() {
        let (_temp_dir, samples_dir, _) = setup_test_environment();
        
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
    
    #[test]
    fn test_find_or_load_sample() {
        let (_temp_dir, samples_dir, _) = setup_test_environment();
        
        if let Ok(mut player) = SamplePlayer::new(&samples_dir) {
            // First load should succeed
            let result1 = player.find_or_load_sample("Kick", "kick.wav");
            if result1.is_ok() {
                let idx1 = result1.unwrap();
                
                // Second load of the same sample should return the same index
                let result2 = player.find_or_load_sample("Kick", "kick.wav");
                if result2.is_ok() {
                    let idx2 = result2.unwrap();
                    assert_eq!(idx1, idx2, "Should return the same sample index when loading the same sample twice");
                }
            }
        }
    }
}
