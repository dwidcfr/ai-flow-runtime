use ai_evals::{crate_root, ScenarioLoader};

#[test]
fn loader_parses_valid_scenario() {
    let loader = ScenarioLoader::new(&crate_root()).unwrap();
    let scenario = loader.load_by_name("greeting_confirm").unwrap();
    assert_eq!(scenario.flow, "payment");
    assert_eq!(scenario.user_message, "да, это я");
}

#[test]
fn loader_rejects_duplicate_names() {
    let dir = tempfile::tempdir().unwrap();
    let evals = dir.path().join("evals");
    std::fs::create_dir_all(&evals).unwrap();
    std::fs::write(
        dir.path().join("fixtures.yaml"),
        std::fs::read_to_string(crate_root().join("fixtures.yaml")).unwrap(),
    )
    .unwrap();
    std::fs::write(evals.join("a.yaml"), "name: dup\nflow: payment\nclient: sample_client\nuser_message: test\nexpected: {}").unwrap();
    std::fs::write(evals.join("b.yaml"), "name: dup\nflow: payment\nclient: sample_client\nuser_message: test\nexpected: {}").unwrap();

    let loader = ScenarioLoader::new(dir.path()).unwrap();
    assert!(loader.discover_all().is_err());
}

#[test]
fn loader_invalid_router_type_fails() {
    let dir = tempfile::tempdir().unwrap();
    let evals = dir.path().join("evals");
    std::fs::create_dir_all(&evals).unwrap();
    std::fs::write(
        dir.path().join("fixtures.yaml"),
        std::fs::read_to_string(crate_root().join("fixtures.yaml")).unwrap(),
    )
    .unwrap();
    std::fs::write(
        evals.join("bad.yaml"),
        r#"
name: bad
flow: payment
client: sample_client
user_message: test
expected:
  router:
    type: invalid
"#,
    )
    .unwrap();

    let loader = ScenarioLoader::new(dir.path()).unwrap();
    assert!(loader.discover_all().is_err());
}
