use std::fs;
use std::sync::{Arc, RwLock};

use ai_prompts::{bundled_prompts_root, PromptRegistry};
use tempfile::TempDir;

#[test]
fn loads_demo_prompt_set() {
    let registry = PromptRegistry::load(bundled_prompts_root()).expect("registry loads");
    let ids: Vec<_> = registry.list_sets().into_iter().map(|m| m.id).collect();
    assert!(ids.contains(&"demo".to_string()));
    assert_eq!(registry.default_set_id(), "demo");
}

#[test]
fn demo_identity_has_expected_fields() {
    let registry = PromptRegistry::load(bundled_prompts_root()).expect("registry loads");
    let demo = registry.identity("demo").expect("demo identity");
    assert_eq!(demo.name, "Alex");
    assert_eq!(demo.bank_name, "Acme Insurance");
    assert!(!demo.tone.is_empty());
}

#[test]
fn reload_picks_up_changed_yaml() {
    let source = bundled_prompts_root();
    let temp = TempDir::new().expect("temp dir");
    let prompts_root = temp.path().join("prompts");
    copy_dir_recursive(&source.join("prompts"), &prompts_root);

    let mut registry = PromptRegistry::load(temp.path()).expect("registry loads");
    assert_eq!(registry.identity("demo").unwrap().name, "Alex");

    let identity_path = prompts_root.join("demo/identity.yaml");
    let mut content = fs::read_to_string(&identity_path).expect("read identity");
    content = content.replace("Alex", "TestAlex");
    fs::write(&identity_path, content).expect("write identity");

    registry.reload().expect("reload");
    assert_eq!(registry.identity("demo").unwrap().name, "TestAlex");
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) {
    fs::create_dir_all(dst).expect("create dst");
    for entry in fs::read_dir(src).expect("read src") {
        let entry = entry.expect("entry");
        let target = dst.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy file");
        }
    }
}

#[test]
fn registry_is_shareable_via_arc_rwlock() {
    let registry = PromptRegistry::load(bundled_prompts_root()).expect("registry loads");
    let shared = Arc::new(RwLock::new(registry));
    let guard = shared.read().expect("read lock");
    assert!(guard.has_set("demo"));
}
