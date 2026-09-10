use std::collections::BTreeMap;

use ai_flow_engine::parse::{parse_yaml, RawFlowDocument};
use ai_flow_engine::validate::validate_raw_document;
use serde_json::{json, Value};
use serde_yaml::Mapping;

use crate::studio::types::{
    DraftFlow, DraftFlowNode, DraftKnowledge, DraftPromptSet, KnowledgeDocument,
};

pub fn draft_flow_to_yaml(flow: &DraftFlow) -> String {
    let mut nodes = Mapping::new();
    for node in &flow.nodes {
        let mut node_map = Mapping::new();
        node_map.insert(
            serde_yaml::Value::String("type".into()),
            serde_yaml::Value::String(node.node_type.clone()),
        );
        if let Some(obj) = node.payload.as_object() {
            for (k, v) in obj {
                if k == "type" {
                    continue;
                }
                node_map.insert(
                    serde_yaml::Value::String(k.clone()),
                    json_to_yaml_value(v),
                );
            }
        }
        if !node.transitions.is_empty() {
            let mut trans = Mapping::new();
            for t in &node.transitions {
                trans.insert(
                    serde_yaml::Value::String(t.action.clone()),
                    serde_yaml::Value::String(t.target.clone()),
                );
            }
            node_map.insert(
                serde_yaml::Value::String("transitions".into()),
                serde_yaml::Value::Mapping(trans),
            );
        }
        nodes.insert(
            serde_yaml::Value::String(node.id.clone()),
            serde_yaml::Value::Mapping(node_map),
        );
    }

    let mut root = Mapping::new();
    root.insert(
        serde_yaml::Value::String("id".into()),
        serde_yaml::Value::String(flow.id.clone()),
    );
    if let Some(name) = &flow.name {
        root.insert(
            serde_yaml::Value::String("name".into()),
            serde_yaml::Value::String(name.clone()),
        );
    }
    root.insert(
        serde_yaml::Value::String("version".into()),
        serde_yaml::Value::Number(flow.version.into()),
    );
    root.insert(
        serde_yaml::Value::String("initial".into()),
        serde_yaml::Value::String(flow.initial.clone()),
    );
    root.insert(
        serde_yaml::Value::String("nodes".into()),
        serde_yaml::Value::Mapping(nodes),
    );
    serde_yaml::to_string(&serde_yaml::Value::Mapping(root)).unwrap_or_default()
}

