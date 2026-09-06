use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::report::read_signing_key;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u8,
    pub name: String,
    pub rto_seconds: u64,
    pub signing_key: PathBuf,
    pub compose: Compose,
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    pub steps: Vec<Step>,
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Compose {
    pub file: PathBuf,
    pub project: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Step {
    ComposeUp {
        id: String,
        services: Vec<String>,
        #[serde(default = "default_timeout")]
        timeout_seconds: u64,
    },
    Copy {
        id: String,
        artifact: String,
        service: String,
        destination: String,
        #[serde(default = "default_timeout")]
        timeout_seconds: u64,
    },
    Exec {
        id: String,
        service: String,
        argv: Vec<String>,
        #[serde(default = "default_timeout")]
        timeout_seconds: u64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Check {
    Health {
        id: String,
        service: String,
        #[serde(default = "default_timeout")]
        timeout_seconds: u64,
    },
    RowCount {
        id: String,
        service: String,
        argv: Vec<String>,
        min: u64,
        max: Option<u64>,
        #[serde(default = "default_timeout")]
        timeout_seconds: u64,
    },
    ExitCode {
        id: String,
        service: String,
        argv: Vec<String>,
        #[serde(default = "default_timeout")]
        timeout_seconds: u64,
    },
}

fn default_timeout() -> u64 {
    120
}

impl Step {
    pub fn id(&self) -> &str {
        match self {
            Step::ComposeUp { id, .. } | Step::Copy { id, .. } | Step::Exec { id, .. } => id,
        }
    }
}

impl Check {
    pub fn id(&self) -> &str {
        match self {
            Check::Health { id, .. } | Check::RowCount { id, .. } | Check::ExitCode { id, .. } => {
                id
            }
        }
    }
}

pub fn load_manifest(path: &Path) -> Result<Manifest, String> {
    let source =
        fs::read_to_string(path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let manifest: Manifest =
        toml::from_str(&source).map_err(|e| format!("invalid manifest {}: {e}", path.display()))?;
    manifest.validate(path.parent().unwrap_or_else(|| Path::new(".")))?;
    Ok(manifest)
}

impl Manifest {
    pub fn validate(&self, base: &Path) -> Result<(), String> {
        if self.version != 1 {
            return Err(format!(
                "unsupported manifest version {}; expected 1",
                self.version
            ));
        }
        validate_label("name", &self.name)?;
        validate_label("compose.project", &self.compose.project)?;
        let project = self.compose.project.to_ascii_lowercase();
        if ["prod", "production", "live", "primary"]
            .iter()
            .any(|term| project.contains(term))
        {
            return Err(
                "compose.project looks production-like; use a disposable rehearsal project".into(),
            );
        }
        if !["rehearsal", "restore", "drill", "rrc"]
            .iter()
            .any(|term| project.contains(term))
        {
            return Err("compose.project must contain rehearsal, restore, drill, or rrc".into());
        }
        if self.rto_seconds == 0 {
            return Err("rto_seconds must be greater than zero".into());
        }
        if self.steps.is_empty() || self.checks.is_empty() {
            return Err("declare at least one step and one check".into());
        }
        let compose_path = base.join(&self.compose.file);
        if !compose_path.is_file() {
            return Err(format!(
                "compose.file does not exist: {}",
                compose_path.display()
            ));
        }
        validate_compose_file(&compose_path)?;
        read_signing_key(&base.join(&self.signing_key))?;
        let mut ids = HashSet::new();
        let mut artifacts = HashSet::new();
        for artifact in &self.artifacts {
            validate_id("artifact", &artifact.id)?;
            if !artifacts.insert(artifact.id.as_str()) {
                return Err(format!("duplicate artifact id: {}", artifact.id));
            }
            let path = base.join(&artifact.path);
            if !path.is_file() {
                return Err(format!("artifact does not exist: {}", path.display()));
            }
        }
        for step in &self.steps {
            validate_id("step", step.id())?;
            if !ids.insert(step.id()) {
                return Err(format!("duplicate step/check id: {}", step.id()));
            }
            match step {
                Step::ComposeUp {
                    services,
                    timeout_seconds,
                    ..
                } => {
                    if services.is_empty() {
                        return Err("compose-up requires services".into());
                    }
                    validate_timeout(*timeout_seconds)?;
                    for service in services {
                        validate_service(service)?;
                    }
                }
                Step::Copy {
                    artifact,
                    service,
                    destination,
                    timeout_seconds,
                    ..
                } => {
                    if !artifacts.contains(artifact.as_str()) {
                        return Err(format!("copy references unknown artifact: {artifact}"));
                    }
                    validate_service(service)?;
                    if !destination.starts_with("/tmp/restore-rehearsal/")
                        || destination.contains("..")
                    {
                        return Err("copy destination must be below /tmp/restore-rehearsal/".into());
                    }
                    validate_timeout(*timeout_seconds)?;
                }
                Step::Exec {
                    service,
                    argv,
                    timeout_seconds,
                    ..
                } => {
                    validate_service(service)?;
                    validate_argv(argv)?;
                    validate_timeout(*timeout_seconds)?;
                }
            }
        }
        for check in &self.checks {
            validate_id("check", check.id())?;
            if !ids.insert(check.id()) {
                return Err(format!("duplicate step/check id: {}", check.id()));
            }
            match check {
                Check::Health {
                    service,
                    timeout_seconds,
                    ..
                } => {
                    validate_service(service)?;
                    validate_timeout(*timeout_seconds)?;
                }
                Check::RowCount {
                    service,
                    argv,
                    min,
                    max,
                    timeout_seconds,
                    ..
                } => {
                    validate_service(service)?;
                    validate_argv(argv)?;
                    validate_timeout(*timeout_seconds)?;
                    if max.is_some_and(|value| value < *min) {
                        return Err("row-count max must be greater than or equal to min".into());
                    }
                }
                Check::ExitCode {
                    service,
                    argv,
                    timeout_seconds,
                    ..
                } => {
                    validate_service(service)?;
                    validate_argv(argv)?;
                    validate_timeout(*timeout_seconds)?;
                }
            }
        }
        Ok(())
    }
}

/// `check` is deliberately Docker-free, but it must still reject a compose
/// document Docker could never read. Docker performs the authoritative
/// Compose-schema validation immediately before a run.
fn validate_compose_file(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("could not read compose.file {}: {e}", path.display()))?;
    let document: serde_yaml::Value = serde_yaml::from_str(&source)
        .map_err(|e| format!("invalid Compose YAML {}: {e}", path.display()))?;
    let services = document
        .as_mapping()
        .and_then(|map| map.get(serde_yaml::Value::String("services".into())))
        .and_then(serde_yaml::Value::as_mapping);
    if services.is_none_or(|items| items.is_empty()) {
        return Err(format!(
            "compose.file must declare at least one service: {}",
            path.display()
        ));
    }
    Ok(())
}

fn validate_label(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 80 {
        return Err(format!("{field} must contain 1–80 characters"));
    }
    Ok(())
}

fn validate_id(kind: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 48
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "{kind} id must use 1–48 letters, digits, hyphens, or underscores"
        ));
    }
    Ok(())
}

