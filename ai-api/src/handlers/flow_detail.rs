use axum::Json;
use axum::extract::{Path, State};

use ai_flow_runtime::FlowExport;

use crate::dto::{
    FlowDetailResponse, FlowEdgeDto, FlowGraphEdgeDto, FlowGraphResponse, FlowNodeDto,
    FlowStatsDto,
};
use crate::errors::ApiError;
use crate::state::{with_runtime_read, AppState};

fn map_flow(export: FlowExport) -> FlowDetailResponse {
    FlowDetailResponse {
        id: export.id,
        name: export.name,
        version: export.version,
        initial: export.initial,
        stats: FlowStatsDto {
            node_count: export.stats.node_count,
            end_node_count: export.stats.end_node_count,
        },
        nodes: export
            .nodes
            .into_iter()
            .map(|n| FlowNodeDto {
                id: n.id,
                node_type: n.node_type,
                payload: n.payload,
                transitions: n
                    .transitions
                    .into_iter()
                    .map(|t| FlowEdgeDto {
                        action: t.action,
                        target: t.target,
                    })
                    .collect(),
            })
            .collect(),
    }
}

pub async fn get_flow_detail(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<FlowDetailResponse>, ApiError> {
    let export = with_runtime_read(&state, move |rt| rt.export_flow(&flow_id)).await?;
    Ok(Json(map_flow(export)))
}

pub async fn get_flow_graph(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<Json<FlowGraphResponse>, ApiError> {
    let export = with_runtime_read(&state, move |rt| rt.export_flow(&flow_id)).await?;
    let detail = map_flow(export.clone());
    let mut edges = Vec::new();
    for node in &export.nodes {
        for t in &node.transitions {
            edges.push(FlowGraphEdgeDto {
                from: node.id.clone(),
                to: t.target.clone(),
                action: t.action.clone(),
            });
        }
    }
    Ok(Json(FlowGraphResponse {
        flow_id: detail.id,
        initial: detail.initial,
        nodes: detail.nodes,
        edges,
    }))
}
