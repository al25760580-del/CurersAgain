# Hover inspector

Passive measurement tooling for the HoloCure HiScores UI.

- `HoverInspector.cpp`: native Aurie/YYToolkit observer (build with `build.sh`).
- `viewer/`: local HTTP viewer and its parser tests.

See `rust-port/docs/hover-inspector.md` for capabilities and limits.

Build for the exact executable documented there, run the game from Steam, open
Scores and move the cursor. The viewer is read-only: it never sends input to the
game.

```sh
cd viewer
python3 -m unittest -v test_inspector.py
python3 server.py --local
```
