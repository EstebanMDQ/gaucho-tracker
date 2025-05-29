use crate::model::{Project, Pattern};
use std::fs;
use std::path::Path;

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
