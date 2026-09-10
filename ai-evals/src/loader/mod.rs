use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{EvalError, Result};
use crate::scenario::{EvalScenario, FixturesManifest};

pub struct ScenarioLoader {
    fixtures: FixturesManifest,
    evals_root: PathBuf,
}

impl ScenarioLoader {
    pub fn new(crate_root: &Path) -> Result<Self> {
        let fixtures_path = crate_root.join("fixtures.yaml");
        let evals_root = crate_root.join("evals");
        Ok(Self {
            fixtures: FixturesManifest::load_from(&fixtures_path)?,
            evals_root,
        })
    }

    pub fn fixtures(&self) -> &FixturesManifest {
        &self.fixtures
    }

    pub fn discover_all(&self) -> Result<Vec<EvalScenario>> {
        let mut scenarios = Vec::new();
        self.discover_dir(&self.evals_root, &mut scenarios)?;
        self.ensure_unique_names(&scenarios)?;
        scenarios.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(scenarios)
    }

    pub fn load_by_name(&self, name: &str) -> Result<EvalScenario> {
        let scenarios = self.discover_all()?;
        scenarios
            .into_iter()
            .find(|s| s.name == name || s.name.contains(name))
            .ok_or_else(|| EvalError::ScenarioNotFound(name.into()))
    }

    pub fn load_by_domain(&self, domain: &str) -> Result<Vec<EvalScenario>> {
        let scenarios = self.discover_all()?;
        Ok(scenarios
            .into_iter()
            .filter(|s| {
                s.domain.as_deref() == Some(domain)
                    || s.source_path.to_string_lossy().contains(domain)
            })
            .collect())
    }

    fn discover_dir(&self, dir: &Path, out: &mut Vec<EvalScenario>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.discover_dir(&path, out)?;
            } else if is_yaml(&path) {
                out.push(self.load_file(&path)?);
            }
        }
        Ok(())
    }

    fn load_file(&self, path: &Path) -> Result<EvalScenario> {
        let content = fs::read_to_string(path)?;
        let mut scenario: EvalScenario = serde_yaml::from_str(&content).map_err(|e| EvalError::Load {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        scenario.source_path = path.to_path_buf();
        scenario.validate()?;
        Ok(scenario)
    }

    fn ensure_unique_names(&self, scenarios: &[EvalScenario]) -> Result<()> {
        let mut seen = HashMap::new();
        for scenario in scenarios {
            if let Some(prev) = seen.insert(&scenario.name, &scenario.source_path) {
                return Err(EvalError::DuplicateScenario(format!(
                    "{} ({} and {})",
                    scenario.name,
                    prev.display(),
                    scenario.source_path.display()
                )));
            }
        }
        Ok(())
    }
}

fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e == "yaml" || e == "yml")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crate_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn discovers_scenarios() {
        let loader = ScenarioLoader::new(&crate_root()).unwrap();
        let scenarios = loader.discover_all().unwrap();
        assert!(!scenarios.is_empty());
    }

    #[test]
    fn loads_by_name() {
        let loader = ScenarioLoader::new(&crate_root()).unwrap();
        let scenario = loader.load_by_name("greeting_confirm").unwrap();
        assert_eq!(scenario.name, "greeting_confirm");
    }
}
