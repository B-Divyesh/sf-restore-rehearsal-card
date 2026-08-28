# Demo sandbox

## Entry points

- Website: `https://restore-rehearsal-card.sociobot.in/demo/`
- CLI: `rrc demo`

The landing-page action **Try it with sample data** opens `/demo/`. The page
shows the exact CLI command, a persistent **Demo — sample data, nothing is
saved** notice, **Reset demo**, and **Start for real** controls.

## Sample and isolation

`rrc demo` embeds and writes a PostgreSQL Compose definition, a one-row SQL
backup, a declared health check, a row-count check, and a fresh Ed25519 key to
a newly created operating-system temporary directory. Its Compose project is
`rrc-postgres-demo`. The command runs the same rehearsal runner as `rrc run`,
removes the Compose target by default, and prints both the temporary workspace
and signed card path. It does not read any user manifest or browser storage.

Run `rrc demo --keep-target` only when inspecting the disposable target. The
temporary workspace is intentionally left in place so the resulting card can
be inspected and verified.

The website demo stores no state. **Reset demo** only restores its explanatory
sample state; every CLI invocation creates a fresh workspace.
