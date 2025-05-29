use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub bpm: u32,
    pub swing: f32,
    pub author: String,
    pub created: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Track {
    pub name: String,
    pub sample: String,
    pub volume: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pattern {
    pub pattern_id: u32,
    pub steps: Vec<Vec<bool>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackMapEntry {
    pub channel: u32,
    pub sample: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FxEntry {
    pub retrigger: Option<u32>,
    pub reverse: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatternMeta {
    pub track_map: Vec<TrackMapEntry>,
    pub fx: HashMap<String, FxEntry>,
}
