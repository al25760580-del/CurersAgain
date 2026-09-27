# Roadmap

Where the port is, what comes next, and how each stage is proven. Every stage ends
with a commit; nothing here is claimed as finished before it is measured.

## Where we are

| Room | State |
|---|---|
| `rm_Pre_Intro`, `rm_Intro` | ported, strings from the language packs |
| `rm_Title` | ported; 8 icons at measured `170 + i*50`, y 305; Leaderboards and Quit route, the rest are placeholders |
| `rm_HiScores` | ported 1:1: layout, sprites, hitboxes, filters, both modals, settings submenu, footer |
| `rm_InitRoom` | first-run language prompt and profile |

Proven by side-by-side captures in `docs/parity/`, both sides in English, both
reading the same official Firestore backend.

## The plan

Each stage is one room. A stage is done when the room's state machine has unit
tests derived from the original's own code, the drawing matches a measured
capture, and the honest gap list in `docs/parity-status.md` is updated.

### Stage 1 - `rm_Options` (settings)

The room behind the title's gear-free Settings icon. Source of truth is the
original's `obj_Options` GML, which is fully available.

- Two pages: 7 options on page 0, 12 on page 1, with a 7-row window and a
  `showOptionRange` scroll offset exactly as the original computes it.
- Left/Right adjust: music volume, sound volume, language, resolution, attack
  alpha, and the `portDisplay` toggle; Confirm applies the resolution.
- Toggles on page 1: fullscreen, damage text, light FX, screen shake, HUD HP,
  HP value, above-HP, hide-full-HP, skill radius.
- `keybindMenu`: rebind the 6 action keys, one at a time or all 6 at once, with
  the original's illegal-key list and duplicate rejection.
- `controllerMenu`: 6 slots into the original's 10-entry controller button list,
  rejecting duplicate assignments.
- Quirk preserved: the Create event forces `global.Resolution = 1` after reading
  it, so a cancelled visit still leaves the stored value untouched while the
  live value is 1280x720.

Measured names, types, values and pixel geometry are in
`docs/measured-settings.md`. Note that `ValidKeysOnly` is **empty** in the
export, so only its illegal-key list and its duplicate check are verified.

Deliverable: `src/scenes/options.rs` (state machine, no drawing yet) plus tests.
The renderer follows once the room is measured live with the hover inspector.

### Stage 2 - `obj_Credits`

Four GML files, a scrolling list. Small enough to finish in one pass.

### Stage 3 - `obj_Achievements`

Nine GML files. A grid driven by unlock state, so it needs the save reader
extended to the achievement flags - read-only, as with everything else.

### Stage 4 - `rm_CharSelect`

The room behind Play. The character data (stats, descriptions, sprites) is
already extracted; this stage adds the grid, the portrait, the stat panel and
the confirm flow.

### Stage 5 - `rm_Shop`, `rm_HoloOffice`

The two remaining title routes.

### Stage 6 - gameplay

`Room1` and the stage rooms. This is the actual game and needs its own plan;
the room list is in `asset-lab/rooms.json`.

## Cross-cutting, worked in parallel

- **Triangles.** They replay a captured schedule today; the original RNG
  sequence is not recovered. This is the largest single fidelity gap in `rm_Title`.
- **Transitions and tween timing.** Only endpoints are measured.
- **Text metrics.** Glyph advance is not pixel-identical to Scribble.
- **Platform.** Linux now; Windows x64 portable build and the OS path split come
  later, as decided.

## Explicitly not claimed

- No offline, read-only or disabled text in the UI; that information is logged.
- No server writes: publishing, renaming and deletion stay disabled.
- The original installation, saves and mods are never modified.
