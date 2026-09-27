# Parity status

## Matched by measurement

- Sprites, frames, positions and scales for the Scores menu, settings and both
  modals, including the two different small-button sprites and the version-arrow
  swap.
- Hitboxes: `short` 90x30, `long` 180x29, `arrow` 20x20, all anchored as measured.
- Text anchors, fonts and colours, including the red `DELETE SCORES` label and the
  black-on-white selection swap.
- Hover behaviour: hover selects a row, the large button sprite follows the
  cursor, and selection persists after leaving.
- Filters: stage, character (including "all"), time (All Time / Daily) and
  "My Score" as a real client-side filter.
- Footer text matches the native string.

## Deliberate differences

- No offline / read-only / disabled text in the UI. That information is logged
  as `SCORES_STATUS <code>` lines; the codes are not language keys.
- An empty filter result shows one neutral hint line; the original room draws
  no text for that case.
- Modal buttons apply their effect to the port's own profile and never reach the
  server; the skip is logged as `remote_write=skipped`.

## Not done

- Particles, sparkles, shaders and screen effects.
- The character grid and equipment/run-detail panels.
- Transitions and tween timing (only endpoints are measured).
- Legacy leaderboard versions: the arrows report unavailability.
- The rename field background is a filled rectangle; the original primitive was
  not captured by the inspector.
- Text rendering metrics differ between the port's renderer and Scribble, so
  glyph advance is not pixel-identical.
