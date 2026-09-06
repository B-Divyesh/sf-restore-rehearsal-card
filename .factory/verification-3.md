# Restore Rehearsal Card CLI verification — FAIL

Tested at `2026-09-06T05:21:21Z`.

- Implementation candidate: `3664b099f09bb0b7b625d775fbe106c5c1fe5eb6`
- Documentation SHA: `6d8866c3113bd1ad19ad961ce81f01e7893f2021`
- Deployment: `5d2b65d2-16ce-4056-83e5-ae5edcce576a`
- Live URL: https://restore-rehearsal-card.sociobot.in
- Artifact: Rust CLI plus static product site
- Finding count: **1**
- Untested claim count: **1**
- Verdict: **FAIL — do not release**

The later documentation commit changes only `.factory/handoff.md`. The built and
live product was compared with implementation candidate `3664b09`.

## Release-blocking finding

### High — the one-command PostgreSQL demo cannot start on a clean Docker host

The public claim says a clean-installed `rrc` runs the bundled PostgreSQL sample
with `rrc demo`. The documented prerequisites name Docker Compose, but do not
require the PostgreSQL image to be downloaded first.

Independent reproduction:

1. Installed Docker 29.1.3 and Docker Compose 2.40.3 in the fresh verifier.
2. Installed the published source shape with the documented command,
   `cargo install --git https://github.com/B-Divyesh/sf-restore-rehearsal-card`.
3. Confirmed the fresh daemon had no `postgres:17-alpine` image.
4. Confirmed the CLI's Compose start command always includes
   `--no-build --pull never` (`cli/src/runner.rs`).
5. Against that fresh daemon, Compose with the sample image and the same
   `--pull never` policy returned `No such image: postgres:17-alpine`.

The declared `sample-cli-demo` command nevertheless passes because
`scripts/verify-installed-artifact.sh` replaces Docker with a process harness.
That harness returns canned success for image startup, PostgreSQL health,
restore execution, and row count. It proves CLI orchestration, card creation,
privacy, and cleanup, but does not prove that PostgreSQL starts or restores.

The verifier also attempted the installed `rrc demo` against a fresh local
daemon. This container lacks the kernel capability needed to create the
sample's internal bridge network, so the real run exited 4 before PostgreSQL
could start. The signed failed card correctly omitted command output and the
CLI attempted cleanup. Because a capable Docker host was unavailable, the
actual PostgreSQL restore remains the one untested claim.

Required repair: make `rrc demo` acquire its fixed sample image when missing,
or ship it in a form that needs no preloaded image. Then run the installed
artifact on a clean, Docker-enabled host and verify the real row, health check,
cleanup, card, and signature. Merely documenting a manual image pull would not
satisfy the one-command demo contract.

## Claims gate

All 15 exact commands from `.factory/claims.json` were run separately after
`npm ci`. Every command exited 0, and every claim ID has exactly one source tag.
The command result does not override the incomplete sandbox for the first
claim.

| Claim | Command | Disposition |
| --- | --- | --- |
| `sample-cli-demo` | `bash scripts/verify-installed-artifact.sh demo` | **FAIL / incomplete** — clean install and harness pass; no real PostgreSQL runs, and a clean image cache cannot satisfy `--pull never` |
| `compose-isolation` | `bash scripts/verify-installed-artifact.sh isolation` | PASS — 12 unsafe resolved models refused before startup |
| `cli-no-network` | `bash scripts/verify-installed-artifact.sh network` | PASS — installed demo made no IPv4 or IPv6 socket attempt under the denial shim |
| `safe-command-surface` | named Cargo integration test | PASS |
| `large-command-output` | named Cargo integration test | PASS — 1 MiB output path completed |
| `check-local-inputs` | named Cargo integration test | PASS |
| `signed-private-card` | named Cargo integration test | PASS |
| `failed-drill-card` | named Cargo integration test | PASS |
| `stable-exit-codes` | named Cargo integration test | PASS |
| `sample-demo-route` | tagged Playwright test | PASS on desktop and phone |
| `site-no-analytics` | tagged Playwright test | PASS on desktop and phone |
| `operator-pack-offer` | tagged Playwright test | PASS on desktop and phone; all three files and six worksheets inspected |
| `production-checkout` | tagged Playwright test | PASS on desktop and phone |
| `production-license-verification` | tagged Vitest test | PASS |
| `license-storage-cache` | tagged Playwright test | PASS on desktop and phone |

Public landing, demo, legal, README, CLI help, and manifest-reference statements
were cross-checked against the inventory. No additional unlisted public claim
was found. The future-update wording is part of the stated one-time commercial
offer; the current deliverables and entitlement boundary are tested.

## Clean build and package gates

- `npm ci`: PASS; 97 packages, 0 vulnerabilities.
- `npm test`: PASS; 7 Rust unit tests, 7 CLI integration tests, strict
  TypeScript, and 5 Vitest tests.
