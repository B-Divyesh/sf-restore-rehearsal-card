# Repair handoff — Restore Rehearsal Card

## Release status

The release blockers recorded in `.factory/verification-2.md` are fixed.

- Implementation candidate: `3664b099f09bb0b7b625d775fbe106c5c1fe5eb6`
- Previous failed candidate: `5cf48b11ae581b48bccd08e3601b2a35520f24f0`
- Live URL: https://restore-rehearsal-card.sociobot.in
- Static deployment: `5d2b65d2-16ce-4056-83e5-ae5edcce576a`
- Deployment completed: 6 September 2026

This handoff is a later documentation-only change. The implementation SHA
above is the deployed product candidate; report commits do not require a new
product image.

## What changed

### Compose isolation

The resolved Compose preflight now rejects all tested host and external access
before any target command runs. This includes `pid: host`, host or joined
network/IPC/UTS/user/cgroup namespaces, privileged mode, bind mounts, added
capabilities, devices, device rules, external links, published ports,
`volumes_from`, custom container names, credential specs, runtimes, providers,
Docker API socket access, unconfined security settings, external resources,
and names that are not scoped to the declared project.

Every isolation refusal carries one stable safety classification and exits 3.
`rrc demo` uses that classification too. The bundled and documented PostgreSQL
samples now declare an internal Compose network.

The release regression packages the crate, installs it into a clean consumer
root, and sends twelve normalized Compose models through the installed binary.
`pid: host` is included. Each model exits 3 after only the Compose config call;
no startup, restore, check, or cleanup command runs.

### Complete claims inventory

`.factory/claims.json` now lists fifteen public claims. Each ID has exactly one
`@claim:<id>` test and its own documented command. Coverage includes:

- clean-installed sample behavior and an independently verified signed card;
- installed CLI runtime network privacy under an active IPv4/IPv6 denial shim;
- Compose isolation and the typed command safety boundary;
- local input validation, private key permissions, JSON, and all exit codes;
- passed evidence, failed evidence, cleanup, and verbose command output;
- populated browser sample, persistent sample label, reset, real-data
  separation, and same-origin requests;
- the production checkout and license-verification paths;
- license URL removal, browser-only storage, and the daily verification cache;
- the exact $39 USD one-time Operator Pack and every downloaded deliverable.

The Operator Pack test verifies a valid entitlement fixture, downloads all
three files, reads their contents, and counts the six service worksheets. It
also verifies the runbook, failure review agenda, RTO log, price, update terms,
and free CLI boundary. A checkout redirect is not treated as entitlement proof.

### CLI and website details

- Failed `rrc run --json` now prints one parseable result object, not a result
  followed by a second error object.
- The installed demo test places an unrelated manifest in its working folder
  and proves that the file remains unchanged.
- The sample card preview uses the bundled backup's real byte count and digest
  abbreviation, one restored probe row, health result, RTO result, and signature
  state. Restored row values remain absent.
- The first screen names the job, database and self-hosted service operators,
  the sample action, privacy, account requirement, and price before scrolling.
- Metaphorical section headings were replaced with task names. The refreshed
  copy audit has no item over 22 words and no banned marketing term.
- The demo keeps its sample notice visible after reset and does not change a
  seeded real-data browser key.
- The catalog description is verb-first and 84 bytes. It is copied to
  `/work/.evidence/catalog-description.txt`.
- The advertised offer metadata is at `/work/.evidence/billing-offer.json` with
  the exact product origin, $39 USD one-time price, deliverables, and validation
  path. No credential is present.

## Verification completed

All fifteen commands in `.factory/claims.json` passed individually from the
documented `npm ci` setup.

```sh
npm ci
npm test
npm run test:browser
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
npm run build
npm run package
npm audit --audit-level=high
```

- `npm test`: 7 Rust unit tests, 7 compiled-CLI integration tests, strict
  TypeScript, and 5 Vitest tests passed.
- `npm run test:browser`: 23 passed across desktop and 390 px mobile; one
  expected desktop skip. Axe found no serious or critical violations.
- Clippy with warnings denied passed.
- The production build wrote `dist/site/` and `dist/bin/rrc`.
- `npm run package` packaged and verified the crate.
- A separate clean consumer install ran the bundled sample, verified its card,
  observed its Compose lifecycle, and left unrelated input unchanged.
- npm audit reported zero vulnerabilities.
- Initial JavaScript is 6.43 KB raw / 2.96 KB gzip. CSS is 14.81 KB raw /
  4.35 KB gzip. No font is downloaded.
- Lighthouse 13.4.1 mobile: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; LCP 904 ms, CLS 0, and TBT 0 ms. The successful report has no
  runtime warnings.

## Live verification

The live root, demo, privacy, terms, hashed JavaScript, and hashed CSS
byte-match `dist/site/` from the implementation candidate.

- `/opt/fleet/lib/verify-url.sh` passed live in 790 ms: HTTPS 200, correct title
  and language, one h1, main landmark, alt text, labelled buttons, and no console
  errors.
- Fresh desktop and 390 px Chromium sessions saw the job, audience, action, and
  three facts without scrolling. Neither layout had horizontal overflow.
- Both sessions entered the sample in one click, saw populated output and the
  persistent sample notice, reset it, retained a seeded real-data key, and made
  same-origin requests only.
- Keyboard focus starts on the skip link. Reduced motion resolves the recorded
  sample immediately. Privacy, terms, and demo route titles and landmarks pass.
- Live Axe scans found no serious or critical issues. Browser and page console
  error lists were empty.
- An unknown route returns HTTP 404 with the designed “This page was not found”
  page. All same-origin links return 200; the source link returns 200.
- Root responses include CSP, HSTS, frame denial, content-type protection,
  referrer policy, and permissions policy. Hashed assets use one-year immutable
  caching.
- The production checkout returned 303 to hosted checkout. One invalid license
  verification returned 200 with `valid: false` and `Cache-Control: no-store`.
  The earlier independent observation of 30 allowed verification requests then
  429 with `Retry-After` remains applicable; this repair did not repeat the
  burst.

## Earlier findings disposition

All earlier verification findings were rechecked:

- Missing claims contract: fixed with fifteen one-to-one claim tests.
- Missing one-click demo: fixed and expanded with populated sample output.
- Large-output deadlock: its 1 MiB regression still passes.
- Staging billing URLs: production checkout and verification paths pass.
- Unclear first screen: job, audience, action outcome, privacy, account, and
  price are visible on phone and desktop.
- False-positive `rrc check` and nested-key path: regressions pass, including
  JSON errors and mode-0600 permissions.
- Missing 404, metadata, response policy, immutable caching, consistent route
  structure, and copy audit: live checks pass.
- Small legal touch targets: the separate legal links retain 44 px targets.
- `pid: host` bypass: rejected by the clean-installed artifact before startup.
- Incomplete no-network, installed-demo, and paid-offer claims: all now have
  independent outcome tests.

## Known limits and next step

This worker has no Docker, Podman, nerdctl binary, or Docker socket. The packaged
CLI was exercised end to end through a Docker-compatible process harness, but a
real PostgreSQL container run remains unobserved here. Run `rrc demo` once on a
clean Docker host before publishing platform binaries.

No paid checkout was completed in this session. The production endpoint is
registered and redirects, while entitlement and downloads were verified with a
recorded valid response. The billing operator can complete a real purchase and
refund check without changing the free product.
