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
