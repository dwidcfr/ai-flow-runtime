use crate::dto::studio::{
    DraftCompanyResponse, DraftFlowDto, DraftKnowledgeResponse, DraftPromptResponse,
    FlowNodeDto, FlowTransitionDto, KnowledgeCategoryDto, KnowledgeDocumentDto,
    PromptTemplateDto,
};
use crate::studio::types::{
    DraftCompany, DraftFlow, DraftFlowNode, DraftFlowTransition, DraftKnowledge,
    DraftPromptSet, DraftPromptTemplate, KnowledgeCategory, KnowledgeDocument, NodePosition,
};

pub fn company_to_dto(c: &DraftCompany) -> DraftCompanyResponse {
    DraftCompanyResponse {
        id: c.id.clone(),
        name: c.name.clone(),
        description: c.description.clone(),
        prompt_set: c.prompt_set.clone(),
        knowledge_source: c.knowledge_source.clone(),
        flows: c.flows.clone(),
    }
}

pub fn company_from_dto(c: &DraftCompanyResponse) -> DraftCompany {
    DraftCompany {
        id: c.id.clone(),
        name: c.name.clone(),
        description: c.description.clone(),
        prompt_set: c.prompt_set.clone(),
        knowledge_source: c.knowledge_source.clone(),
        flows: c.flows.clone(),
    }
}

pub fn flow_to_dto(f: &DraftFlow) -> DraftFlowDto {
    DraftFlowDto {
        id: f.id.clone(),
        name: f.name.clone(),
        version: f.version,
        initial: f.initial.clone(),
        nodes: f.nodes.iter().map(node_to_dto).collect(),
    }
}

pub fn flow_from_dto(f: &DraftFlowDto) -> DraftFlow {
    DraftFlow {
        id: f.id.clone(),
        name: f.name.clone(),
        version: f.version,
        initial: f.initial.clone(),
        nodes: f.nodes.iter().map(node_from_dto).collect(),
    }
}

fn node_to_dto(n: &DraftFlowNode) -> FlowNodeDto {
    FlowNodeDto {
        id: n.id.clone(),
        node_type: n.node_type.clone(),
        payload: n.payload.clone(),
        transitions: n
            .transitions
            .iter()
            .map(|t| FlowTransitionDto {
                action: t.action.clone(),
                target: t.target.clone(),
            })
            .collect(),
        position: n.position.as_ref().map(|p| crate::dto::studio::NodePositionDto {
            x: p.x,
            y: p.y,
        }),
    }
}

fn node_from_dto(n: &FlowNodeDto) -> DraftFlowNode {
    DraftFlowNode {
        id: n.id.clone(),
        node_type: n.node_type.clone(),
        payload: n.payload.clone(),
        transitions: n
            .transitions
            .iter()
            .map(|t| DraftFlowTransition {
                action: t.action.clone(),
                target: t.target.clone(),
            })
            .collect(),
        position: n.position.as_ref().map(|p| NodePosition {
            x: p.x,
            y: p.y,
        }),
    }
}

pub fn knowledge_to_dto(k: &DraftKnowledge) -> DraftKnowledgeResponse {
    DraftKnowledgeResponse {
        categories: k
            .categories
            .iter()
            .map(|c| KnowledgeCategoryDto {
                id: c.id.clone(),
                name: c.name.clone(),
            })
            .collect(),
        documents: k.documents.iter().map(doc_to_dto).collect(),
    }
}

pub fn knowledge_from_dto(k: &DraftKnowledgeResponse) -> DraftKnowledge {
    DraftKnowledge {
        categories: k
            .categories
            .iter()
            .map(|c| KnowledgeCategory {
                id: c.id.clone(),
                name: c.name.clone(),
            })
            .collect(),
        documents: k.documents.iter().map(doc_from_dto).collect(),
    }
}

fn doc_to_dto(d: &KnowledgeDocument) -> KnowledgeDocumentDto {
    KnowledgeDocumentDto {
        id: d.id.clone(),
        category_id: d.category_id.clone(),
        title: d.title.clone(),
        section: d.section.clone(),
        content: d.content.clone(),
        metadata: d.metadata.clone(),
    }
}

fn doc_from_dto(d: &KnowledgeDocumentDto) -> KnowledgeDocument {
    KnowledgeDocument {
        id: d.id.clone(),
        category_id: d.category_id.clone(),
        title: d.title.clone(),
        section: d.section.clone(),
        content: d.content.clone(),
        metadata: d.metadata.clone(),
    }
}

pub fn prompt_to_dto(p: &DraftPromptSet) -> DraftPromptResponse {
    DraftPromptResponse {
        id: p.id.clone(),
        name: p.name.clone(),
        version: p.version.clone(),
        identity: p.identity.clone(),
        router: template_to_dto(&p.router),
        response: template_to_dto(&p.response),
    }
}

pub fn prompt_from_dto(p: &DraftPromptResponse) -> DraftPromptSet {
    DraftPromptSet {
        id: p.id.clone(),
        name: p.name.clone(),
        version: p.version.clone(),
        identity: p.identity.clone(),
        router: template_from_dto(&p.router),
        response: template_from_dto(&p.response),
    }
}

pub fn template_to_dto(t: &DraftPromptTemplate) -> PromptTemplateDto {
    PromptTemplateDto {
        system: t.system.clone(),
        developer: t.developer.clone(),
        assembly: t.assembly.clone(),
    }
}

pub fn template_from_dto(t: &PromptTemplateDto) -> DraftPromptTemplate {
    DraftPromptTemplate {
        system: t.system.clone(),
        developer: t.developer.clone(),
        assembly: t.assembly.clone(),
    }
}
