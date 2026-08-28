use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use std::fs;
use std::path::Path;

const SIGNATURE_MARKER: &str = "\n<!-- rrc-signature\n";

#[derive(Debug, Clone, Serialize)]
pub struct Evidence {
    pub id: String,
    pub kind: String,
    pub passed: bool,
    pub duration_ms: u128,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactEvidence {
    pub id: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Card {
    pub rehearsal: String,
    pub target: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u128,
    pub rto_seconds: u64,
    pub passed: bool,
    pub artifacts: Vec<ArtifactEvidence>,
    pub evidence: Vec<Evidence>,
    pub failure: Option<String>,
}

impl Card {
    pub fn markdown_body(&self) -> String {
        let outcome = if self.passed { "PASSED" } else { "FAILED" };
        let rto = if self.duration_ms <= self.rto_seconds as u128 * 1_000 {
            "met"
        } else {
            "missed"
        };
        let mut body = format!(
            "# Restore rehearsal card\n\n\
             **{}** · `{}`\n\n\
             | Result | Observed | RTO objective | RTO |\n\
             | --- | ---: | ---: | --- |\n\
             | **{}** | {:.2} s | {} s | {} |\n\n\
             - Started: `{}`\n\
             - Finished: `{}`\n\
             - Report format: `rrc/v1`\n\n",
            escape_markdown(&self.rehearsal),
            self.target,
            outcome,
            self.duration_ms as f64 / 1_000.0,
            self.rto_seconds,
            rto,
            self.started_at,
            self.finished_at,
        );
        body.push_str("## Artifact evidence\n\n");
        if self.artifacts.is_empty() {
            body.push_str("No host artifacts were declared.\n\n");
        } else {
            body.push_str("| Artifact | Bytes | SHA-256 |\n| --- | ---: | --- |\n");
            for artifact in &self.artifacts {
                body.push_str(&format!(
                    "| `{}` | {} | `{}` |\n",
                    artifact.id, artifact.bytes, artifact.sha256
                ));
            }
            body.push('\n');
        }
        body.push_str("## Steps and checks\n\n");
        body.push_str(
            "| State | ID | Kind | Time | Evidence |\n| --- | --- | --- | ---: | --- |\n",
        );
        for item in &self.evidence {
            body.push_str(&format!(
                "| {} | `{}` | {} | {} ms | {} |\n",
                if item.passed { "PASS" } else { "FAIL" },
                item.id,
                item.kind,
                item.duration_ms,
                escape_table(&item.detail),
            ));
        }
        if let Some(failure) = &self.failure {
            body.push_str(&format!("\n## Failure\n\n{}\n", escape_markdown(failure)));
        }
        body.push_str(
            "\n> Privacy note: this card contains checksums, counts, timings, and states only. Command output and restored records are intentionally omitted.\n",
        );
        body
    }

    pub fn sign(&self, signing_key: &SigningKey) -> String {
        let body = self.markdown_body();
        let signature = signing_key.sign(body.as_bytes());
        format!(
            "{}{}algorithm: ed25519\npublic-key: {}\nsignature: {}\n-->\n",
            body,
            SIGNATURE_MARKER,
            STANDARD.encode(signing_key.verifying_key().as_bytes()),
            STANDARD.encode(signature.to_bytes()),
        )
    }
}

pub fn write_new_key(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "refusing to overwrite signing key: {}",
            path.display()
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("could not create {}: {e}", parent.display()))?;
    }
    let mut secret = [0_u8; 32];
    getrandom::getrandom(&mut secret)
        .map_err(|e| format!("could not generate signing key: {e}"))?;
    let encoded = format!("rrc-ed25519-v1:{}\n", STANDARD.encode(secret));
    fs::write(path, encoded).map_err(|e| format!("could not write {}: {e}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("could not secure {}: {e}", path.display()))?;
    }
    Ok(())
}

pub fn read_signing_key(path: &Path) -> Result<SigningKey, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("could not read signing key {}: {e}", path.display()))?;
    let encoded = source
        .trim()
        .strip_prefix("rrc-ed25519-v1:")
        .ok_or_else(|| "unsupported signing key format".to_string())?;
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "invalid signing key encoding".to_string())?;
    let secret: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "signing key must contain 32 bytes".to_string())?;
    Ok(SigningKey::from_bytes(&secret))
}

pub fn verify_card(source: &str) -> Result<(), String> {
    let (body, block) = source
        .rsplit_once(SIGNATURE_MARKER)
        .ok_or_else(|| "card has no rrc signature block".to_string())?;
    let block = block
        .strip_suffix("-->\n")
        .or_else(|| block.strip_suffix("-->"))
        .ok_or_else(|| "signature block is not closed".to_string())?;
    let value = |name: &str| -> Result<&str, String> {
        block
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{name}: ")))
            .ok_or_else(|| format!("signature block is missing {name}"))
    };
    if value("algorithm")? != "ed25519" {
        return Err("unsupported signature algorithm".into());
    }
    let public_bytes: [u8; 32] = STANDARD
        .decode(value("public-key")?)
        .map_err(|_| "invalid public key encoding".to_string())?
        .try_into()
        .map_err(|_| "public key must contain 32 bytes".to_string())?;
    let signature_bytes: [u8; 64] = STANDARD
        .decode(value("signature")?)
        .map_err(|_| "invalid signature encoding".to_string())?
        .try_into()
        .map_err(|_| "signature must contain 64 bytes".to_string())?;
    let key =
        VerifyingKey::from_bytes(&public_bytes).map_err(|_| "invalid public key".to_string())?;
    key.verify(body.as_bytes(), &Signature::from_bytes(&signature_bytes))
        .map_err(|_| "card signature is invalid".to_string())
}

fn escape_markdown(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "\\`")
}

fn escape_table(value: &str) -> String {
    escape_markdown(value)
        .replace('|', "\\|")
        .replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card() -> Card {
        Card {
            rehearsal: "Weekly restore".into(),
            target: "restore-weekly".into(),
            started_at: "2026-08-28T00:00:00Z".into(),
            finished_at: "2026-08-28T00:00:01Z".into(),
            duration_ms: 1_000,
            rto_seconds: 10,
            passed: true,
            artifacts: vec![],
            evidence: vec![],
            failure: None,
        }
    }

    #[test]
    fn signed_card_verifies_and_tampering_fails() {
        let key = SigningKey::from_bytes(&[7_u8; 32]);
        let signed = card().sign(&key);
        assert!(verify_card(&signed).is_ok());
        assert!(verify_card(&signed.replace("PASSED", "FAILED")).is_err());
    }

    #[test]
    fn card_never_includes_command_output_field() {
        let body = card().markdown_body();
        assert!(body.contains("restored records are intentionally omitted"));
        assert!(!body.contains("stdout"));
    }
}