- `npm run test:browser`: PASS; 23 passed, 1 intentional project skip.
- `cargo fmt --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `npm run build`: PASS; produced `dist/site/` and `dist/bin/rrc`.
- `npm run package`: PASS; packaged and verified 15 crate files.
- `npm audit --audit-level=high`: PASS; 0 vulnerabilities.
- Documented Git install: PASS from a fresh install root; `rrc --help` exposes
  `init`, `check`, `run`, `verify`, `keygen`, and `demo`.

The initial JavaScript is 6.43 KB raw / 2.96 KB gzip. CSS is 14.81 KB raw /
4.33 KB gzip. No web font is downloaded.

## Live site and demo

Fresh 1440×900 desktop and 390×844 phone contexts were opened before scrolling.
Both showed:

- Job: “Test your backup before an incident.”
- Audience: database and self-hosted service operators.
- First action: “Try it with sample data.”
- Immediate outcome: opens the shipped PostgreSQL sample and exact CLI command.
- Facts: local CLI/no telemetry, no account, and free CLI/$39 templates.

The action opened `/demo/` in one click. The page immediately showed PASS,
target `rrc-postgres-demo`, 152-byte backup, checksum abbreviation, healthy
service, one expected row, RTO met, and valid Ed25519 signature. The persistent
sample notice, Reset demo, and Start for real remained visible. Reset restored
the sample state and did not change a seeded `real:qa-sentinel` local-storage
value. Both flows made same-origin requests only and produced no console, page,
or failed-request errors.

The root, demo, privacy, terms, 404, hashed JavaScript, hashed CSS, and hero
image byte-match `dist/site/`. This proves the live runtime corresponds to the
implementation candidate despite the later documentation-only commit.

## Accessibility, responsive behavior, and performance

- `/opt/fleet/lib/verify-url.sh`: PASS in 661 ms; title, `lang=en`, one h1,
  main landmark, alt text, labelled buttons, and zero console errors.
- Fresh live Axe scans on `/`, `/demo/`, `/privacy/`, `/terms/`, and
  `/404.html`: zero serious or critical violations.
- Each route has one h1, header, navigation, main, footer, and its own title.
- First Tab focuses the skip link. Its visible focus ring is 3 px ice blue with
  a 4 px offset. A complete phone tab cycle reached every visible control and
  returned to the skip link without a trap.
- Empty license submission focuses the input, sets `aria-invalid`, and announces
  the recovery message through the described polite live region.
- All 19 visible phone controls measured at least 44×44 CSS px.
- Phone and 200% text-size checks had no horizontal overflow or lost primary
  content.
- Reduced motion resolves all seven recorded-demo lines in about 10 ms and
  removes the hover transform.
- The product intentionally uses one documented dark treatment. Axe found no
  serious color-contrast violation.
- Lighthouse 13.4.1 mobile: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; LCP 1,038 ms, CLS 0, TBT 0 ms.

## Routes, privacy, billing, and response policy

- All same-origin links and the GitHub source link returned 200.
- The production buy link returned 303 to `checkout.dodopayments.com`, not the
  pilot/test checkout.
- An invalid returned license was stored only in the documented browser key,
  removed from the visible URL, returned `valid: false`, left the free action
  available, and used `Cache-Control: no-store` plus exact-origin CORS.
- The product verification API allowed 30 requests; request 31 returned 429
  with `Retry-After: 2`. This is the applicable live request allowance check.
- The site has no product backend, tenant store, or server persistence; backend
  health, tenant isolation, and restart persistence are not applicable.
- The site is not a PWA and makes no offline-reload or update claim. Its loaded
  page displays the documented reconnect notice when connectivity is removed.
- The live root sends CSP, HSTS, frame denial, MIME sniffing protection,
  referrer policy, and permissions policy. Hashed assets use one-year immutable
  caching.
- `robots.txt` and `sitemap.xml` return 200 and list all public routes.
- An unknown URL returns HTTP 404 with the designed “This page was not found”
  page and a working return action. This expected 404 is not a defect.

The billing metadata path referenced in the incoming handoff was absent from
`/work/.evidence` at the start of this verification. That did not leave the
public offer untested: the live $39 USD one-time offer, production checkout,
verification API, cached license flow, and every downloadable deliverable were
independently checked without completing a purchase or using a credential.

## Earlier findings disposition

| Earlier finding | Current disposition |
| --- | --- |
| Claims inventory absent/incomplete | Fixed: 15 entries and 15 unique tags; one sandbox remains incomplete as described above |
| One-click website demo absent | Fixed live on desktop and phone |
| Large command output deadlock | Fixed; 1 MiB regression passes |
| Staging billing endpoints | Fixed; production checkout and verifier observed |
| First screen omits audience/action/facts | Fixed before scrolling on desktop and phone |
| `rrc check` false positives and nested key path | Fixed; malformed Compose and corrupt key fail, nested key is mode 0600 |
| Missing designed 404 | Fixed; unknown URL returns the expected HTTP 404 page |
| Missing metadata and response policy | Fixed live |
| Hashed assets not immutable | Fixed live |
| Inconsistent route skeleton and missing build identity | Fixed live |
| Copy audit missing or long copy | Fixed; audit present with no flagged line |
| Small legal touch targets | Fixed; no visible phone target below 44×44 px |
| Invalid JSON command result | Fixed; every documented status parses as one JSON object |
| `pid: host` isolation bypass | Fixed; installed artifact rejects it before startup |
| CLI privacy and paid-offer claims incomplete | Fixed by installed socket-denial and downloadable-content tests |

## Decision

**FAIL.** There is one release-blocking finding and one untested claim. All
other tested product behavior and quality gates pass. Repair the clean-image
demo path, then execute the installed binary against real PostgreSQL on a clean
Docker host before repeating verification.
