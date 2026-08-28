use restore_rehearsal_card::report::{verify_card, write_new_key};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn fixture(failing: bool) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("temp directory");
    fs::create_dir(dir.path().join("bin")).unwrap();
    fs::write(dir.path().join("compose.yml"), "services: {}\n").unwrap();
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
case " $* " in
  *" config --format json "*) printf '%s\n' '{"services":{"database":{"image":"example"}},"networks":{"default":{}}}' ;;
  *" ps --quiet "*) printf '%s\n' 'container-id' ;;
  *" broken-check "*) exit 9 ;;
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
        .output()
        .expect("rrc executes")
}

#[test]
fn passing_run_emits_a_private_signed_card() {
    let (dir, _, card) = fixture(false);
    let result = run(dir.path(), &card);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let source = fs::read_to_string(card).unwrap();
    assert!(source.contains("**PASSED**"));
    assert!(source.contains("SHA-256"));
    assert!(!source.contains("private restored value"));
    verify_card(&source).unwrap();
}

#[test]
fn broken_check_exits_four_and_still_emits_valid_evidence() {
    let (dir, _, card) = fixture(true);
    let result = run(dir.path(), &card);
    assert_eq!(result.status.code(), Some(4));
    let source = fs::read_to_string(card).unwrap();
    assert!(source.contains("**FAILED**"));
    assert!(source.contains("check exited with code 9; output omitted"));
    verify_card(&source).unwrap();
}
