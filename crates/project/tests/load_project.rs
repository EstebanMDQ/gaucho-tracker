use project::load_project;

#[test]
fn test_load_project_folder() {
    let folder = "tests/fixtures/my-song";
    let result = load_project(folder);
    assert!(result.is_ok(), "Project loading failed");
    let (project, patterns) = result.unwrap();
    assert_eq!(project.name, "My Song");
    assert_eq!(patterns.len(), 1);
}

use project::get_project_path;

#[test]
fn resolves_dev_or_home_path() {
    let path = get_project_path("my-song");
    assert!(path.ends_with("my-song"));
}