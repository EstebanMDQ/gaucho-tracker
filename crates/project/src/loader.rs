use crate::model::{Project, Pattern};
use std::fs;
use std::path::Path;
use std::path::{PathBuf};
// canuse std::env;
use dirs::home_dir;

pub fn get_project_path(project_name: &str) -> PathBuf {
    // Check if running in development (e.g., if /projects folder exists in current dir)
    let dev_path = PathBuf::from("gaucho-projects").join(project_name);
    if dev_path.exists() {
        return dev_path;
    }

    // Fallback to production path
    let home = home_dir().unwrap_or_else(|| PathBuf::from("/home/pi"));
    home.join("gaucho-projects").join(project_name)
}

pub fn load_project<P: AsRef<Path>>(folder: P) -> Result<(Project, Vec<Pattern>), Box<dyn std::error::Error>> {
    let folder = folder.as_ref();

    let proj_toml = fs::read_to_string(folder.join("project.toml"))?;
    let project: Project = toml::from_str(&proj_toml)?;

    let mut patterns = Vec::new();
    for pat_file in &project.patterns {
        let pat_str = fs::read_to_string(folder.join(pat_file))?;
        let pattern: Pattern = toml::from_str(&pat_str)?;
        patterns.push(pattern);
    }

    Ok((project, patterns))
}
