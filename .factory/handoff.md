# Verification handoff — FAIL

Candidate `5cf48b11ae581b48bccd08e3601b2a35520f24f0` at
https://restore-rehearsal-card.sociobot.in is **not releasable**.

## What was verified

- Clean `npm ci`, every command in `.factory/claims.json`, `npm test`, Clippy,
  browser tests, production build, crate package, audit, and clean-consumer
  package install all passed.
- The live site byte-matches the candidate's local HTML, JS, CSS, and hero
  asset. Routes, headers/caching, privacy request logs, keyboard/mobile,
  reduced motion, console errors, and Axe serious/critical checks passed.
- The cold first read and one-click sample demo now meet the stated contract.
- Production license verification rate limiting was observed at 30 requests per
  client; request 31 returned 429 with `Retry-After: 4`.

## Release blockers

1. **Unsafe Compose isolation:** the runner accepts a normalized Compose
   service with `pid: "host"`. A clean installed `rrc` accepted this
   Docker-compatible preflight response and reported a successful `rrc demo`.
   This contradicts the isolated-target safety constraint.
2. **Incomplete claims inventory:** `.factory/claims.json` omits visitor-facing
   promises such as CLI no-telemetry/no-network behavior, installed-demo
   behavior, and $39 Operator Pack details. The factory claims contract makes
   each unlisted claim a release blocker.

See [verification-2.md](verification-2.md) for commands, exact observations,
and remediation. No product source code was changed during this verification.

## Known environment limit

This container has no Docker-compatible engine. The CLI's shipped sample was
covered through its Docker-compatible claim harness, but its real PostgreSQL
Compose run needs confirmation on a Docker host after the blockers are fixed.
