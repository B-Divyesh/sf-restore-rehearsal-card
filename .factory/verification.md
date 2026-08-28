# Independent product verification — FAIL

Tested at `2026-08-28T09:59:32Z`.

- Candidate: `7887afe1ec3ad5603809e7f6ced35a5d306aa06a`
- Branch: `main`
- Live URL: `https://restore-rehearsal-card.sociobot.in`
- Artifact: Rust CLI plus static product site
- Verdict: **FAIL — do not release**

The checkout was clean and exactly at the requested candidate before testing.
The deployed HTML, JavaScript, CSS, and hero image byte-match the candidate's
production build, so the findings apply to the candidate and its live deploy.

## Mandatory gates

### Claims gate — FAIL

The first command run was the required claims gate. `.factory/claims.json` is
missing, so no claim test commands exist to run. This is release-blocking under
the acceptance contract.

The live page and README contain many unlisted claims, including no telemetry,
an isolated target, signed cards, row-count and health checks, cleanup after
failure, stable exit codes, and once-daily license verification. None is
represented by the required `@claim:<id>` test inventory.

### Cold first-read and demo — FAIL

Cold desktop and 390 px mobile reads showed:

- What it does: restore into Compose, check health and row counts, and produce a
  signed Markdown card. This is clear.
- Who it is for: the first screen does not name operators of managed databases
  or self-hosted services in plain words.
- What to click first: “Run your first drill” only jumps down to install steps;
  there is no adjacent explanation of the outcome.
- The required “Try it with sample data” action is absent.
- The first screen's facts are “Typed commands,” “No telemetry,” and “Ed25519
  signed,” not the required privacy/offline/price facts.

The later “Play demo” control reveals a prerecorded transcript. It does not run
the product, use an isolated sample, or show the required demo banner/reset/start
controls. `/demo` returns the ordinary landing page. The packaged CLI rejects
both `rrc demo` and `rrc --demo` with exit 2. `.factory/demo.md` is missing.

### Real job-to-be-done — FAIL

The installed packaged binary completed success, failure, cleanup, signature,
privacy, and recovery flows against an independent Docker-compatible command
harness. However, two functional defects prevent acceptance:

1. A declared restore command that emits 1 MiB of output falsely times out. The
   process is started with piped stdout/stderr, but the implementation waits for
   exit before draining either pipe. The child blocks when the pipe fills. With
   `timeout_seconds = 1`, the packaged CLI exited 4 after 1 second, emitted a
   signed failed card containing `operation timed out after 1s`, and then ran
   cleanup. Verbose restore tools can hit this during a normal drill.
2. `rrc check` does not validate all local inputs as its help promises. It exited
   0 and printed `"valid":true` for both a corrupt signing key and a malformed
   Compose file. `rrc init --file nested/restore.toml` with its default key path
   also creates the key outside the path later resolved from the manifest;
   `check` passes, but `run` exits 4 because the key is missing.

No Docker-compatible engine or socket exists in this verifier environment, so a
real PostgreSQL/Compose run could not be executed. The real Docker integration
therefore remains unproven in both the builder and independent environments.

## Findings by severity

### Release-blocking / high

1. **Required claims contract is absent.** `.factory/claims.json` is missing and
   all public claims are unlisted and lack claim-tagged sandbox tests.
2. **Required one-click demo is absent.** There is no first-screen sample action,
   sandbox mode, demo command, persistent demo notice, reset, or demo document.
3. **Large command output can deadlock and falsely fail rehearsals.** A 1 MiB
   output probe reproduced the failure in the packaged CLI.
4. **The production site sells through staging.** The live buy link and compiled
   verifier both use `https://pilot-api.sociobot.in`. A live click returned 303
   to `https://test.checkout.dodopayments.com/...`, not live checkout.
5. **The first screen fails the mandatory plain-words contract.** It does not
   plainly name the intended operator or provide the required sample-data action.

### Medium

1. **`check` gives false confidence.** Missing/corrupt keys and malformed Compose
   content pass as valid; nested default `init` output cannot be run as created.
2. **There is no real 404.** `/does-not-exist` returns status 200 and the home
   page. `/demo` does the same. `staticwebapp.config.json` is absent.
3. **Required metadata is missing.** There is no canonical URL, Open Graph or
   Twitter card metadata/image, or Apple touch icon.
4. **Live response policy is incomplete.** CSP, Permissions-Policy, and a
   clickjacking policy are absent. The checked-in `_headers` file is not fully
   honored by the deploy.
5. **Hashed assets are not immutable live.** JavaScript and CSS return
   `Cache-Control: public, must-revalidate, max-age=30` rather than the declared
   year-long immutable policy.
