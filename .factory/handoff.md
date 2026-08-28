# Restore Rehearsal Card v0.1.0 handoff

## What shipped

- A Rust `rrc` single binary with `init`, `check`, `run`, `verify`, and `keygen`
  commands, documented help, stable automation exit codes, and `--json` output.
- A v1 TOML manifest with typed Compose start/copy/exec operations; typed health,
  row-count, and exit-code assertions; per-operation timeouts; SHA-256 artifact
  evidence; RTO evaluation; and default volume/container cleanup.
- Safety boundaries: exact target-name confirmation, production-like project
  refusal, no host or container shell interpretation, copy destinations confined
  below `/tmp/restore-rehearsal/`, and resolved-Compose refusal for host networks,
  privileged services, bind mounts, and external networks/volumes.
- Privacy-safe Ed25519-signed Markdown cards for passed and failed drills, plus
  independent tamper verification. Reports omit command output and restored rows.
- A runnable PostgreSQL example and detailed manifest/threat-model documentation.
- A responsive, keyboard-accessible static documentation site with an original
  recovery-observatory illustration, recorded CLI demo, offline/error states,
  install guidance, and legal pages.
- A $39 one-time Operator Pack using the Sociobot/Dodo hosted checkout contract,
  return-token capture/removal, local storage, once-daily background verification,
  cached offline unlock, paste-to-restore, invalid/revoked handling, and three
  downloadable team resources. Core CLI, card export, safety, and accessibility
  remain free.

## Run and verify

```sh
npm ci
npm test
npm run build
npm run test:browser
npm run package
```

- `npm test`: 6 Rust unit tests, 2 compiled-CLI integration tests, 1 doctest
  target, TypeScript strict checking, and 4 license tests passed.
- `npm run test:browser`: 11 Playwright checks passed across desktop and 390 px
  mobile; 1 expected desktop-only skip. Covers console errors, Axe, landmarks,
  keyboard flow, offline state, license failure, legal pages, and overflow.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `npm audit --audit-level=high`: 0 vulnerabilities.
- `/opt/fleet/lib/verify-url.sh`: HTTP 200, title/lang/main/alt checks passed,
  0 unlabeled buttons, and 0 console errors.
- `npm run package`: crate package built and verified; the factory may publish
  with `cargo publish --manifest-path cli/Cargo.toml` when credentials and release
  policy are ready. This worker did not publish.
- A fresh local clone of the complete implementation passed `npm ci`, `npm test`, and
  `npm run build`; both required output files were present.

`npm run build` is the work-order build command. It writes the static deploy to
`dist/site/` (with `index.html` at that root) and the Linux release binary to
`dist/bin/rrc`.

## Measured quality

Lighthouse 12.8.2 against the production Vite build, mobile defaults:

| Performance | Accessibility | Best practices | SEO | LCP | CLS | TBT |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | 100 | 100 | 100 | 1.0 s | 0 | 0 ms |

- Initial site JavaScript: 6.25 KB raw / 2.89 KB gzip (budget 200 KB).
- CSS: 13.11 KB raw / 3.99 KB gzip (budget 50 KB).
- Hero imagery: 58 KB desktop and 24 KB mobile WebP (budget 300 KB).
- Fonts: 0 KB; local system stacks only.
- Release binary: approximately 1.5 MB on this Linux worker.

## Known gaps and release notes

- Docker is not installed in the build worker, so the PostgreSQL image example
  could not be exercised against a live daemon here. The complete success and
  intentionally broken flows were exercised through the real compiled CLI with
  a deterministic Docker-compatible integration harness, including cleanup,
  exit codes, card creation, privacy assertions, and signature verification.
- The checked-in buy and verify URLs intentionally use
  `https://pilot-api.sociobot.in` for staging. The factory must switch
  `VITE_BILLING_BASE_URL` and the static checkout link to
  `https://api.sociobot.in` for production after registering the slug; no product
  ID is hardcoded.
- Release archives for macOS/Windows and signed binary provenance are factory
  release work. The source package and local Linux binary are ready.

## Next useful checks

1. Run `examples/postgres` on a clean Docker host, generate its local key, and
   retain the first signed card as release evidence.
2. Register the test/live billing product and exercise checkout return, restore,
   invalid, revoked, and refund flows with factory-owned credentials.
3. Add scheduled release builds for Linux, macOS, and Windows; do not embed
   credentials or signing keys.
