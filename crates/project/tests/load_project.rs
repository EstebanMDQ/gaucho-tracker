use project::load_project;

#[test]
fn test_load_project_folder() {
    let folder = "tests/fixtures/my-song";
    let result = load_project(folder);
    assert!(result.is_ok(), "Project loading failed");
    
    let (project, tracks, patterns, pattern_metas) = result.unwrap();
    
    // Test project metadata
    assert_eq!(project.name, "My Song");
    assert_eq!(project.version, "1.0");
    assert_eq!(project.bpm, 120);
    assert_eq!(project.swing, 0.0);
    assert_eq!(project.author, "esteban");
    
    // Test tracks
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].name, "Kick");
    assert_eq!(tracks[0].sample, "samples/kick.wav");
    assert_eq!(tracks[0].volume, 1.0);
    
    // Test patterns
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].pattern_id, 0);
    assert_eq!(patterns[0].steps.len(), 2);
    assert_eq!(patterns[0].steps[0].len(), 8);
    assert!(patterns[0].steps[0][0]); // First step of first track should be true
}

use project::get_project_path;

#[test]
fn resolves_dev_or_home_path() {
    let path = get_project_path("my-song");
    assert!(path.ends_with("my-song"));
}