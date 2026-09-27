# Hover inspector

A passive native plugin plus a live viewer, used to measure the UI instead of
guessing it. Source in `tools/hover-inspector/`.

## What it shows

- The original `MouseOverButton` result and arguments under the cursor.
- Text actually passed to the renderer, with position, font, alignment, effective
  colours and alpha.
- Sprite with thumbnail, id, frame, position, scale, rotation, alpha and visual
  rectangle.
- Allowed `obj_HiScores` variables, before and after the draw event.
- Per-frame differences: which resource, frame, position or colour changed.
- A conservative classification: `not determined yet`, `no observed changes`,
  `changes with selection` or `temporal variation observed`.
- Capture age, frame and room. A stale capture is never labelled as live.

## How it works

1. `obj_HiScores` draw callbacks delimit each capture.
2. Hooks on the native sprite getter read the parameters YYC really uses.
3. A hook on the YYC dispatcher, with an allow-list of draw functions, recovers
   texts that earlier hooks never exposed.
4. `MouseOverButton` supplies the original result, not a sprite-derived guess.
5. Post-draw state is used for correlation: selection can change during Draw, so
   using only pre-draw state can misreport a selection change as an animation.
6. The viewer groups calls by order and geometry. It does **not** claim that
   GameMaker exposes a DOM or native element ids.

## Limits

- While the rename field accepts text the game parks the system cursor, so hover
  cannot be driven with the physical mouse.
- `Esc` alone does not close the rename modal; cancel is `Shift+Esc`.
- Rectangle primitives are not captured, so the rename field background could not
  be measured.
- `Up`, `Down`, `Left`, `Right` produced no observed change of `deleteSelect`.
- Grouping is geometric, not by native element identity.
- A registered hook is not proof of coverage: logs must be checked per capture.
- Instrumentation changes timing; wall-clock times are not the uninstrumented
  baseline.

## Compatibility

Built for one exact executable (SHA-256
`f897d5f34228410cb74091a0278d7cd94722e0d2d194b179e4f18b009362d1c2`) with an
existing Aurie + YYToolkit + CallbackManager installation. It is not a general
GameMaker inspector. Remove it before updating the game.

## Safety

Observation only: no game variable writes, no arbitrary GML evaluation, no input
injection, no publishing, no destructive confirmation. It writes diagnostics to
`Logs` only. Those captures can contain UI text, including a typed username, so
review logs before sharing them.
