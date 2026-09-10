use std::path::Path;

use crate::error::{PromptError, Result};
use crate::types::PromptTemplate;

pub fn validate_template(template: &PromptTemplate, path: &Path, require_context_json: bool) -> Result<()> {
    if require_context_json && !template.assembly.contains("{{context_json}}") {
        return Err(PromptError::MissingField {
            field: "template.assembly must contain {{context_json}}".into(),
            path: path.display().to_string(),
        });
    }

    let found = crate::renderer::extract_placeholders(&template.assembly);
    let declared: std::collections::HashSet<_> = template
        .placeholders
        .declared
        .iter()
        .map(|s| s.as_str())
        .collect();

    for placeholder in &found {
        if placeholder == "system" || placeholder == "developer" {
            continue;
        }
        if !declared.contains(placeholder.as_str()) {
            // warning only — not blocking load
            let _ = placeholder;
        }
    }

    Ok(())
}
