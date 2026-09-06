use crate::config::{Check, Manifest, Step};
use crate::report::{read_signing_key, ArtifactEvidence, Card, Evidence};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use wait_timeout::ChildExt;

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub manifest_path: PathBuf,
    pub confirm_target: String,
    pub output_path: PathBuf,
    pub keep_target: bool,
}

#[derive(Debug, Serialize)]
pub struct RunResult {
    pub passed: bool,
    pub card: PathBuf,
    pub target: String,
    pub duration_ms: u128,
    pub rto_met: bool,
}

pub fn run_rehearsal(manifest: &Manifest, options: &RunOptions) -> Result<RunResult, String> {
    if options.confirm_target != manifest.compose.project {
        return Err(format!(
            "target confirmation mismatch; pass --confirm-target {} exactly",
            manifest.compose.project
        ));
    }
    let base = options
        .manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let key = read_signing_key(&base.join(&manifest.signing_key))?;
    preflight_compose(manifest, base)?;
    let started_wall = SystemTime::now();
    let started = Instant::now();
    let mut evidence = Vec::new();
    let artifacts = hash_artifacts(manifest, base)?;
    let mut failure = None;

    for step in &manifest.steps {
        let timer = Instant::now();
        let result = run_step(manifest, base, step);
        let (passed, detail) = match result {
            Ok(detail) => (true, detail),
            Err(error) => {
                failure = Some(format!("step `{}` failed: {error}", step.id()));
                (false, error)
            }
        };
        evidence.push(Evidence {
            id: step.id().to_string(),
            kind: step_kind(step).into(),
            passed,
            duration_ms: timer.elapsed().as_millis(),
            detail,
        });
        if !passed {
            break;
        }
    }
    if failure.is_none() {
        for check in &manifest.checks {
            let timer = Instant::now();
            let result = run_check(manifest, base, check);
            let (passed, detail) = match result {
                Ok(detail) => (true, detail),
                Err(error) => {
                    failure = Some(format!("check `{}` failed: {error}", check.id()));
                    (false, error)
                }
            };
            evidence.push(Evidence {
                id: check.id().to_string(),
                kind: check_kind(check).into(),
                passed,
                duration_ms: timer.elapsed().as_millis(),
                detail,
            });
        }
    }

    if !options.keep_target {
        if let Err(error) = cleanup(manifest, base) {
            if failure.is_none() {
                failure = Some(format!("cleanup failed: {error}"));
            }
            evidence.push(Evidence {
                id: "cleanup".into(),
                kind: "compose-down".into(),
                passed: false,
                duration_ms: 0,
                detail: error,
            });
        }
    }
    let duration_ms = started.elapsed().as_millis();
    let rto_met = duration_ms <= manifest.rto_seconds as u128 * 1_000;
    if !rto_met && failure.is_none() {
        failure = Some(format!(
            "RTO missed: {:.2}s exceeds {}s",
            duration_ms as f64 / 1_000.0,
            manifest.rto_seconds
        ));
    }
    let finished_wall = SystemTime::now();
    let card = Card {
        rehearsal: manifest.name.clone(),
        target: manifest.compose.project.clone(),
        started_at: timestamp(started_wall)?,
        finished_at: timestamp(finished_wall)?,
        duration_ms,
        rto_seconds: manifest.rto_seconds,
        passed: failure.is_none(),
        artifacts,
        evidence,
        failure,
    };
    let signed = card.sign(&key);
    write_card(&options.output_path, &signed)?;
    Ok(RunResult {
        passed: card.passed,
        card: options.output_path.clone(),
        target: manifest.compose.project.clone(),
        duration_ms,
        rto_met,
    })
}

fn hash_artifacts(manifest: &Manifest, base: &Path) -> Result<Vec<ArtifactEvidence>, String> {
    manifest
        .artifacts
        .iter()
        .map(|artifact| {
            let path = base.join(&artifact.path);
            let mut file = File::open(&path)
                .map_err(|e| format!("could not open artifact {}: {e}", artifact.id))?;
            let mut hash = Sha256::new();
            let mut bytes = 0_u64;
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let read = file
                    .read(&mut buffer)
                    .map_err(|e| format!("could not hash artifact {}: {e}", artifact.id))?;
                if read == 0 {
                    break;
                }
                hash.update(&buffer[..read]);
                bytes += read as u64;
            }
            Ok(ArtifactEvidence {
                id: artifact.id.clone(),
                bytes,
                sha256: format!("{:x}", hash.finalize()),
            })
        })
        .collect()
}