fn validate_service(value: &str) -> Result<(), String> {
    validate_id("service", value)
}

fn validate_timeout(value: u64) -> Result<(), String> {
    if !(1..=3600).contains(&value) {
        return Err("timeout_seconds must be between 1 and 3600".into());
    }
    Ok(())
}

pub fn validate_argv(argv: &[String]) -> Result<(), String> {
    if argv.is_empty() || argv.len() > 64 || argv.iter().any(|arg| arg.contains('\0')) {
        return Err("argv must contain 1–64 non-NUL arguments".into());
    }
    let executable = Path::new(&argv[0])
        .file_name()
        .and_then(|part| part.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let blocked = [
        "sh",
        "bash",
        "dash",
        "zsh",
        "fish",
        "cmd",
        "powershell",
        "pwsh",
    ];
    if blocked.contains(&executable.as_str()) {
        return Err(format!(
            "shell interpreters are not allowed in argv: {executable}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_shells() {
        assert!(validate_argv(&["sh".into(), "-c".into(), "echo secret".into()]).is_err());
        assert!(validate_argv(&["psql".into(), "--version".into()]).is_ok());
    }

    #[test]
    fn rejects_production_target_before_execution() {
        let manifest = Manifest {
            version: 1,
            name: "Weekly".into(),
            rto_seconds: 60,
            signing_key: "key".into(),
            compose: Compose {
                file: "missing.yml".into(),
                project: "production-restore".into(),
            },
            artifacts: vec![],
            steps: vec![],
            checks: vec![],
        };
        assert!(manifest
            .validate(Path::new("."))
            .unwrap_err()
            .contains("production-like"));
    }

    #[test]
    fn check_rejects_malformed_compose_and_corrupt_key() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join("restore.toml");
        fs::write(dir.path().join("compose.yml"), "services: [not valid\n").unwrap();
        fs::write(dir.path().join("key"), "not a signing key\n").unwrap();
        fs::write(
            &manifest_path,
            r#"version = 1
name = "Check inputs"
rto_seconds = 30
signing_key = "key"
[compose]
file = "compose.yml"
project = "rrc-check-inputs"
[[steps]]
id = "start"
kind = "compose-up"
services = ["database"]
[[checks]]
id = "ready"
kind = "health"
service = "database"
"#,
        )
        .unwrap();
        assert!(load_manifest(&manifest_path)
            .unwrap_err()
            .contains("invalid Compose YAML"));
        fs::write(
            dir.path().join("compose.yml"),
            "services:\n  database:\n    image: postgres\n",
        )
        .unwrap();
        assert!(load_manifest(&manifest_path)
            .unwrap_err()
            .contains("signing key"));
    }
}
