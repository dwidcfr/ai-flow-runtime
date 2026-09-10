use std::path::PathBuf;

use ai_flow_runtime::{
    Runtime, COMPANY_MODULE_ID, CONVERSATION_MODULE_ID, SMALLTALK_MODULE_ID,
};

fn data_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn setup_session() -> (Runtime, String) {
    let mut rt = Runtime::new();
    let flow_id = rt
        .load_flow(data_path("data/flows/payment_flow.yaml").to_str().unwrap())
        .unwrap();
    let session_id = rt
        .create_session(
            &flow_id,
            data_path("data/clients/sample.json").to_str().unwrap(),
        )
        .unwrap();
    (rt, session_id)
}

#[test]
fn search_company_module_finds_payment_methods() {
    let (mut rt, session_id) = setup_session();

    rt.load_session_module(
        &session_id,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();

    let results = rt
        .search_session(&session_id, "Click payment", Some(&[COMPANY_MODULE_ID.to_string()]), 3)
        .unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].module_id, COMPANY_MODULE_ID);
    assert_eq!(results[0].section_id.as_deref(), Some("payment_methods"));
    assert!(results[0].score > 0.0);
}

#[test]
fn conversation_entry_is_searchable_after_reindex() {
    let (mut rt, session_id) = setup_session();

    rt.set_conversation_entry(&session_id, "payment_promise", "обещал оплатить вечером")
        .unwrap();

    let results = rt
        .search_session(
            &session_id,
            "оплатить вечером",
            Some(&[CONVERSATION_MODULE_ID.to_string()]),
            3,
        )
        .unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].module_id, CONVERSATION_MODULE_ID);
    assert_eq!(results[0].section_id.as_deref(), Some("payment_promise"));
}

#[test]
fn smalltalk_module_search_finds_greetings() {
    let (mut rt, session_id) = setup_session();

    rt.load_session_module(
        &session_id,
        SMALLTALK_MODULE_ID,
        data_path("data/modules/smalltalk.yaml").to_str().unwrap(),
    )
    .unwrap();

    let results = rt
        .search_session(
            &session_id,
            "Добрый день привет",
            Some(&[SMALLTALK_MODULE_ID.to_string()]),
            3,
        )
        .unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].module_id, SMALLTALK_MODULE_ID);
    assert_eq!(results[0].section_id.as_deref(), Some("greetings"));
}

#[test]
fn multi_module_search_returns_results_from_both() {
    let (mut rt, session_id) = setup_session();

    rt.load_session_module(
        &session_id,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();
    rt.load_session_module(
        &session_id,
        SMALLTALK_MODULE_ID,
        data_path("data/modules/smalltalk.yaml").to_str().unwrap(),
    )
    .unwrap();

    let results = rt.search_session(&session_id, "оплата день", None, 5).unwrap();

    let modules: Vec<&str> = results.iter().map(|r| r.module_id.as_str()).collect();
    assert!(modules.contains(&COMPANY_MODULE_ID));
    assert!(modules.contains(&SMALLTALK_MODULE_ID));
}

#[test]
fn empty_index_returns_no_results() {
    let (rt, session_id) = setup_session();

    let results = rt.search_session(&session_id, "Click", None, 5).unwrap();
    assert!(results.is_empty());
}

#[test]
fn build_session_index_indexes_all_loaded_modules() {
    let (mut rt, session_id) = setup_session();

    rt.load_session_module(
        &session_id,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();
    rt.load_session_module(
        &session_id,
        SMALLTALK_MODULE_ID,
        data_path("data/modules/smalltalk.yaml").to_str().unwrap(),
    )
    .unwrap();

    rt.build_session_index(&session_id).unwrap();

    let results = rt.search_session(&session_id, "Click Payme", None, 5).unwrap();
    assert!(!results.is_empty());
    assert!(results[0].score >= results.last().unwrap().score);
}

#[test]
fn search_results_include_source_metadata() {
    let (mut rt, session_id) = setup_session();

    rt.load_session_module(
        &session_id,
        COMPANY_MODULE_ID,
        data_path("data/company/acme.yaml").to_str().unwrap(),
    )
    .unwrap();

    let results = rt
        .search_session(&session_id, "Click", Some(&[COMPANY_MODULE_ID.to_string()]), 1)
        .unwrap();

    let result = &results[0];
    assert!(!result.document_id.is_empty());
    assert!(!result.title.is_empty());
    assert!(!result.snippet.is_empty());
    assert_eq!(result.module_id, COMPANY_MODULE_ID);
}
