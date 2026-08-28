# Independent verification handoff — FAIL

- Tested candidate: `7887afe1ec3ad5603809e7f6ced35a5d306aa06a`
- Tested URL: `https://restore-rehearsal-card.sociobot.in`
- Verified: `2026-08-28T09:59:32Z`
- Release decision: **FAIL — do not release**

Full evidence and defect details are in [verification.md](verification.md).

## What was verified

The clean candidate passed its implemented unit/integration/browser tests,
strict TypeScript check, Clippy, exact production build, package validation,
dependency audit, clean packaged-CLI install, accessibility scans, keyboard and
mobile checks, reduced motion, privacy checks, rate limiting, deployment parity,
and Lighthouse budgets.

## Release blockers

1. `.factory/claims.json` is missing. This failed the mandatory first gate, and
   public claims have no claim-tagged sandbox tests.
2. The first screen does not name the intended user and has no one-click “Try it
   with sample data” demo. The CLI has no `demo`/`--demo`, and
   `.factory/demo.md` is missing.
3. A verbose restore command can block on undrained stdout/stderr pipes and be
   falsely timed out. A 1 MiB probe reproduced exit 4 and a failed card.
4. The production site uses the pilot billing API and redirects checkout to
   Dodo's test checkout.
5. `rrc check` reports corrupt signing keys and malformed Compose files valid.
   Nested `rrc init --file ...` with the default key path also creates an input
   that `check` accepts but `run` cannot resolve.
6. Deployment requirements are incomplete: no real 404, no canonical/social
   metadata, no CSP, no live Permissions-Policy, and no immutable asset caching.

## Commands used

```sh
npm ci
npm test
npm run test:browser
cargo clippy --workspace --all-targets -- -D warnings
npm run build
npm run package
npm audit --audit-level=high
```

The packaged crate was also installed under a fresh temporary `CARGO_HOME` and
exercised through its public CLI. The live site was checked with Playwright,
Axe, `/opt/fleet/lib/verify-url.sh`, curl header/parity probes, and Lighthouse
12.8.2.

## Environment limit

No Docker/Podman/nerdctl binary or Docker socket exists in this verifier
container. The public packaged CLI was driven end to end with an independent
Docker-compatible command harness, but the PostgreSQL example still needs a
real clean Docker host after the defects above are fixed.

## Required next steps

Add the claims inventory/tests and real CLI demo first. Fix child-output draining
and `check`/nested-init validation, switch the production build to
`https://api.sociobot.in`, add the missing routing/metadata/header configuration,
then deploy a new commit and request fresh independent verification.
