# Visual thesis: the recovery observatory

Restore Rehearsal Card uses a **luminous glass data landscape**: a dark, quiet
operator's room in which a backup crosses a chain of translucent checkpoints
and arrives as a healthy service. The depth is explanatory, not decorative.
Glass strata distinguish artifact, isolated target, assertions, and signed
evidence—the four things an operator must keep mentally separate during a
recovery drill.

## Palette

The site is deliberately single-mode. Recovery drills happen in consoles and
status rooms, and a painted midnight background lets luminous state changes
read instantly without borrowing a generic dashboard aesthetic.

| Token | Value | Role |
| --- | --- | --- |
| `night` | `#07110f` | page background |
| `basin` | `#0d1c19` | raised glass surface |
| `glass` | `rgba(21, 53, 47, .72)` | translucent layer |
| `chalk` | `#f2f5ec` | primary text |
| `mist` | `#b7c9c0` | secondary text (7.7:1 on night) |
| `signal` | `#a8ffcf` | actions and passed paths |
| `signal-ink` | `#062116` | text on signal |
| `ice` | `#8ee8ff` | timing and evidence |
| `amber` | `#ffc86a` | warnings |
| `flare` | `#ff8f8f` | errors |

All copy meets 4.5:1 against its painted surface. State always carries a word,
shape, or icon as well as color.

## Type and spacing

The display face is the local system serif stack (`Iowan Old Style`, `Palatino
Linotype`, `Book Antiqua`, Georgia) for the sober feel of a signed field note.
Interface copy and code use the system sans and monospace stacks. No font is
downloaded. The scale is 16, 18, 22, 31, 44, and 64 px with 1.5 body leading.
The spacing rhythm is 4/8 px; common intervals are 8, 16, 24, 40, 64, and 96.
Reading measures stop at 68 characters.

## Shape, icon, and interaction grammar

Panels have clipped corners and hairline inner highlights like labelled glass
slides. A four-node route—artifact → target → checks → card—is the recurring
motif. Buttons are solid signal green; secondary actions are quiet outlined
controls. Focus is a 3 px ice ring with an offset. Targets are at least 44 px.
The phone layout drops atmospheric side labels, stacks the route vertically,
and keeps the CLI recipe ahead of pricing.

## Motion

On first view, route nodes resolve in source-to-evidence order over 600 ms.
The recorded terminal advances only after an explicit Play action and can be
paused. Hover and disclosure transitions last 180–240 ms and animate only
opacity/transform. Under `prefers-reduced-motion`, sequencing and movement are
removed; all content remains visible and state changes are instant.

## Original asset plan and provenance

- `site/public/recovery-observatory.webp`: generated specifically for this
  product with the factory `factory-image` deployment, then locally converted
  to WebP. Prompt: “Wide editorial 3D illustration for a database recovery CLI
  landing page. A dark obsidian data landscape with four translucent glass
  stations connected left to right: sealed backup capsule, isolated container
  basin, two verification beacons, signed evidence card. Luminous mint and icy
  cyan internal light, subtle amber fault shard, technical but calm, tactile
  glass, deep negative space, no people, no logos, no lettering, no UI, no
  watermark. Cinematic orthographic perspective, dark edges that blend into
  #07110f.” License: project-owned generated asset.
- All interface glyphs are hand-made inline SVG using simple geometry and are
  decorative unless paired with a text label. No third-party icon library.

This system fits the product because it makes an otherwise invisible recovery
path spatial: operators can see the separation boundary, checkpoints, and
evidence trail before reading a command.
