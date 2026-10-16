---
title: Lucky Monkey - build, deploy and test
---

# Build, deploy and test

> Two files come out the far end: a 106 KB JavaScript loader and a 28 MB WebAssembly engine. This is how they are made, shipped and checked.

## One command

    tools/deploy.sh          # regenerate assets, build on fridge, deploy, restart
    tools/deploy.sh build    # stop after the build (output: fridge:~/build/luckymonkey/public/game)
    tools/deploy.sh data     # push game/data/*.json live in seconds: no Rust build, no restart

Run it on freezer from the repo root. It needs SSH as `fridge@fridge` (key
login) and, for the deploy half, the askpass helper on fridge for `sudo -A`.

## Live data

Everything you'd want to tune is JSON in `game/data/`, fetched by the page at
startup ([`data.rs`](source/data.md)):

| File | Holds |
|---|---|
| `tuning.json` | faults, responses, actions, crew, levels, the training script, concepts, notices, roles, every number |
| `islands.json` | the archipelago: islands, heights, hills, the river, bridges, islets |
| `nodes.json` | the 28 nodes: names, emoji, IPs, connections, lore |
| `world.json` | generated from the two above by `tools/gen-world.py`; don't edit by hand |
| `npcs.json` | the wandering critters |

Edit, then `tools/deploy.sh data`: it regenerates `world.json`, checks every file
is valid JSON, and copies them live. Reload the page. A partial `tuning.json`
works (it's merged over the built-in copy), and a broken one falls back to the
built-in copy with a console warning, so a typo can't take the game down.

## Why the build happens on fridge

A Bevy release build needs several GB of disk for intermediate files, and
freezer has about 3 GB free. Fridge has room, so the script copies the source
there and compiles it as user `fridge`. Cost: **a cold build takes 30+ minutes**
on fridge's 4 cores; **incremental builds take about 4 minutes** (the `target/`
directory on fridge is kept between runs).

## The toolchain (on fridge, user `fridge`, in `~/.cargo`)

