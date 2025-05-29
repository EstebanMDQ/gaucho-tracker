use crate::model::{Project, Pattern, Track, PatternMeta};
use std::env;
use std::fs;
use std::path::Path;
use std::path::{PathBuf};
use dirs::home_dir;
use log::{debug, error, info};

pub fn get_project_path(project_name: &str) -> PathBuf {
    // Check for projects in the current directory first
    let base_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let dev_paths = vec![
        base_path.join("projects").join(project_name),
        base_path.join("gaucho-projects").join(project_name)
    ];
    
    for path in dev_paths {
        debug!("Checking path: {:?}", path);
        if path.exists() {
            return path;
        }
    }

    // Fallback to production path
    let home = home_dir().unwrap_or_else(|| PathBuf::from("/home/pi"));
    home.join("gaucho-projects").join(project_name)
}

pub fn load_project<P: AsRef<Path>>(folder: P) -> Result<(Project, Vec<Track>, Vec<Pattern>), Box<dyn std::error::Error>> {
    let folder = folder.as_ref();
    info!("Loading project from: {:?}", folder);

    // Load project metadata
    let gaucho_toml_path = folder.join("gaucho.toml");
    debug!("Loading project metadata from: {:?}", gaucho_toml_path);
    let proj_toml = fs::read_to_string(&gaucho_toml_path)?;
    let project: Project = toml::from_str(&proj_toml)?;
    info!("Project loaded: {}", project.name);

    // Load tracks
    let tracks_json_path = folder.join("tracks.json");
    debug!("Loading tracks from: {:?}", tracks_json_path);
    let tracks_json = fs::read_to_string(&tracks_json_path)?;
    let tracks: Vec<Track> = serde_json::from_str(&tracks_json)?;
    info!("Loaded {} tracks", tracks.len());

    // Load patterns
    let patterns_dir = folder.join("patterns");
    debug!("Loading patterns from: {:?}", patterns_dir);
    let mut patterns = Vec::new();
    
    if patterns_dir.exists() && patterns_dir.is_dir() {
        let entries = fs::read_dir(&patterns_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Only process JSON files and skip metadata files
            if path.is_file() && 
               path.extension().map_or(false, |ext| ext == "json") && 
               !path.to_string_lossy().contains(".meta.json") {
                
                debug!("Loading pattern from: {:?}", path);
                let pattern_json = fs::read_to_string(&path)?;
                let pattern: Pattern = serde_json::from_str(&pattern_json)?;
                patterns.push(pattern);
                
                // Optionally load the metadata file
                let meta_filename = path.file_stem()
                    .and_then(|stem| Some(format!("{}.meta.json", stem.to_string_lossy())));
                
                if let Some(meta_name) = meta_filename {
                    let meta_path = path.with_file_name(meta_name);
                    if meta_path.exists() {
                        debug!("Loading pattern metadata from: {:?}", meta_path);
                        let meta_json = fs::read_to_string(&meta_path)?;
                        let _meta: PatternMeta = serde_json::from_str(&meta_json)?;
                        // Process metadata if needed
                    }
                }
            }
        }
    }
    
    info!("Loaded {} patterns", patterns.len());
    Ok((project, tracks, patterns))
}