fn preflight_compose(manifest: &Manifest, base: &Path) -> Result<(), String> {
    let mut args = compose_prefix(manifest, base);
    args.extend(["config".into(), "--format".into(), "json".into()]);
    let output = execute("docker", &args, 60)?;
    if !output.status.success() {
        return Err("Docker Compose could not validate the rehearsal target".into());
    }
    let config: Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| "Docker Compose returned invalid config JSON".to_string())?;
    reject_unsafe_compose(&config, &manifest.compose.project)
}

const UNSAFE_COMPOSE_PREFIX: &str = "unsafe Compose isolation:";

fn unsafe_compose(message: impl AsRef<str>) -> String {
    format!("{UNSAFE_COMPOSE_PREFIX} {}", message.as_ref())
}

fn has_items(value: Option<&Value>) -> bool {
    value.is_some_and(|item| match item {
        Value::Array(items) => !items.is_empty(),
        Value::Object(items) => !items.is_empty(),
        Value::Null => false,
        _ => true,
    })
}

fn reject_unsafe_compose(config: &Value, project: &str) -> Result<(), String> {
    if let Some(services) = config.get("services").and_then(Value::as_object) {
        for (name, service) in services {
            if service
                .get("network_mode")
                .and_then(Value::as_str)
                .is_some_and(|mode| !mode.is_empty() && mode != "none")
            {
                return Err(unsafe_compose(format!(
                    "service {name} overrides its project network namespace"
                )));
            }
            for namespace in ["pid", "ipc", "uts", "userns_mode", "cgroup"] {
                if service
                    .get(namespace)
                    .and_then(Value::as_str)
                    .is_some_and(|mode| !mode.is_empty() && mode != "private")
                {
                    return Err(unsafe_compose(format!(
                        "service {name} overrides its {namespace} namespace"
                    )));
                }
            }
            if service.get("privileged").and_then(Value::as_bool) == Some(true) {
                return Err(unsafe_compose(format!("service {name} is privileged")));
            }
            if service
                .get("volumes")
                .and_then(Value::as_array)
                .is_some_and(|volumes| {
                    volumes
                        .iter()
                        .any(|volume| volume.get("type").and_then(Value::as_str) == Some("bind"))
                })
            {
                return Err(unsafe_compose(format!(
                    "service {name} uses a host bind mount; copy artifacts explicitly instead"
                )));
            }
            for field in [
                "cap_add",
                "devices",
                "device_cgroup_rules",
                "external_links",
                "ports",
                "volumes_from",
            ] {
                if has_items(service.get(field)) {
                    return Err(unsafe_compose(format!(
                        "service {name} declares host-facing `{field}` access"
                    )));
                }
            }
            for field in ["container_name", "credential_spec", "provider", "runtime"] {
                if has_items(service.get(field)) {
                    return Err(unsafe_compose(format!(
                        "service {name} declares host-scoped `{field}` access"
                    )));
                }
            }
            if service.get("use_api_socket").and_then(Value::as_bool) == Some(true) {
                return Err(unsafe_compose(format!(
                    "service {name} requests the Docker API socket"
                )));
            }
            if service
                .get("security_opt")
                .and_then(Value::as_array)
                .is_some_and(|options| {
                    options.iter().filter_map(Value::as_str).any(|option| {
                        let option = option.to_ascii_lowercase();
                        option.contains("unconfined")
                            || option == "label:disable"
                            || option == "no-new-privileges:false"
                    })
                })
            {
                return Err(unsafe_compose(format!(
                    "service {name} disables a container security boundary"
                )));
            }
        }
    }
    for group in ["networks", "volumes"] {
        if let Some(items) = config.get(group).and_then(Value::as_object) {
            for (name, item) in items {
                if item.get("external").and_then(Value::as_bool) == Some(true) {
                    return Err(unsafe_compose(format!(
                        "external {group} entry is not isolated: {name}"
                    )));
                }
                if item
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|resolved| !resolved.starts_with(&format!("{project}_")))
                {
                    return Err(unsafe_compose(format!(
                        "{group} entry `{name}` is not scoped to project `{project}`"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn run_step(manifest: &Manifest, base: &Path, step: &Step) -> Result<String, String> {
    let mut args = compose_prefix(manifest, base);
    let timeout = match step {
        Step::ComposeUp {
            services,
            timeout_seconds,
            ..
        } => {
            args.extend([
                "up".into(),
                "--detach".into(),
                "--no-build".into(),
                "--pull".into(),
                "never".into(),
            ]);
            args.extend(services.iter().cloned());
            *timeout_seconds
        }
        Step::Copy {
            artifact,
            service,
            destination,
            timeout_seconds,
            ..
        } => {
            let source = manifest
                .artifacts
                .iter()
                .find(|item| item.id == *artifact)
                .ok_or_else(|| format!("unknown artifact {artifact}"))?;
            args.push("cp".into());
            args.push(base.join(&source.path).display().to_string());
            args.push(format!("{service}:{destination}"));
            *timeout_seconds
        }
        Step::Exec {
            service,
            argv,
            timeout_seconds,
            ..
        } => {
            args.extend(["exec".into(), "--no-TTY".into(), service.clone()]);
            args.extend(argv.iter().cloned());
            *timeout_seconds
        }
    };
    let output = execute("docker", &args, timeout)?;
    if !output.status.success() {
        return Err(format!(
            "container operation exited with {}",
            exit_label(&output)
        ));
    }
    Ok(match step {
        Step::ComposeUp { services, .. } => {
            format!("started {} declared service(s)", services.len())
        }
        Step::Copy { artifact, .. } => {
            format!("copied artifact `{artifact}` into the isolated target")
        }
        Step::Exec { .. } => "declared command exited 0; output omitted".into(),
    })
}

fn run_check(manifest: &Manifest, base: &Path, check: &Check) -> Result<String, String> {
    match check {
        Check::Health {
            service,
            timeout_seconds,
            ..
        } => {
            let mut ps = compose_prefix(manifest, base);
            ps.extend(["ps".into(), "--quiet".into(), service.clone()]);
            let container = execute("docker", &ps, *timeout_seconds)?;
            if !container.status.success() {
                return Err("could not resolve service container".into());
            }
            let id = String::from_utf8_lossy(&container.stdout)
                .trim()
                .to_string();
            if id.is_empty() {
                return Err("service has no running container".into());
            }
            let inspect = execute(
                "docker",
                &[
                    "inspect".into(),
                    "--format".into(),
                    "{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}"
                        .into(),
                    id,
                ],
                *timeout_seconds,
            )?;
            let state = String::from_utf8_lossy(&inspect.stdout).trim().to_string();
            if !inspect.status.success() || state != "healthy" {
                return Err(format!(
                    "service health is `{state}`; declare a Compose healthcheck"
                ));
            }
            Ok("service reported healthy".into())
        }
        Check::RowCount {
            service,
            argv,
            min,
            max,
            timeout_seconds,
            ..
        } => {
            let output = exec_check(manifest, base, service, argv, *timeout_seconds)?;
            let source = String::from_utf8(output.stdout)
                .map_err(|_| "row-count output was not UTF-8".to_string())?;
            let count: u64 = source
                .trim()
                .parse()
                .map_err(|_| "row-count command must print one integer only".to_string())?;
            if count < *min || max.is_some_and(|upper| count > upper) {
                return Err(format!(
                    "row count {count} is outside expected range {}..{}",
                    min,
                    max.map_or("∞".into(), |v| v.to_string())
                ));
            }
            Ok(format!(
                "count {count} is within expected range {}..{}",
                min,
                max.map_or("∞".into(), |v| v.to_string())
            ))
        }
        Check::ExitCode {
            service,
            argv,
            timeout_seconds,
            ..
        } => {
            exec_check(manifest, base, service, argv, *timeout_seconds)?;
            Ok("declared check exited 0; output omitted".into())
        }
    }
}

fn exec_check(
    manifest: &Manifest,
    base: &Path,
    service: &str,
    argv: &[String],
    timeout: u64,
) -> Result<Output, String> {
    let mut args = compose_prefix(manifest, base);
    args.extend(["exec".into(), "--no-TTY".into(), service.into()]);
    args.extend(argv.iter().cloned());
    let output = execute("docker", &args, timeout)?;
    if !output.status.success() {
        return Err(format!(
            "check exited with {}; output omitted",
            exit_label(&output)
        ));
    }
    Ok(output)
}

fn cleanup(manifest: &Manifest, base: &Path) -> Result<(), String> {
    let mut args = compose_prefix(manifest, base);
    args.extend([
        "down".into(),
        "--volumes".into(),
        "--remove-orphans".into(),
        "--timeout".into(),
        "15".into(),
    ]);
    let output = execute("docker", &args, 60)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("compose down exited with {}", exit_label(&output)))
    }
}

fn compose_prefix(manifest: &Manifest, base: &Path) -> Vec<String> {
    vec![
        "compose".into(),
        "--file".into(),
        base.join(&manifest.compose.file).display().to_string(),
        "--project-name".into(),
        manifest.compose.project.clone(),
        "--project-directory".into(),
        base.display().to_string(),
    ]
}

fn execute(program: &str, args: &[String], timeout_seconds: u64) -> Result<Output, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start {program}: {e}"))?;
    // Drain both streams while the child is running. Waiting first can deadlock
    // a verbose restore tool once either OS pipe buffer fills.
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("could not capture {program} stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("could not capture {program} stderr"))?;
    let stdout_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut stream = stdout;
        stream.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut stream = stderr;
        stream.read_to_end(&mut bytes).map(|_| bytes)
    });
    let status = child
        .wait_timeout(Duration::from_secs(timeout_seconds))
        .map_err(|e| format!("could not wait for {program}: {e}"))?;
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
        let _ = stdout_reader.join();
        let _ = stderr_reader.join();
        return Err(format!("operation timed out after {timeout_seconds}s"));
    }
    let status = child
        .wait()
        .map_err(|e| format!("could not collect {program} status: {e}"))?;
    let stdout = stdout_reader
        .join()
        .map_err(|_| format!("could not collect {program} stdout"))?
        .map_err(|e| format!("could not collect {program} stdout: {e}"))?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| format!("could not collect {program} stderr"))?
        .map_err(|e| format!("could not collect {program} stderr: {e}"))?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

fn exit_label(output: &Output) -> String {
    output
        .status
        .code()
        .map_or("a signal".into(), |code| format!("code {code}"))
}

fn write_card(path: &Path, source: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("could not create {}: {e}", parent.display()))?;
    }
    let temporary = path.with_extension("md.tmp");
    let mut file = File::create(&temporary)
        .map_err(|e| format!("could not create {}: {e}", temporary.display()))?;
    file.write_all(source.as_bytes())
        .map_err(|e| format!("could not write card: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("could not sync card: {e}"))?;
    fs::rename(&temporary, path)
        .map_err(|e| format!("could not publish card {}: {e}", path.display()))
}

fn timestamp(value: SystemTime) -> Result<String, String> {
    OffsetDateTime::from(value)
        .format(&Rfc3339)
        .map_err(|e| format!("could not format timestamp: {e}"))
}

fn step_kind(step: &Step) -> &'static str {
    match step {
        Step::ComposeUp { .. } => "compose-up",
        Step::Copy { .. } => "copy",
        Step::Exec { .. } => "exec",
    }
}

fn check_kind(check: &Check) -> &'static str {
    match check {
        Check::Health { .. } => "health",
        Check::RowCount { .. } => "row-count",
        Check::ExitCode { .. } => "exit-code",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compose_safety_rejects_host_and_external_access() {
        let unsafe_models = [
            json!({"services": {"db": {"network_mode": "host"}}}),
            json!({"services": {"db": {"pid": "host"}}}),
            json!({"services": {"db": {"ipc": "service:other"}}}),
            json!({"services": {"db": {"uts": "host"}}}),
            json!({"services": {"db": {"userns_mode": "host"}}}),
            json!({"services": {"db": {"cgroup": "host"}}}),
            json!({"services": {"db": {"privileged": true}}}),
            json!({"services": {"db": {"volumes": [{"type": "bind"}]}}}),
            json!({"services": {"db": {"cap_add": ["SYS_ADMIN"]}}}),
            json!({"services": {"db": {"devices": ["/dev/kvm"]}}}),
            json!({"services": {"db": {"device_cgroup_rules": ["c 1:3 mr"]}}}),
            json!({"services": {"db": {"external_links": ["production-db"]}}}),
            json!({"services": {"db": {"ports": [{"target": 5432, "published": "5432"}]}}}),
            json!({"services": {"db": {"volumes_from": ["production-db"]}}}),
            json!({"services": {"db": {"container_name": "production-db"}}}),
            json!({"services": {"db": {"credential_spec": {"file": "host.json"}}}}),
            json!({"services": {"db": {"runtime": "nvidia"}}}),
            json!({"services": {"db": {"use_api_socket": true}}}),
            json!({"services": {"db": {"security_opt": ["seccomp=unconfined"]}}}),
            json!({"services": {"db": {"image": "postgres"}}, "networks": {"default": {"external": true}}}),
            json!({"services": {"db": {"image": "postgres"}}, "volumes": {"data": {"external": true}}}),
            json!({"services": {"db": {"image": "postgres"}}, "networks": {"default": {"name": "shared"}}}),
        ];
        for model in unsafe_models {
            let error = reject_unsafe_compose(&model, "rrc-test").unwrap_err();
            assert!(error.starts_with(UNSAFE_COMPOSE_PREFIX), "{error}");
        }

        let safe = json!({
            "services": {"db": {"image": "postgres", "ipc": "private"}},
            "networks": {"default": {"name": "rrc-test_default"}},
            "volumes": {"data": {"name": "rrc-test_data"}}
        });
        assert!(reject_unsafe_compose(&safe, "rrc-test").is_ok());
    }

    #[test]
    fn timed_process_collects_a_completed_status() {
        let output = execute("true", &[], 1).expect("true should finish");
        assert!(output.status.success());
    }
}
