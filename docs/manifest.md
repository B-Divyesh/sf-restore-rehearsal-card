# Manifest reference and safety model

The v1 TOML manifest is intentionally declarative. Paths are resolved relative
to the manifest, so a rehearsal directory can move as one unit.

## Top level

- `version`: must be `1`.
- `name`: human-readable card title.
- `rto_seconds`: objective measured across steps, checks, and default cleanup.
- `signing_key`: local Ed25519 private key created by `rrc init` or `rrc keygen`.
- `compose.file`: the isolated Compose definition.
- `compose.project`: disposable target name. It must contain `rehearsal`,
  `restore`, `drill`, or `rrc`, and must not look production-like.
- `artifacts`: local inputs identified by stable IDs. Each is SHA-256 hashed.

## Typed steps

`compose-up` accepts `services`. `copy` accepts an artifact ID, service, and a
destination strictly below `/tmp/restore-rehearsal/`. `exec` accepts a service
and an argv array. Each accepts `timeout_seconds` (default 120, maximum 3600).

No step invokes a host shell. Container shell interpreters are rejected too,
which keeps interpolation, pipelines, redirects, and compound destructive
commands out of the manifest. Restore tools such as `pg_restore`, `psql`, and
`sqlite3` receive arguments directly.

## Typed checks

- `health`: requires the service's Compose health check to report `healthy`.
- `row-count`: runs a declared argv array and accepts exactly one unsigned
  integer. `min` is required and `max` is optional.
- `exit-code`: passes only on exit 0.

Command output is used only in memory while a check runs. The card includes
states, the final integer count, duration, artifact size, and checksum—not
stdout, stderr, container IDs, credentials, filenames, or restored records.

## Isolation and cleanup

Before starting, `rrc` asks Docker Compose for the fully resolved JSON model.
It refuses host or external namespaces, bind mounts, added capabilities,
devices, published ports, Docker API access, unconfined security settings,
privileged services, and external networks or volumes. Named networks and
volumes must remain scoped to the declared Compose project.

The exact project name must be passed through `--confirm-target`. CI never
receives an interactive prompt. On completion or a failed step,
`docker compose down --volumes --remove-orphans` runs by default.

Use dedicated non-production credentials in the Compose file. Do not reference
provider production secrets or Docker contexts pointed at production hosts.
`rrc` deliberately does not call provider APIs or attempt to infer whether a
credential belongs to production.

`--keep-target` exists for local debugging and is never required for evidence.
Its use leaves restored data in the declared disposable Compose target; clean it
up with the same project/file arguments before leaving the workstation.