fn json_to_yaml_value(v: &Value) -> serde_yaml::Value {
    serde_yaml::to_value(v).unwrap_or(serde_yaml::Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::studio::types::{DraftFlow, DraftFlowNode, DraftFlowTransition};

    #[test]
    fn validate_rejects_unknown_transition_target() {
        let flow = DraftFlow {
            id: "bad".into(),
            name: None,
            version: 1,
            initial: "start".into(),
            nodes: vec![
                DraftFlowNode {
                    id: "start".into(),
                    node_type: "flow".into(),
                    payload: json!({}),
                    transitions: vec![DraftFlowTransition {
                        action: "go".into(),
                        target: "missing".into(),
                    }],
                    position: None,
                },
                DraftFlowNode {
                    id: "end".into(),
                    node_type: "end".into(),
                    payload: json!({}),
                    transitions: vec![],
                    position: None,
                },
            ],
        };
        let (valid, issues) = validate_draft_flow(&flow);
        assert!(!valid, "expected invalid flow, issues={issues:?}");
    }
}

pub fn validate_draft_flow(flow: &DraftFlow) -> (bool, Vec<crate::dto::studio::ValidationIssueDto>) {
    let yaml = draft_flow_to_yaml(flow);
    let doc: RawFlowDocument = match parse_yaml(&yaml) {
        Ok(d) => d,
        Err(e) => {
            return (
                false,
                vec![crate::dto::studio::ValidationIssueDto {
                    rule: "YamlParse".into(),
                    message: e.to_string(),
                    node_id: None,
                    action: None,
                    target: None,
                }],
            );
        }
    };
    match validate_raw_document(&doc) {
        Ok(()) => (true, Vec::new()),
        Err(ai_flow_engine::FlowError::Validation(issues)) => {
            let blocking: Vec<_> = issues
                .into_iter()
                .filter(|i| {
                    !matches!(
                        i.rule,
                        ai_flow_engine::ValidationRule::EndNodeHasTransitions
                    )
                })
                .map(|i| crate::dto::studio::ValidationIssueDto {
                    rule: format!("{:?}", i.rule),
                    message: i.message,
                    node_id: i.node_id,
                    action: i.action,
                    target: i.target,
                })
                .collect();
            (blocking.is_empty(), blocking)
        }
        Err(e) => (
            false,
            vec![crate::dto::studio::ValidationIssueDto {
                rule: "FlowError".into(),
                message: format!("{e}"),
                node_id: None,
                action: None,
                target: None,
            }],
        ),
    }
}

pub fn knowledge_to_yaml(knowledge: &DraftKnowledge) -> String {
    let mut sections = Mapping::new();
    for doc in &knowledge.documents {
        let mut section = Mapping::new();
        section.insert(
            serde_yaml::Value::String("description".into()),
            serde_yaml::Value::String(doc.title.clone()),
        );
        section.insert(
            serde_yaml::Value::String("content".into()),
            serde_yaml::Value::String(doc.content.clone()),
        );
        if !doc.metadata.is_null() && doc.metadata != json!({}) {
            section.insert(
                serde_yaml::Value::String("metadata".into()),
                json_to_yaml_value(&doc.metadata),
            );
        }
        sections.insert(
            serde_yaml::Value::String(doc.section.clone()),
            serde_yaml::Value::Mapping(section),
        );
    }
    let mut root = Mapping::new();
    root.insert(
        serde_yaml::Value::String("sections".into()),
        serde_yaml::Value::Mapping(sections),
    );
    serde_yaml::to_string(&serde_yaml::Value::Mapping(root)).unwrap_or_default()
}

pub fn yaml_to_knowledge(content: &str) -> DraftKnowledge {
    let mut knowledge = DraftKnowledge::default();
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(content) else {
        return knowledge;
    };
    let Some(sections) = value.get("sections").and_then(|v| v.as_mapping()) else {
        return knowledge;
    };
    let mut categories: BTreeMap<String, String> = BTreeMap::new();
    for (key, section) in sections {
        let section_id = key.as_str().unwrap_or("").to_string();
        let title = section
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or(&section_id)
            .to_string();
        let content = section
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or_default()
            .to_string();
        let metadata = section
            .get("metadata")
            .map(|m| serde_json::to_value(m).unwrap_or(json!({})))
            .unwrap_or(json!({}));
        let category_id = section_id.split('_').next().unwrap_or("general").to_string();
        categories
            .entry(category_id.clone())
            .or_insert_with(|| category_id.clone());
        knowledge.documents.push(KnowledgeDocument {
            id: section_id.clone(),
            category_id,
            title,
            section: section_id,
            content,
            metadata,
        });
    }
    knowledge.categories = categories
        .into_iter()
        .map(|(id, name)| crate::studio::types::KnowledgeCategory { id, name })
        .collect();
    knowledge
}

pub fn export_to_draft_flow(export: &ai_flow_runtime::FlowExport) -> DraftFlow {
    DraftFlow {
        id: export.id.clone(),
        name: export.name.clone(),
        version: export.version.unwrap_or(1),
        initial: export.initial.clone(),
        nodes: export
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| DraftFlowNode {
                id: n.id.clone(),
                node_type: n.node_type.clone(),
                payload: n.payload.clone(),
                transitions: n
                    .transitions
                    .iter()
                    .map(|t| crate::studio::types::DraftFlowTransition {
                        action: t.action.clone(),
                        target: t.target.clone(),
                    })
                    .collect(),
                position: Some(crate::studio::types::NodePosition {
                    x: 80.0 + (i as f64 % 4.0) * 160.0,
                    y: 80.0 + (i as f64 / 4.0).floor() * 100.0,
                }),
            })
            .collect(),
    }
}

pub fn prompt_files_to_draft(
    set_id: &str,
    identity: &serde_yaml::Value,
    router: &serde_yaml::Value,
    response: &serde_yaml::Value,
) -> DraftPromptSet {
    let identity_meta = identity
        .get("meta")
        .cloned()
        .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    let name = identity_meta
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(set_id)
        .to_string();
    let version = identity_meta
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();
    DraftPromptSet {
        id: set_id.to_string(),
        name,
        version,
        identity: identity
            .get("identity")
            .map(|v| serde_json::to_value(v).unwrap_or(json!({})))
            .unwrap_or(json!({})),
        router: template_from_yaml(router),
        response: template_from_yaml(response),
    }
}

fn template_from_yaml(v: &serde_yaml::Value) -> crate::studio::types::DraftPromptTemplate {
    let template = v.get("template").cloned().unwrap_or_else(|| v.clone());
    crate::studio::types::DraftPromptTemplate {
        system: template
            .get("system")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        developer: template
            .get("developer")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        assembly: template
            .get("assembly")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
    }
}
