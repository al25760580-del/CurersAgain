# CurersAgain - Rust port

A readable, modular Rust reconstruction of HoloCure's **HiScores** screen, with
parity **measured** against the retail Steam executable, plus the scripts used to
extract the original data.

## Status

The screen is measured, not guessed: sprite names, frames, positions, hitboxes,
text and colours come from live observation of the Steam build (see
`docs/measured-ui.md` (repo `rust-port/docs/`)). Remaining gaps are listed honestly in
`docs/parity-status.md`. **This is not a claim of a finished 1:1 clone.**

## Layout

```
(this crate)
src/scenes/scores.rs    rm_HiScores state, filters, navigation
src/render/scores.rs    drawing with measured sprites and positions
src/language.rs         external TOML language packs
src/score_server.rs     read-only transport to the official backend
src/native_save.rs      read-only original save reader
languages/              eng, es, id, jp text packs + layout limits
tests/                  conformance tests and SDL screenshots
docs/                   measurements, parity status, backend notes
```

## Build and run

```sh
cargo build --release
./run-title-reference.sh /path/to/extracted/assets
```

`HOLOCURE_ASSET_ROOT` must point at the extracted original sprites and fonts; they
are **not** redistributed here.

To read the official leaderboard (opt-in, read-only):

```sh
HOLOCURE_FIREBASE_CONFIG=/path/to/public-client-config.json HOLOCURE_OPEN_SCORES=1 ./target/release/title_reference /path/to/assets
```

`cargo run --bin menu` opens the separate title-menu lab prototype; it uses
placeholder art and is not the Scores screen.

## Safety model

- Reads only. No publishing, renaming or deletion on the server.
- The transport keeps a URL allow-list and rejects anything else.
- Interface buttons work completely on the client side; remote writes are skipped
  and logged (`remote_write=skipped`).
- The original game installation, saves and mods are never modified.

## Tests

```sh
cargo test --all-targets
HOLOCURE_ASSET_ROOT=... HOLOCURE_TEST_OUTPUT=... cargo test --test scores_sdl -- --ignored
```