| Tool | Why |
|---|---|
| [rustup](https://rustup.rs) + stable Rust (cargo 1.99) | compiler and package manager ([the Cargo book](https://doc.rust-lang.org/cargo/)) |
| target `wasm32-unknown-unknown` | compile Rust to browser WebAssembly (`rustup target add wasm32-unknown-unknown`) |
| [wasm-bindgen-cli](https://github.com/wasm-bindgen/wasm-bindgen) **0.2.129** | generates the JavaScript glue (`luckymonkey.js`) that loads the wasm and connects it to browser APIs |

**The wasm-bindgen CLI version must equal the `wasm-bindgen` crate version in
`game/Cargo.lock`.** If Cargo.lock moves to a new version, install the matching
CLI: `cargo install wasm-bindgen-cli --version X.Y.Z --locked`. A mismatch
fails with a schema version error.

Freezer needs: Node (for `export-world.cjs`; it borrows TypeScript from
`node_modules`, the shared fridge-next runtime's), Python 3 with Pillow (for
`make-textures.py`), rsync, ssh.

## What [`tools/deploy.sh`](source/deploy-sh.md) does, step by step

1. `python3 tools/gen-world.py`: build `game/data/world.json` from `islands.json` + `nodes.json`; then validate every JSON file.
   `python3 tools/gen-docs.py` regenerates [Failure types](failures.md) and [Nodes and lore](nodes.md) from the same data.
2. [`python3 tools/make-textures.py`](source/make-textures.md): regenerate `tiles.png`, `card.png`, `card_border.png`.
3. `rsync` the repo to `fridge:~/build/luckymonkey/` (excluding `node_modules`,
   `.next`, `.git`, `game/target`, `public/game`).
4. On fridge: `cargo build --release --target wasm32-unknown-unknown` in `game/`,
   then `wasm-bindgen --target web --no-typescript --out-dir ../public/game ...`.
   Release profile ([`Cargo.toml`](source/cargo-toml.md)): `opt-level = "s"` (optimise for size), thin LTO, one codegen
   unit, stripped symbols, `panic = "abort"` (all for a smaller wasm).
5. *(deploy only)* On fridge, as root through `sudo -A`:
   - `rsync` the build dir to `/srv/apps/luckymonkey/` (the live app dir, root-owned, not a git checkout);
   - `rsync` `public/game/` into it;
   - `chown -R root:root` and make it world-readable (the container mounts it read-only);
   - `fridge-next luckymonkey build` (the Next.js production build inside the shared image);
   - `fridge-next luckymonkey restart` and print the container status.

Output sizes: `luckymonkey_bg.wasm` about 28 MB raw, about **6.6 MB** as served
compressed; `luckymonkey.js` about 106 KB.

## Hosting

- **Container** `luckymonkey` from [`compose.yml`](source/compose.md): image `fridge-next:14`, app
  mounted read-only at `/opt/fridge-next/apps/luckymonkey`, `next start` on port
  3000, 256 MB memory limit, on the Docker network shared with Caddy.
  Control: `sudo fridge-next luckymonkey up|down|restart|build|logs`.
- **Base path** `/luckymonkey` (from [`next.config.mjs`](source/next-config.md), `LM_BASE_PATH`).
  `lib/asset.ts` prefixes it to the game file URLs.
- **Routing:** `fridge-caddy` sends `go.fridge.run/luckymonkey` to the container
  (see [Architecture](:dev:architecture)). `fridge.run/luckymonkey` forwards there.
  The page is public: no sign-in.
- The game files are served with `Cache-Control: public, max-age=0` plus an
  ETag, so browsers revalidate and pick up a new build immediately.

## Rollback

- **Previous Bevy build:** check out the older commit and run `tools/deploy.sh`.
- **The Pixi.js version (v0.2)** before the port is backed up at
  `fridge:~/backups/luckymonkey-pixi-20261015.tgz` (and is commit `a10d835`).

## Testing

**The rules** are tested natively: `game/simtest/` compiles `game/src/sim.rs` without Bevy.

    ssh fridge@fridge 'cd build/luckymonkey/game/simtest && ~/.cargo/bin/cargo test --release'
    ssh fridge@fridge 'cd build/luckymonkey/game/simtest && ~/.cargo/bin/cargo run --release --bin balance'

12 tests (A-L: root vs impact, mitigation, director budget, spreading, training script, crew, scoring…) and a balancing table, explained in [The simulation](simulation.md#tests_and_balancing).

**The rendering** is checked by playing it in a real browser.

    tools/browser-test.sh                                   # the live site
    tools/browser-test.sh http://127.0.0.1:8791/ ./out      # a local copy

[The script](source/browser-test.md) starts Firefox inside a virtual X display (`xvfb-run`), so it is
invisible and silent. It drives the game with `xdotool` key presses (welcome,
start shift, Space, fix 1, wait, zoom out), screenshots with ImageMagick
`import`, and fails if the console log shows a Rust panic. Look at the PNGs.

To test a build before deploying: copy `fridge:~/build/luckymonkey/public/game`
into a folder next to a minimal `index.html` that has
`<canvas id="lm-canvas">` and imports `./game/luckymonkey.js`. Serve it with
`python3 -m http.server 8791 --bind 127.0.0.1`. **Port 8765 on freezer is taken by
the HedgePets test server**; use 8791.

Rendering in the rig is software (llvmpipe), so it is slow and the first
frame takes up to 45 s. Its colours and transparency match real Firefox.

## Troubleshooting

| Symptom | Likely cause / fix |
|---|---|
| Page says "This browser has no WebGL2" | Browser or GPU without WebGL2; nothing to do server-side |
| Black screen, console `panicked at` | A Rust panic; the message names the file and line. Reproduce with the test rig |
| wasm-bindgen "schema version" error in the build | CLI version ≠ crate version (see toolchain) |
| A symbol shows as an empty box | Character not in the embedded fonts: run `tools/subset-fonts.py`, rebuild |
| New node or river not appearing | `world.json` not regenerated: `node tools/export-world.cjs` (deploy.sh does it) |
| `fridge-next luckymonkey build` fails | The script prints the last 30 lines of `/tmp/lm-build.log` on fridge (usually a TypeScript error in `app/` or `lib/`) |
| Build takes 30+ minutes | `game/target` on fridge was lost or `Cargo.lock` changed: a cold build. Wait |

Next: [Outside assets and dependencies](assets.md)
