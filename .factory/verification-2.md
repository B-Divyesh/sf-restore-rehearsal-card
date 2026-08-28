# Independent product verification — FAIL

Tested 2026-08-28 against candidate `5cf48b11ae581b48bccd08e3601b2a35520f24f0` on `main`.

- Live URL: `https://restore-rehearsal-card.sociobot.in`
- Artifact: Rust CLI plus static documentation/product site
- Verdict: **FAIL — do not release**

The live deployment is the requested candidate: SHA-256 matched local production output for `index.html`, the hashed JS and CSS assets, and `recovery-observatory.webp`.

## Mandatory first checks

### Claims gate — PASS after clean install

`.factory/claims.json` exists and every exact command was run. The browser command cannot resolve `@playwright/test` before dependencies are installed in a clean checkout; after the required `npm ci`, all nine claim entries passed.

| Claim ID | Result | Evidence |
| --- | --- | --- |
| `sample-cli-demo` | PASS | bundled sample passed in its fresh Docker-compatible harness |
| `large-command-output` | PASS | 1 MiB command-output probe completed without deadlock |
| `check-local-inputs` | PASS | malformed Compose/corrupt key rejected; nested init output valid |
| `signed-private-card` | PASS | signed PASS card omitted the private backup value |
| `failed-drill-card` | PASS | failed check exited 4 and produced a valid signed card |
| `sample-demo-route` | PASS | desktop and 390 px browser tests passed |
| `site-no-analytics` | PASS | desktop and 390 px demo request tests passed |
| `production-checkout` | PASS | desktop and 390 px browser tests passed |
| `production-license-verification` | PASS | Vitest production verification URL test passed |

### Cold first-read and one-click demo — PASS

From a fresh live browser context, the first screen says: “Test your backup before an incident.” It names “database and self-hosted service operators” and puts **Try it with sample data** first, with the immediate outcome: “Opens the shipped PostgreSQL sample and its exact CLI command.” `/demo/` provides the persistent “Demo — sample data, nothing is saved” banner plus Reset demo and Start for real. This passes the plain-words and sample-demo gates.

## Release-blocking findings

### High — Compose safety does not enforce an isolated target

`cli/src/runner.rs` rejects host networking, privileged containers, bind mounts, and external networks/volumes, but does **not** reject Docker Compose `pid: host` (or other host-namespace modes). That violates the brief’s isolated-target constraint and lets declared commands execute in the host PID namespace.

Independent reproduction used the crate installed from `cargo package` in a clean consumer directory. A Docker-compatible harness returned this normalized Compose config during preflight:

```json
{"services":{"database":{"image":"postgres","pid":"host"}},"networks":{"default":{}}}
```

`rrc demo` accepted it and printed `Sample rehearsal passed ...`; exit status was `0`. The project must reject `pid: host` at minimum, and should review other host namespace/device/capability escape routes before claiming isolation.

### High — required claims inventory is incomplete

The mandatory claim tests that exist pass, but the live page and README make additional user-reliant promises with no corresponding item in `.factory/claims.json`. The claims contract says an unlisted claim fails review until removed or tested.

Examples include:

- “Local CLI; no telemetry” and “The CLI itself never phones home” (`site/index.html`; README says “no telemetry or network client”). The existing `site-no-analytics` test only observes the documentation-page demo flow, not the CLI.
- “Sample runs after install.” The declared bundled-demo claim uses the Cargo test binary rather than an installed consumer package.
- The $39 one-time Operator Pack price and the stated included materials.
- Metadata descriptions promising an “isolated restore drill” and “signed card of what happened.”

Add one exact sandbox test per retained public claim (including an installed-binary network observation where needed), or remove/soften the untestable copy.

## Passing verification evidence

### Repository and package

- `npm ci`: passed; 97 packages installed; `npm audit --audit-level=high`: 0 vulnerabilities.
- `npm test`: passed: 7 Rust unit tests, 5 Rust CLI integrations, strict TypeScript, and 5 Vitest tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `npm run test:browser`: passed (19 passed, 1 intentional desktop skip); `.last-run.json` reports `passed` with no failed tests.
- `npm run build`: passed and produced `dist/site/` plus `dist/bin/rrc`; initial JS is 2.96 KB gzip and CSS 4.24 KB gzip.
- `npm run package`: passed. The packaged crate was installed to a fresh Cargo home/root. Its `--help` exposed all six documented commands; a copied bundled manifest passed `rrc check --json`; generated signing key mode was `0600`; a wrong confirmation exited `3` and wrote no card.
- No Docker, Podman, nerdctl binary, or Docker socket is available in this verifier container. An installed `rrc demo` consequently returns exit 4 with the clear “could not start docker” error. The supplied claim harness did exercise its normal and failed sample flows, but real Docker/Compose execution remains unobserved here.

### Live deployment, privacy, and accessibility

- Local and live SHA-256 values match for root HTML, JS, CSS, and the hero image. `/`, `/demo/`, `/privacy/`, `/terms/`, `/404.html`, robots, and sitemap return 200; an unknown route returns 404.
- Hashed JS returns `Cache-Control: public, max-age=31536000, immutable`. The live HTML sends CSP, HSTS, `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, Referrer-Policy, and Permissions-Policy.
- `/opt/fleet/lib/verify-url.sh` passed against the live URL: HTTP 200, title, `lang=en`, one h1, main landmark, image alt text, labelled buttons, zero console errors; load was 578 ms. Evidence is in `/tmp/rrc-verify-2/`.
- Independent live Playwright/Axe scans found zero serious or critical violations on `/`, `/demo/` at 390 px, `/privacy/`, `/terms/`, and `/404.html`; no route had horizontal overflow, page errors, console errors, or failed requests. First Tab reaches Skip to main content; the primary action has a visible `3px` ice-blue focus outline. Reduced-motion contexts were included in the route scans.
- Fresh demo and landing flows made same-origin requests only. No sign-in is present, so Entra tenant validation is not applicable. It is not a PWA, so service-worker/offline update testing is not applicable.
- A mocked invalid returned license was stored under `sb_license:restore-rehearsal-card`, removed from the live URL, showed the quiet inactive notice, and left the free sample action available.
- The production Sociobot verify endpoint enforced a per-client allowance of 30 requests in this observation: requests 1–30 returned 200; requests 31–40 returned 429 with `Retry-After: 4`.

## Required next steps

1. Close the Compose isolation bypass and add regression tests for each prohibited host-escape configuration.
2. Complete the claims inventory and tests for every retained public promise, especially CLI network privacy and installed-binary demo behavior.
3. Deploy a new candidate and repeat independent verification, ideally on a host with Docker Compose to run the real bundled PostgreSQL path.
