# Restore Rehearsal Card

Restore Rehearsal Card (`rrc`) is a local, zero-telemetry CLI for operators who
need proof that a backup restores into a usable service—not merely proof that a
backup file exists. It starts a declared Docker Compose target, copies artifacts
into that isolated target, runs restore commands and explicit assertions, records
checksums and timings, then emits an Ed25519-signed Markdown card.

It does not store backups, host disaster recovery, print restored data, or
bypass provider APIs.

## Install

Download a release binary for your platform, or build from source:

```sh
cargo install --path cli
rrc --help
```

Requirements: Docker with the `docker compose` plugin. Rust 1.88+ is needed only
when building from source.

## Usage

Create a starter manifest and a local signing key:

```sh
rrc init --file restore-rehearsal.toml --signing-key .rrc/rehearsal.key
```

Edit the manifest so every artifact, service, argv array, and assertion matches
your disposable Compose stack. Validate it without touching Docker:

```sh
rrc check --file restore-rehearsal.toml
```

Run the rehearsal. The confirmation must exactly match `compose.project`:

```sh
rrc run --file restore-rehearsal.toml \
  --confirm-target restore-card-weekly \
  --output reports/weekly.md
```

Machine-readable status is written to stdout while the signed Markdown card is
still created:

```sh
rrc run -f restore-rehearsal.toml \
  --confirm-target restore-card-weekly \
  --output reports/weekly.md --json
```

Verify the card later, on another machine, using the public key embedded in it:

```sh
rrc verify reports/weekly.md
```

`rrc` exits `0` for a passed rehearsal, `2` for invalid input, `3` for a safety
refusal, `4` for a failed step/check, and `5` for an invalid signature.

## Manifest surface

The complete typed surface is intentionally small:

- `compose-up` starts named services in the declared Compose project.
- `copy` copies one declared host artifact into `/tmp/restore-rehearsal/...` in
  a declared service.
- `exec` runs an argv array inside a declared service. Shell interpreters and
  host commands are rejected.
- `health` reads the container health state without exposing service data.
- `row-count` runs an argv array and records only the integer count, never rows.
- `exit-code` records only pass/fail for an argv array; stdout/stderr stay out of
  the card.

See [`examples/postgres/restore-rehearsal.toml`](examples/postgres/restore-rehearsal.toml)
for a complete PostgreSQL drill and [`docs/manifest.md`](docs/manifest.md) for
the schema and threat model.

## Develop and verify

```sh
npm ci
npm test
npm run build
npm run build:site       # site only -> dist/site
npm run package          # cargo package validation
```

`npm run build` produces the deployable documentation site at `dist/site/` and
release binaries at `dist/bin/`. The project does not publish from local builds;
the factory owns registry and release credentials.

## Privacy and purchase

The CLI is local-first and has no telemetry or network client. The optional
Operator Pack is a one-time purchase verified by the Sociobot license endpoint
on the documentation site. See the site's `/privacy/` and `/terms/` pages.

## License

MIT © 2026 Sociobot (Param Factory). See [LICENSE](LICENSE).
