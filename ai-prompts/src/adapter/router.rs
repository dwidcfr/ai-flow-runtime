use ai_router_engine::{PromptBuilder, RouterContext};
use std::sync::{Arc, RwLock};

use crate::registry::PromptRegistry;
use crate::renderer::render_template;
use crate::types::RenderVars;

pub struct RegistryRouterPromptBuilder {
    registry: Arc<RwLock<PromptRegistry>>,
}

impl RegistryRouterPromptBuilder {
    pub fn new(registry: Arc<RwLock<PromptRegistry>>) -> Self {
        Self { registry }
    }

    fn resolve_set_id<'a>(&self, context: &'a RouterContext) -> Result<String, String> {
        let registry = self.registry.read().map_err(|e| e.to_string())?;
        let set_id = if context.prompt_set_id.is_empty() {
            registry.default_set_id().to_string()
        } else {
            context.prompt_set_id.clone()
        };
        if !registry.has_set(&set_id) {
            return Err(format!("prompt set not found: {set_id}"));
        }
        Ok(set_id)
    }
}

impl PromptBuilder for RegistryRouterPromptBuilder {
    fn build(&self, context: &RouterContext, normalized_message: &str) -> String {
        self.build_inner(context, normalized_message)
            .unwrap_or_else(|e| format!("Prompt build error: {e}"))
    }
}

impl RegistryRouterPromptBuilder {
    fn build_inner(
        &self,
        context: &RouterContext,
        normalized_message: &str,
    ) -> Result<String, String> {
        let set_id = self.resolve_set_id(context)?;
        let registry = self.registry.read().map_err(|e| e.to_string())?;
        let template = registry
            .router_template(&set_id)
            .map_err(|e| e.to_string())?;
        let identity = registry.identity(&set_id).map_err(|e| e.to_string())?;

        let context_json = context::build_router_context_json(context, normalized_message);
        let mut vars = RenderVars::default();
        vars.insert("context_json", context_json);
        vars.insert("flow.node", context.current_node.clone());
        vars.insert(
            "client.name",
            context
                .client_data
                .get("client_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        );
        vars.insert("identity.name", identity.name);

        Ok(render_template(template, &vars))
    }
}

mod context {
    use ai_router_engine::RouterContext;
    use serde_json::json;

    const MAX_HISTORY_ENTRIES: usize = 10;

    pub fn build_router_context_json(ctx: &RouterContext, normalized_message: &str) -> String {
        let history: Vec<_> = ctx
            .history
            .iter()
            .rev()
            .take(MAX_HISTORY_ENTRIES)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|entry| {
                json!({
                    "role": format!("{:?}", entry.role),
                    "content": entry.content,
                })
            })
            .collect();

        let search_candidates: Vec<_> = ctx
            .search_candidates
            .iter()
            .map(|candidate| {
                json!({
                    "document_id": candidate.document_id,
                    "module_id": candidate.module_id,
                    "section_id": candidate.section_id,
                    "score": candidate.score,
                    "title": candidate.title,
                    "snippet": candidate.snippet,
                })
            })
            .collect();

        let payload = json!({
            "user_message": normalized_message,
            "current_node": ctx.current_node,
            "available_actions": ctx.available_actions,
            "client_data": ctx.client_data,
            "conversation_context": ctx.conversation_context,
            "search_candidates": search_candidates,
            "history": history,
        });

        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    }
}
