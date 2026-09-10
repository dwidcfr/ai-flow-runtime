use std::fs;

use ai_prompts::PromptRegistry;
use tempfile::TempDir;

#[test]
fn missing_identity_file_fails() {
    let temp = TempDir::new().expect("temp dir");
    let prompts = temp.path().join("prompts");
    fs::create_dir_all(prompts.join("broken")).expect("create set dir");
    fs::write(
        prompts.join("registry.yaml"),
        "default_set: broken\n",
    )
    .expect("write registry");
    fs::write(
        prompts.join("broken/router.yaml"),
        "meta: { id: broken, version: \"1.0.0\" }\ntemplate:\n  system: x\n  developer: y\n  assembly: z\nplaceholders:\n  declared: []\n  missing_policy: empty\n",
    )
    .expect("write router");
    fs::write(
        prompts.join("broken/response.yaml"),
        "meta: { id: broken, version: \"1.0.0\" }\ntemplate:\n  system: x\n  developer: y\n  assembly: z\nplaceholders:\n  declared: []\n  missing_policy: empty\n",
    )
    .expect("write response");

    let err = match PromptRegistry::load(temp.path()) {
        Ok(_) => panic!("expected load to fail"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("identity.yaml") || msg.contains("MissingFile"));
}

#[test]
fn invalid_yaml_fails() {
    let temp = TempDir::new().expect("temp dir");
    let prompts = temp.path().join("prompts");
    fs::create_dir_all(prompts.join("bad")).expect("create set dir");
    fs::write(prompts.join("registry.yaml"), "default_set: bad\n").expect("registry");
    fs::write(prompts.join("bad/identity.yaml"), "not: [valid: yaml").expect("bad yaml");
    fs::write(
        prompts.join("bad/router.yaml"),
        "meta: { id: bad, version: \"1.0.0\" }\ntemplate:\n  system: x\n  developer: y\n  assembly: z\nplaceholders:\n  declared: []\n  missing_policy: empty\n",
    )
    .expect("router");
    fs::write(
        prompts.join("bad/response.yaml"),
        "meta: { id: bad, version: \"1.0.0\" }\ntemplate:\n  system: x\n  developer: y\n  assembly: z\nplaceholders:\n  declared: []\n  missing_policy: empty\n",
    )
    .expect("response");

    let err = match PromptRegistry::load(temp.path()) {
        Ok(_) => panic!("expected load to fail"),
        Err(e) => e,
    };
    assert!(err.to_string().contains("InvalidYaml") || err.to_string().contains("yaml"));
}
