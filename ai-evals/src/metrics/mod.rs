use std::time::Duration;

use serde::Serialize;

use crate::runner::{ScenarioResult, ScenarioStatus, ScenarioTimings};

#[derive(Debug, Clone, Serialize)]
pub struct RunMetrics {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub router_errors: usize,
    pub response_errors: usize,
    pub duration_ms: u64,
    pub avg_scenario_ms: f64,
    pub avg_router_ms: f64,
    pub avg_response_ms: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub by_domain: Vec<DomainMetrics>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainMetrics {
    pub domain: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

pub fn collect_metrics(results: &[ScenarioResult], duration: Duration) -> RunMetrics {
    let total = results.len();
    let passed = results
        .iter()
        .filter(|r| r.status == ScenarioStatus::Passed)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == ScenarioStatus::Failed)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == ScenarioStatus::Skipped)
        .count();
    let router_errors = results.iter().filter(|r| r.router_error).count();
    let response_errors = results.iter().filter(|r| r.response_error).count();

    let timed: Vec<&ScenarioTimings> = results
        .iter()
        .filter_map(|r| r.timings.as_ref())
        .collect();

    let avg = |f: fn(&ScenarioTimings) -> u64| -> f64 {
        if timed.is_empty() {
            0.0
        } else {
            timed.iter().map(|t| f(t)).sum::<u64>() as f64 / timed.len() as f64
        }
    };

    RunMetrics {
        total,
        passed,
        failed,
        skipped,
        router_errors,
        response_errors,
        duration_ms: duration.as_millis() as u64,
        avg_scenario_ms: avg(|t| t.total_ms),
        avg_router_ms: avg(|t| t.router_ms),
        avg_response_ms: avg(|t| t.response_ms),
        by_domain: collect_domain_metrics(results),
    }
}

fn collect_domain_metrics(results: &[ScenarioResult]) -> Vec<DomainMetrics> {
    let mut domains: Vec<String> = results
        .iter()
        .filter_map(|r| r.domain.clone())
        .collect();
    domains.sort();
    domains.dedup();

    domains
        .into_iter()
        .map(|domain| {
            let subset: Vec<_> = results
                .iter()
                .filter(|r| r.domain.as_deref() == Some(domain.as_str()))
                .collect();
            DomainMetrics {
                domain: domain.clone(),
                total: subset.len(),
                passed: subset
                    .iter()
                    .filter(|r| r.status == ScenarioStatus::Passed)
                    .count(),
                failed: subset
                    .iter()
                    .filter(|r| r.status == ScenarioStatus::Failed)
                    .count(),
                skipped: subset
                    .iter()
                    .filter(|r| r.status == ScenarioStatus::Skipped)
                    .count(),
            }
        })
        .collect()
}
