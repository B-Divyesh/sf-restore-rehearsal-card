# Repair handoff — Restore Rehearsal Card

## Release repair

This repair addresses every release blocker in independent verification of
candidate `7887afe1ec3ad5603809e7f6ced35a5d306aa06a`.

- Added the required claims inventory at `.factory/claims.json`; every entry
  has one `@claim:<id>` regression test and a fresh-sandbox command.
- Added `rrc demo`, which writes the bundled PostgreSQL sample into a new OS
  temporary directory, runs the same rehearsal runner, cleans up by default,
  and prints its workspace and signed card path. The shipped sample is also
  documented in `.factory/demo.md`.
- Added `/demo/` with the required sample-data action, persistent demo notice,
  reset, and start-for-real controls. It stores no browser data.
- Fixed runner pipe handling: stdout and stderr now drain concurrently while a
  Docker command runs. A 1 MiB output regression completes instead of falsely
  timing out.
- Made `rrc check` validate Compose YAML and the manifest-relative Ed25519 key.
  Nested `rrc init --file nested/restore.toml` now creates
  `nested/.rrc/rehearsal.key`, and invalid `check --json` errors are JSON.
- Switched purchase and license verification defaults from pilot to
  `https://api.sociobot.in`.
- Added a designed 404, canonical/Open Graph/Twitter metadata, 1200×630 social
  image, Apple touch icon, sitemap demo route, CSP, frame/permissions policy,
  and immutable hashed-asset policy in `staticwebapp.config.json`.
- Restored consistent demo/privacy navigation and footer build identity on all
  routes. The landing copy now names database and self-hosted service operators
  and gives the sample outcome next to the primary action.

## Verification evidence

Run from a clean dependency install:

```sh
npm ci
npm test
npm run test:browser
cargo clippy --workspace --all-targets -- -D warnings
npm run build
npm run package
npm audit --audit-level=high
```

All commands passed on 2026-08-28.

- `npm test`: 7 Rust unit tests, 5 CLI integration tests, strict TypeScript,
  and 5 Vitest tests passed.
- `npm run test:browser`: 19 passed across desktop and 390 px mobile; one
  intentional desktop-only skip. Playwright Axe found no serious or critical
  violations on `/`, `/demo/`, `/privacy/`, `/terms/`, or `/404.html`.
- All eight commands in `.factory/claims.json` were run individually; each
  passed. The 1 MiB output, nested init/input validation, bundled CLI sample,
  signed/private card, failed-card, production API, demo banner, and
  same-origin privacy regressions are covered.
- `npm run build`: generated `dist/site/` and `dist/bin/rrc`; initial JS is
  2.96 KB gzip and CSS is 4.24 KB gzip.
- `npm run package`: packaged and verified the crate. A separate temporary
  `cargo install --path target/package/restore-rehearsal-card-0.1.0` completed;
  its installed `rrc --help` exposes `demo` with the documented command.
- `/opt/fleet/lib/verify-url.sh http://127.0.0.1:4173` passed: HTTP 200,
  title, `lang=en`, one h1, main landmark, image alt text, labelled buttons,
  and zero console errors. Evidence: `/tmp/rrc-verify-local/verify.json`.
- Lighthouse mobile against the production build: Performance 100,
  Accessibility 100, Best Practices 100, SEO 100; LCP 1,023 ms and CLS 0.
- Browser privacy test captures the whole demo-reset flow and permits only the
  same origin. No service worker is shipped, so offline reload/update behavior
  is intentionally not claimed.

## Deployment

Deploy class remains static. Push this commit to `main`; the factory static
deployment consumes `dist/site/` and the checked-in `staticwebapp.config.json`.
After deployment, verify `/`, `/demo/`, `/privacy/`, `/terms/`, `/404.html`,
the production checkout URL, CSP/Permissions-Policy/X-Frame-Options headers,
and immutable caching for `/assets/*` against the live origin.

## Known environment limit

This container has no Docker/Podman/nerdctl binary or Docker socket. The CLI
was exercised end-to-end through an independent Docker-compatible harness,
including its bundled sample. Run `rrc demo` once on a clean Docker host to
exercise the real PostgreSQL image pull and Compose runtime.
