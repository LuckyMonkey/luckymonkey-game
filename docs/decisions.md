---
title: Lucky Monkey - decisions and known issues
---

# Decisions and known issues

> Every odd-looking choice in this codebase has a story. These are the stories.

Why things are the way they are. Newest concerns first in each section.

## Design decisions

| Decision | Why | Alternative rejected |
|---|---|---|
| **Bevy (Rust → wasm)** instead of Pixi.js + React | A real game loop, ECS for many small moving parts, built-in input/UI/text/timing | Keep extending Pixi: game logic tangled into React re-renders |
| **Keep the TypeScript map as source**, export to JSON | Existing, seeded, working generators; one source of truth | Rewrite generators in Rust: risk of a different world, no benefit |
| **Embed everything** in the wasm (`include_bytes!`/`include_str!`) | Two files to serve, no loading screens, no missing-asset states, cache-friendly | Bevy asset server over HTTP: more requests, async loading code |
| **Build on fridge** | Freezer lacks disk for Rust builds | Build on freezer: impossible; CI: none exists yet |
| **Own xorshift RNG** | Avoids `getrandom`'s web feature flags and JS glue | `rand` crate (still listed in Cargo.toml, unused) |
| **Terrain as one mesh per iso row** | Exact depth sorting with ~90 draw items instead of 1,700 | Per-tile sprites: correct but heavier |
| **Shift clock + per-alert TTLs** (game v2) | Owner: actions were too quick; wanted movement, countdowns, proactive *and* reactive play | Turn-based quiz with a draining bar (v1) |
| **Monkey routes over the network graph** | The network is the subject; cut-off nodes create real dilemmas | Walk over terrain: prettier, but loses the networking idea |
| **One alert per node** | Readable badges and panels; no pile-ups | Stacked alerts |
| **Wrong answers cost time + SLA, not instant failure** | Encourages reading symptoms without making mistakes fatal | Instant escalation (v1) |
| **Clouds as world sprites styled like divs** | Owner liked the original rounded divs with emoji and yellow flash; the UI-node version broke HUD rendering | Realistic textured clouds (owner: lost the charm); UI nodes (rendering trap) |
| **Bundled Twemoji** | Same emoji everywhere; no Google font | System emoji (inconsistent/missing) |
| **Frames, not dots, on links** | Owner asked for "a stream of frames": reads as network traffic | Single dots (v1) |

## Bugs found and fixed in the port

- **Tile alignment.** ([details](world.md#tiles_and_the_alignment_bug_that_started_it_all)) The Pixi version drew the 32 px isometric *block* sprites
  squashed into 64×32 rectangles, so diamonds never met. Now drawn at 2× as
  blocks; the diamond lands on the grid.
- **Upside-down pyramid.** ([code](source/world.md#fn_plugin), ▶ [see it](https://go.fridge.run/luckymonkey#node=monkey-statue&zoom=1.1)) The temple generator gave the widest layer the
  greatest height. The loader rebuilds it as a stepped pyramid.
- **Floating river.** ([code](source/world.md#fn_plugin)) The river had no ground beside it. Added banks: one full
  ring plus a ragged second ring, deliberately not more, so the void stays visible.
- **Duplicate cells.** Several generators wrote the same cell; last-written won,
  causing holes in platforms. Now the tallest wins.
- **Overlapping nodes.** Quantum Nexus sat on Crashed Sector, and the Reptile node
  was hidden behind the Guardian. Both moved (comments in `lib/world-data.ts`).
- **Node positions are clamped** by `adjustNodePositions()` in `world-data.ts`
  (anything more than 25 cells from (20, 20) is pulled in). It runs *after* the
  terrain is generated, so a clamped node can lose its platform. No current node
  is far enough to be affected; keep new nodes within 25 cells of (20, 20).
- **Missing node.** `vault-01` and `switch-01` connect to `firewall-01`, which
  does not exist; the link is skipped.

## Rendering traps (Bevy 0.20 + Firefox)

See [Interface](interface.md#traps_found_while_building_it) for details: avoid UI `BoxShadow`, use high UI
background alpha, keep nothing behind the canvas, don't pre-create the WebGL context.

## Known issues

- Panels are very slightly see-through; a node card behind the panel can show
  faintly. Cosmetic.
- The idle (pre-shift) HUD is more transparent than the in-shift HUD. Cosmetic.
- The world is the same every load (fixed seeds), and the incident sequence is
  the same too unless your timing differs. Seeding `Rng` from the clock would vary shifts.
- Balance is first-pass: worms spread quickly if ignored; the late-day rate may be harsh.
- **Not tested on phones or touch screens.** Pinch and tap are implemented; the
  narrow layout puts the panel full-width at the bottom.
- Download is about 6.6 MB compressed. `wasm-opt` (from [Binaryen](https://github.com/WebAssembly/binaryen))
  is not installed; it typically shrinks Bevy wasm by 10-20%.
- `rand`/`getrandom` are still in `Cargo.toml` although unused.
- `Interaction` is deprecated in Bevy 0.20 (hence `#![allow(deprecated)]`).
- No sound.

## Roadmap ideas

- More failure types tied to the new places: DNS outage (names stop resolving,
  routes fail), queue backlog at Message Queue Falls, token expiry at the Auth
  Gate, CDN cache poisoning.
- Cascades along dependencies (a failing database degrading the APIs that use it).
- Two-step fixes (isolate, then patch), recovery time after a fix.
- A post-shift "postmortem" screen: timeline, response times, what was prevented.
- Migrate to Bevy picking + `ui_widgets`; drop unused crates; add `wasm-opt`.
- Sound effects; a credits screen with the Twemoji attribution in-game.

Next: [Extending the game](extending.md)
