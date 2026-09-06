use restore_rehearsal_card::report::{verify_card, write_new_key};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn fixture(failing: bool) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("temp directory");
    fs::create_dir(dir.path().join("bin")).unwrap();
    fs::write(
        dir.path().join("compose.yml"),
        "services:\n  database:\n    image: postgres:17-alpine\n",
    )
    .unwrap();
    fs::write(dir.path().join("backup.sql"), "private restored value\n").unwrap();
    write_new_key(&dir.path().join("rehearsal.key")).unwrap();
    let check_command = if failing {
        "broken-check"
    } else {
        "probe-check"
    };
    fs::write(
        dir.path().join("restore.toml"),
        format!(
            r#"version = 1
name = "Integration rehearsal"
rto_seconds = 30
signing_key = "rehearsal.key"

[compose]
file = "compose.yml"
project = "rrc-integration"

[[artifacts]]
id = "backup"
path = "backup.sql"

[[steps]]
id = "start"
kind = "compose-up"
services = ["database"]

[[steps]]
id = "copy"
kind = "copy"
artifact = "backup"
service = "database"
destination = "/tmp/restore-rehearsal/backup.sql"

[[checks]]
id = "ready"
kind = "health"
service = "database"

[[checks]]
id = "row-count"
kind = "row-count"
service = "database"
argv = ["count-command", "--command", "SELECT count(*) FROM probe"]
min = 1
max = 1

[[checks]]
id = "probe"
kind = "exit-code"
service = "database"
argv = ["{check_command}"]
"#,
        ),
    )
    .unwrap();
    let docker = dir.path().join("bin/docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
if [ -n "${RRC_TEST_DOCKER_LOG:-}" ]; then printf '%s\n' "$*" >> "$RRC_TEST_DOCKER_LOG"; fi
case " $* " in
  *" config --format json "*) printf '%s\n' '{"services":{"database":{"image":"example"}},"networks":{"default":{}}}' ;;
  *" ps --quiet "*) printf '%s\n' 'container-id' ;;
  *" broken-check "*) exit 9 ;;
  *" verbose-check "*) head -c 1048576 /dev/zero | tr '\\000' x ;;
  *" --command "*) printf '%s\n' '1' ;;
  *) if [ "$1" = "inspect" ]; then printf '%s\n' 'healthy'; fi ;;
esac
"#,
    )
    .unwrap();
    #[cfg(unix)]
    fs::set_permissions(&docker, fs::Permissions::from_mode(0o755)).unwrap();
    let card = dir.path().join("card.md");
    (dir, docker, card)
}

fn run(dir: &Path, card: &Path) -> std::process::Output {
    let current_path = std::env::var("PATH").unwrap_or_default();
    Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args([
            "run",
            "--file",
            dir.join("restore.toml").to_str().unwrap(),
            "--confirm-target",
            "rrc-integration",
            "--output",
            card.to_str().unwrap(),
            "--json",
        ])
        .env(
            "PATH",
            format!("{}:{current_path}", dir.join("bin").display()),
        )
        .env("RRC_TEST_DOCKER_LOG", dir.join("docker.log"))
        .output()
        .expect("rrc executes")
}

/// @claim:signed-private-card
#[test]
fn passing_run_emits_a_private_signed_card() {
    let (dir, _, card) = fixture(false);
    let result = run(dir.path(), &card);
    let source = fs::read_to_string(&card).unwrap_or_default();
    assert!(
        result.status.success(),
        "stdout: {}\nstderr: {}\ncard: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
        source,
    );
    assert!(source.contains("**PASSED**"));
    assert!(source.contains("SHA-256"));
    assert!(source.contains("count 1 is within expected range 1..1"));
    assert!(source.contains("RTO objective"));
    assert!(source.contains("ed25519"));
    assert!(!source.contains("private restored value"));
    verify_card(&source).unwrap();
}

/// @claim:failed-drill-card
#[test]
fn broken_check_exits_four_and_still_emits_valid_evidence() {
    let (dir, _, card) = fixture(true);
    let result = run(dir.path(), &card);
    assert_eq!(result.status.code(), Some(4));
    let source = fs::read_to_string(card).unwrap();
    assert!(source.contains("**FAILED**"));
    assert!(source.contains("check exited with code 9; output omitted"));
    let docker_log = fs::read_to_string(dir.path().join("docker.log")).unwrap();
    assert!(docker_log.contains("down --volumes --remove-orphans"));
    verify_card(&source).unwrap();
}

