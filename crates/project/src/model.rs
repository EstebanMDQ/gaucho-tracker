use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub bpm: u32,
    pub tracks: Vec<String>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Pattern {
    pub steps: Vec<Vec<bool>>,
}