6. **The site skeleton is inconsistent.** Legal routes do not retain the home
   navigation, and footers do not expose version/build identity.
7. **Required copy audit is absent.** `.factory/copy-audit.md` is missing; the
   landing page also contains copy over the 22-word hard cap.

### Low

1. The inline Terms and Privacy links in the purchase note measure about
   `37.7×15` and `46.3×15` CSS px respectively, below the 44 px touch-target
   baseline.
2. Invalid `check --json` input produces only a plain-text stderr error rather
   than the requested JSON object for scripts.

## Passing evidence

### Clean install, tests, build, and package

- `npm ci`: passed; 97 packages installed; 0 vulnerabilities.
- `npm test`: passed — 6 Rust unit tests, 2 compiled-CLI integration tests,
  TypeScript strict checking, and 4 Vitest license tests.
- `npm run test:browser`: 11 passed across desktop and 390 px mobile; 1 expected
  desktop-only skip.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `npm run build`: passed and produced `dist/site/` plus `dist/bin/rrc`.
- `npm run package`: passed; crate was packaged and verified.
- `npm audit --audit-level=high`: passed with 0 vulnerabilities.

Production output sizes pass budget: JavaScript 6,246 bytes raw / 2.89 KB gzip,
CSS 13,112 bytes raw / 3.99 KB gzip, desktop hero 58,882 bytes, mobile hero
23,684 bytes, and no downloaded fonts.

### Clean package consumer and CLI behavior

The crate package was installed with an isolated `CARGO_HOME` and install root.
The resulting `rrc 0.1.0` binary exposed documented help and commands.

- Valid manifest `check --json`: exit 0 with a valid JSON object.
- Normal run: exit 0, signed PASS card, SHA-256 and exact row-count evidence.
- Intentionally failed check: exit 4, signed FAIL card, output omitted, cleanup
  command recorded.
- Row counts 0, 2, and `u64::MAX` against range 1..1: exit 4 with signed cards.
- Production-like target, shell argv, traversal destination, reversed range,
  and zero timeout: exit 2 with specific recovery messages.
- Wrong target confirmation: exit 3 and no card.
- Tampered card: `verify` exit 5.
- Successful rerun after failures: exit 0 and valid signature.
- Signing keys were mode 0600.
- Secret artifact content and command failure output were absent from cards.

### Live deployment, accessibility, and performance

- Candidate/live parity: exact byte matches for root, privacy, terms, hashed JS,
  hashed CSS, and desktop hero.
- `/opt/fleet/lib/verify-url.sh`: HTTP 200; title and `lang=en`; one H1; main
  landmark; 0 images missing alt; 0 unlabeled buttons; 0 console errors; load
  measured at 848 ms. Evidence: `/tmp/rrc-verify-url.pUS88Z/`.
- Independent Playwright/Axe on `/`, `/privacy/`, and `/terms/`: 0 serious or
  critical violations; 0 console errors, page errors, or failed requests.
- Keyboard: skip link is first; tab order reaches navigation, install, copy,
  demo, purchase, and license controls; focus outline is 3 px cyan.
- Responsive: no horizontal overflow at 390 px or 320 px.
- Reduced motion: transition duration becomes `0.00001s`; all seven demo lines
  appear immediately after activation.
- Initial page flow made only same-origin requests and no analytics/tracker
  requests.
- Lighthouse 12.8.2 mobile: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; LCP 961 ms, CLS 0, TBT 13 ms, FCP 945 ms.

### Privacy, purchase, and endpoint policy

- License query token was stored under the documented local-storage key and
  stripped from the URL before display.
- An invalid live token showed the correct quiet state; reload reused the cached
  verdict and made no second verification request.
- Billing CORS allowed the exact live origin and returned `Cache-Control:
  no-store`.
- Rate-limit burst against the deployed pilot verify endpoint: requests 1–30
  returned 200; request 31 returned 429 with `Retry-After: 3` and
  `X-RateLimit-After: 3`.
- Live origin serves HSTS, Referrer-Policy, and X-Content-Type-Options.
- No sign-in exists, so Entra authority checks are not applicable.
- This is not a PWA or backend product, so service-worker/offline-reload,
  persistence, and backend health/build-identity checks are not applicable.

## Release decision

**FAIL.** The missing mandatory claims suite and sample-data demo independently
force failure. The command-output deadlock, false-positive `check`, staging
checkout on the production site, and deployment-policy gaps provide additional
release blockers. Fix them, add contract tests, deploy a new candidate, and run
a fresh independent verification.
