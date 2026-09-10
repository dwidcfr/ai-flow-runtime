use crate::types::{MissingPlaceholderPolicy, RenderVars};

pub fn extract_placeholders(template: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("}}") {
            let key = rest[..end].trim();
            if !key.is_empty() && !placeholders.iter().any(|p| p == key) {
                placeholders.push(key.to_string());
            }
            rest = &rest[end + 2..];
        } else {
            break;
        }
    }
    placeholders
}

pub struct TemplateRenderer {
    missing_policy: MissingPlaceholderPolicy,
}

impl TemplateRenderer {
    pub fn new(missing_policy: MissingPlaceholderPolicy) -> Self {
        Self { missing_policy }
    }

    pub fn render(&self, template: &str, vars: &RenderVars) -> String {
        let mut result = template.to_string();
        for key in extract_placeholders(&result) {
            let placeholder = format!("{{{{{key}}}}}");
            let value = match key.as_str() {
                "system" | "developer" => continue,
                _ => vars.get(&key),
            };
            let replacement = match value {
                Some(v) => v.to_string(),
                None => match self.missing_policy {
                    MissingPlaceholderPolicy::Empty => String::new(),
                    MissingPlaceholderPolicy::Keep => placeholder.clone(),
                    MissingPlaceholderPolicy::Error => String::new(),
                },
            };
            result = result.replace(&placeholder, &replacement);
        }

        // Second pass for system/developer if present in assembly
        if let Some(system) = vars.get("system") {
            result = result.replace("{{system}}", system);
        } else {
            result = result.replace("{{system}}", "");
        }
        if let Some(developer) = vars.get("developer") {
            result = result.replace("{{developer}}", developer);
        } else {
            result = result.replace("{{developer}}", "");
        }

        result
    }
}

pub fn render_template(
    template: &crate::types::PromptTemplate,
    vars: &RenderVars,
) -> String {
    let mut enriched = vars.clone();
    enriched.insert("system", template.system.clone());
    enriched.insert("developer", template.developer.clone());
    let renderer = TemplateRenderer::new(template.placeholders.missing_policy);
    renderer.render(&template.assembly, &enriched)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_known_placeholder() {
        let mut vars = RenderVars::default();
        vars.insert("identity.name", "Алиса");
        let renderer = TemplateRenderer::new(MissingPlaceholderPolicy::Empty);
        let out = renderer.render("Hello {{identity.name}}!", &vars);
        assert_eq!(out, "Hello Алиса!");
    }

    #[test]
    fn missing_placeholder_becomes_empty() {
        let vars = RenderVars::default();
        let renderer = TemplateRenderer::new(MissingPlaceholderPolicy::Empty);
        let out = renderer.render("Value: {{missing.key}}", &vars);
        assert_eq!(out, "Value: ");
    }

    #[test]
    fn missing_placeholder_kept_when_policy_keep() {
        let vars = RenderVars::default();
        let renderer = TemplateRenderer::new(MissingPlaceholderPolicy::Keep);
        let out = renderer.render("Value: {{missing.key}}", &vars);
        assert_eq!(out, "Value: {{missing.key}}");
    }
}