/// @claim:large-command-output
#[test]
fn verbose_command_output_does_not_deadlock_the_rehearsal() {
    let (dir, _, card) = fixture(false);
    let manifest = dir.path().join("restore.toml");
    let source = fs::read_to_string(&manifest)
        .unwrap()
        .replace("probe-check", "verbose-check");
    fs::write(&manifest, source).unwrap();
    let result = run(dir.path(), &card);
    assert!(
        result.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(fs::read_to_string(card).unwrap().contains("**PASSED**"));
}

/// @claim:check-local-inputs
#[test]
fn check_rejects_corrupt_inputs_and_nested_init_creates_a_runnable_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("nested/restore.toml");
    let init = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args(["init", "--file", nested.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(dir.path().join("nested/.rrc/rehearsal.key").is_file());
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(dir.path().join("nested/.rrc/rehearsal.key"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let source = fs::read_to_string(&nested).unwrap();
    assert!(source.contains("signing_key = \".rrc/rehearsal.key\""));

    fs::write(
        dir.path().join("nested/rehearsal.compose.yml"),
        "services: [bad\n",
    )
    .unwrap();
    let invalid_compose = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args(["check", "--file", nested.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(invalid_compose.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid_compose.stdout).contains("\"valid\":false"));

    fs::write(
        dir.path().join("nested/rehearsal.compose.yml"),
        "services:\n  database:\n    image: postgres:17-alpine\n",
    )
    .unwrap();
    fs::write(dir.path().join("nested/.rrc/rehearsal.key"), "corrupt\n").unwrap();
    let invalid_key = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args(["check", "--file", nested.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(invalid_key.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid_key.stdout).contains("\"valid\":false"));
}

/// @claim:safe-command-surface
#[test]
fn unsafe_commands_destinations_and_targets_are_rejected_before_docker() {
    let (dir, _, card) = fixture(false);
    let manifest = dir.path().join("restore.toml");
    let original = fs::read_to_string(&manifest).unwrap();

    let shell = original.replace(
        "argv = [\"probe-check\"]",
        "argv = [\"sh\", \"-c\", \"echo unsafe\"]",
    );
    fs::write(&manifest, shell).unwrap();
    let check = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args(["check", "--file", manifest.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(check.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&check.stdout).contains("shell interpreters"));

    let traversal = original.replace(
        "/tmp/restore-rehearsal/backup.sql",
        "/tmp/restore-rehearsal/../host.sql",
    );
    fs::write(&manifest, traversal).unwrap();
    let check = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args(["check", "--file", manifest.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(check.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&check.stdout).contains("copy destination"));

    fs::write(&manifest, original).unwrap();
    let current_path = std::env::var("PATH").unwrap_or_default();
    let refusal = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args([
            "run",
            "--file",
            manifest.to_str().unwrap(),
            "--confirm-target",
            "production",
            "--output",
            card.to_str().unwrap(),
            "--json",
        ])
        .env(
            "PATH",
            format!("{}:{current_path}", dir.path().join("bin").display()),
        )
        .output()
        .unwrap();
    assert_eq!(refusal.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&refusal.stdout).contains("target confirmation mismatch"));
    assert!(!card.exists());
    assert!(!dir.path().join("docker.log").exists());
}

/// @claim:stable-exit-codes
#[test]
fn documented_exit_codes_and_json_results_are_observable() {
    let (dir, _, card) = fixture(false);
    let manifest = dir.path().join("restore.toml");
    let valid = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args(["check", "--file", manifest.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(valid.status.code(), Some(0));
    let valid_json: serde_json::Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(valid_json["valid"], true);

    let invalid = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args(["check", "--file", "missing.toml", "--json"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(2));
    let invalid_json: serde_json::Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(invalid_json["valid"], false);

    let refusal = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args([
            "run",
            "--file",
            manifest.to_str().unwrap(),
            "--confirm-target",
            "wrong-target",
            "--output",
            card.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(refusal.status.code(), Some(3));
    let refusal_json: serde_json::Value = serde_json::from_slice(&refusal.stdout).unwrap();
    assert!(refusal_json["error"]
        .as_str()
        .unwrap()
        .contains("target confirmation mismatch"));

    let (failed_dir, _, failed_card) = fixture(true);
    let failed = run(failed_dir.path(), &failed_card);
    assert_eq!(failed.status.code(), Some(4));
    let failed_json: serde_json::Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert_eq!(failed_json["passed"], false);

    let tampered = fs::read_to_string(&failed_card)
        .unwrap()
        .replace("**FAILED**", "**PASSED**");
    fs::write(&failed_card, tampered).unwrap();
    let invalid_signature = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .args(["verify", failed_card.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(invalid_signature.status.code(), Some(5));
    let signature_json: serde_json::Value =
        serde_json::from_slice(&invalid_signature.stdout).unwrap();
    assert!(signature_json["error"]
        .as_str()
        .unwrap()
        .contains("signature"));
}

#[test]
fn bundled_demo_runs_the_shipped_sample_in_a_temp_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let docker = dir.path().join("docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
case " $* " in
  *" config --format json "*) printf '%s\n' '{"services":{"database":{"image":"example"}},"networks":{"default":{}}}' ;;
  *" ps --quiet "*) printf '%s\n' 'container-id' ;;
  *" --command "*) printf '%s\n' '1' ;;
  *) if [ "$1" = "inspect" ]; then printf '%s\n' 'healthy'; fi ;;
esac
"#,
    )
    .unwrap();
    #[cfg(unix)]
    fs::set_permissions(&docker, fs::Permissions::from_mode(0o755)).unwrap();
    let current_path = std::env::var("PATH").unwrap_or_default();
    let result = Command::new(env!("CARGO_BIN_EXE_rrc"))
        .arg("demo")
        .env("PATH", format!("{}:{current_path}", dir.path().display()))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = String::from_utf8_lossy(&result.stdout);
    assert!(output.contains("Sample rehearsal passed. Workspace:"));
    assert!(output.contains("Card:"));
}
