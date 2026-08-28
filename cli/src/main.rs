use clap::{Parser, Subcommand};
use restore_rehearsal_card::report::{verify_card, write_new_key};
use restore_rehearsal_card::{load_manifest, run_rehearsal, RunOptions};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "rrc",
    version,
    about = "Prove that a backup restores into a usable service",
    long_about = "Run typed restore steps in an isolated Docker Compose project, check the result, and emit a signed Markdown evidence card. No restored records or command output are written to the card."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Write a documented starter manifest and a new signing key
    Init {
        /// Manifest path to create
        #[arg(short, long, default_value = "restore-rehearsal.toml")]
        file: PathBuf,
        /// Private signing key path to create and reference
        #[arg(long, default_value = ".rrc/rehearsal.key")]
        signing_key: PathBuf,
    },
    /// Validate a manifest and all local inputs without starting Docker
    Check {
        #[arg(short, long, default_value = "restore-rehearsal.toml")]
        file: PathBuf,
        /// Print one JSON object for scripts
        #[arg(long)]
        json: bool,
    },
    /// Run a rehearsal, clean up its target, and write a signed card
    Run {
        #[arg(short, long, default_value = "restore-rehearsal.toml")]
        file: PathBuf,
        /// Must exactly equal compose.project; there is no interactive prompt
        #[arg(long)]
        confirm_target: String,
        /// Signed Markdown card destination
        #[arg(short, long, default_value = "restore-card.md")]
        output: PathBuf,
        /// Keep containers and volumes for debugging (the card records no restored data)
        #[arg(long)]
        keep_target: bool,
        /// Print one JSON result object for scripts
        #[arg(long)]
        json: bool,
    },
    /// Verify the embedded Ed25519 signature on a card
    Verify {
        card: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Generate a standalone signing key without changing a manifest
    Keygen {
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("rrc: {message}");
            ExitCode::from(code)
        }
    }
}

fn execute(cli: Cli) -> Result<(), (u8, String)> {
    match cli.command {
        Commands::Init { file, signing_key } => {
            if file.exists() {
                return Err((
                    2,
                    format!("refusing to overwrite manifest: {}", file.display()),
                ));
            }
            write_new_key(&signing_key).map_err(|error| (2, error))?;
            let key_reference = relative_key_reference(&file, &signing_key);
            let source = starter_manifest(&key_reference);
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| (2, format!("could not create {}: {e}", parent.display())))?;
            }
            fs::write(&file, source)
                .map_err(|e| (2, format!("could not write {}: {e}", file.display())))?;
            println!(
                "Created {} and {}. Edit the manifest, then run `rrc check`.",
                file.display(),
                signing_key.display()
            );
        }
        Commands::Check { file, json } => {
            let manifest = load_manifest(&file).map_err(|error| (2, error))?;
            if json {
                println!(
                    "{}",
                    json!({"valid": true, "name": manifest.name, "target": manifest.compose.project})
                );
            } else {
                println!(
                    "Valid: {} → {} ({} steps, {} checks)",
                    manifest.name,
                    manifest.compose.project,
                    manifest.steps.len(),
                    manifest.checks.len()
                );
            }
        }
        Commands::Run {
            file,
            confirm_target,
            output,
            keep_target,
            json,
        } => {
            let manifest = load_manifest(&file).map_err(|error| (2, error))?;
            if confirm_target != manifest.compose.project {
                return Err((
                    3,
                    format!(
                        "target confirmation mismatch; expected `{}`",
                        manifest.compose.project
                    ),
                ));
            }
            let options = RunOptions {
                manifest_path: file,
                confirm_target,
                output_path: output,
                keep_target,
            };
            let result = run_rehearsal(&manifest, &options).map_err(|error| {
                let safety = [
                    "host networking",
                    "privileged",
                    "bind mount",
                    "external networks",
                    "external volumes",
                    "confirmation",
                ]
                .iter()
                .any(|needle| error.contains(needle));
                (if safety { 3 } else { 4 }, error)
            })?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&result).expect("serializable result")
                );
            } else {
                println!(
                    "{}: signed card written to {} ({:.2}s)",
                    if result.passed { "Passed" } else { "Failed" },
                    result.card.display(),
                    result.duration_ms as f64 / 1_000.0
                );
            }
            if !result.passed {
                return Err((4, "rehearsal failed; inspect the signed card".into()));
            }
        }
        Commands::Verify { card, json } => {
            let source = fs::read_to_string(&card)
                .map_err(|e| (5, format!("could not read {}: {e}", card.display())))?;
            verify_card(&source).map_err(|error| (5, error))?;
            if json {
                println!("{}", json!({"valid": true, "card": card}));
            } else {
                println!("Valid signature: {}", card.display());
            }
        }
        Commands::Keygen { output } => {
            write_new_key(&output).map_err(|error| (2, error))?;
            println!(
                "Created {}. Keep it private and back it up separately.",
                output.display()
            );
        }
    }
    Ok(())
}

fn relative_key_reference(manifest: &Path, key: &Path) -> String {
    let base = manifest.parent().unwrap_or_else(|| Path::new("."));
    key.strip_prefix(base)
        .unwrap_or(key)
        .to_string_lossy()
        .replace('\\', "/")
}

fn starter_manifest(key: &str) -> String {
    format!(
        r#"version = 1
name = "Weekly database restore"
rto_seconds = 300
signing_key = "{key}"

[compose]
file = "rehearsal.compose.yml"
project = "restore-card-weekly"

[[artifacts]]
id = "database-backup"
path = "backups/latest.dump"

[[steps]]
id = "start-database"
kind = "compose-up"
services = ["database"]
timeout_seconds = 120

[[steps]]
id = "prepare-staging-path"
kind = "exec"
service = "database"
argv = ["mkdir", "-p", "/tmp/restore-rehearsal"]

[[steps]]
id = "copy-backup"
kind = "copy"
artifact = "database-backup"
service = "database"
destination = "/tmp/restore-rehearsal/latest.dump"

[[steps]]
id = "restore-backup"
kind = "exec"
service = "database"
argv = ["pg_restore", "--clean", "--if-exists", "--dbname", "app", "/tmp/restore-rehearsal/latest.dump"]

[[checks]]
id = "database-ready"
kind = "health"
service = "database"

[[checks]]
id = "accounts-present"
kind = "row-count"
service = "database"
argv = ["psql", "--username", "app", "--dbname", "app", "--tuples-only", "--no-align", "--command", "SELECT count(*) FROM accounts"]
min = 1
"#
    )
}
