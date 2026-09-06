# Demo sandbox

## Entry points

- Website: `https://restore-rehearsal-card.sociobot.in/demo/`
- CLI: `rrc demo`

The landing-page action **Try it with sample data** opens `/demo/`. The page
shows a populated signed-card preview and the exact CLI command. Its persistent
**Demo — sample data, nothing is saved** notice includes **Reset demo** and
**Start for real** controls.

## Sample and isolation

`rrc demo` embeds and writes a PostgreSQL Compose definition with an internal
network, a one-row SQL backup, a health check, a row-count check, and a fresh
Ed25519 key to a new operating-system temporary directory. Its Compose project
is `rrc-postgres-demo`. The command runs the same runner as `rrc run`, removes
the Compose target by default, and prints both the workspace and card path. It
does not read a user manifest.

Run `rrc demo --keep-target` only when inspecting the disposable target. The
temporary workspace is intentionally left in place so the resulting card can
be inspected and verified.

The website demo stores no state. **Reset demo** restores its explanatory
sample state without reading or changing other browser data. Every CLI
invocation creates a fresh workspace. The installed-artifact claim tests run
with an unrelated manifest in the consumer directory and verify it is unchanged.
