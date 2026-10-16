---
title: Lucky Monkey - outside assets and dependencies
---

# Outside assets and dependencies

> Nothing in a modern project is built from scratch. Here is everything Lucky Monkey stands on, where it came from, what it costs (in bytes and in licence terms), and why it was chosen.

Everything the game uses that we did not write: what it is, where it comes
from, its licence, and **why it is used**. Versions are the exact ones in
`game/Cargo.lock` and on the build host as of 2026-10-15.

## Rust libraries (compiled into the wasm)

Declared in [`game/Cargo.toml`](source/cargo-toml.md); exact versions locked in `game/Cargo.lock`. Browse any crate on [crates.io](https://crates.io) or its API on [docs.rs](https://docs.rs).

| Library | Version | What it is | Why we use it | Licence |
|---|---|---|---|---|
| [Bevy](https://bevyengine.org) ([source](https://github.com/bevyengine/bevy), [docs](https://docs.rs/bevy/0.20.0/bevy/)) | 0.20.0 | Game engine: ECS, rendering, input, UI, text, assets | The whole game runs on it. Chosen over hand-rolled Pixi.js + React for a real game loop. See [Engine](engine.md) | MIT or Apache-2.0 |
| [wgpu](https://wgpu.rs) | 30.0.1 | Graphics API layer; maps to WebGL2 in browsers | Bevy's renderer; we never call it directly | MIT or Apache-2.0 |
| [winit](https://github.com/rust-windowing/winit) | 0.30.13 | Window and input events; on the web, the canvas | Bevy's window layer: attaches to `#lm-canvas` | Apache-2.0 |
| [Taffy](https://github.com/DioxusLabs/taffy) | 0.14.0 | Flexbox/grid layout engine | Lays out Bevy UI (HUD, panels) | MIT |
| [glam](https://github.com/bitshifter/glam-rs) | 0.33.12 | Vector maths (`Vec2`, `Vec3`, `Quat`) | Every position and rotation | MIT or Apache-2.0 |
| [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) | 0.2.129 | Rust ↔ JavaScript bridge | Lets the wasm use browser APIs; its CLI generates `luckymonkey.js` | MIT or Apache-2.0 |
| [serde](https://serde.rs) | 1.0.229 | (De)serialisation framework | Turns `world.json`/`npcs.json` into Rust structs with `#[derive(Deserialize)]` | MIT or Apache-2.0 |
| [serde_json](https://github.com/serde-rs/json) | 1.0.154 | JSON for serde | Parses the embedded world data at start-up | MIT or Apache-2.0 |
| [rand](https://github.com/rust-random/rand) / [getrandom](https://github.com/rust-random/getrandom) | 0.9.5 / 0.3.4 | Random numbers | **Listed in Cargo.toml but unused** (leftover from the port). The game uses its own `Rng` in `main.rs` to avoid getrandom's web glue. Safe to remove | MIT or Apache-2.0 |

Bevy pulls in many more crates (about 400 in the lock file). Only the Bevy
*features* we need are enabled: `default_app std web webgl2 bevy_winit
default_font 2d_api 2d_bevy_render ui_api ui_bevy_render png mouse touch keyboard`.
Why: every feature left out (3D, audio, gamepads, other image formats) is less
code to compile and download.

## Web host (the page around the canvas)

Supplied by the shared image `fridge-next:14` ([fridge-next-runtime](:git:next-runtime:start));
the app lists what it needs in `package.json` under `fridgeNext.needs`.

| Package | What it is | Why | Licence |
|---|---|---|---|
| [Next.js](https://nextjs.org) 14 | React web framework and server | Serves the page and the static game files under `/luckymonkey`, consistent with other fridge web apps | MIT |
| [React](https://react.dev) | UI library | The host page (`app/page.tsx`) is a React component; it only shows "Generating World" and starts the game | MIT |
| [Tailwind CSS](https://tailwindcss.com), PostCSS, Autoprefixer | CSS tooling | Utility classes on the host page (black full-screen layout) | MIT |
| [TypeScript](https://www.typescriptlang.org) | Typed JavaScript | The host page and `lib/*.ts`; `tools/export-world.cjs` also uses its compiler to run the map generators | Apache-2.0 |

Why keep Next.js at all, when the game is two static files? The fridge.run
deployment standard for web apps is a fridge-next container behind Caddy; this
reuses its routing, base path handling and operations commands. A static file
server would also work.

## Fonts (embedded in the wasm)

Subset with [`tools/subset-fonts.py`](source/subset-fonts.md) to only the characters the game uses. Why
subset: the full fonts are several MB (the CJK and emoji ones especially); the
subsets are about 33 KB, 32 KB, 11 KB and 34 KB.

| File in `game/assets/fonts/` | Source | Used for | Licence |
|---|---|---|---|
| `DejaVuSansMono.ttf`, `DejaVuSansMono-Bold.ttf` | [DejaVu fonts](https://dejavu-fonts.github.io), from Ubuntu's `fonts-dejavu-core` (`/usr/share/fonts/truetype/dejavu/`) | All interface and label text: the green monospace terminal look of the original design | Bitstream Vera licence + public-domain changes (free to embed and redistribute) |
| `NotoSansMonoCJKjp-subset.otf` | [Noto Sans CJK](https://github.com/notofonts/noto-cjk), from Ubuntu's `fonts-noto-cjk` (`NotoSansCJK-Bold.ttc`, collection face 5, Noto Sans Mono CJK JP) | Japanese node names and section labels (メインルーター, ステータス...) | SIL Open Font License 1.1 |
| `Twemoji.Mozilla.ttf` | [Twemoji Mozilla](https://github.com/mozilla/twemoji-colr) (a colour font build of [Twemoji](https://github.com/jdecked/twemoji), originally [twitter/twemoji](https://github.com/twitter/twemoji)). Full copy kept at `~/Projects/hedgepets/assets/fonts/emoji/` | Every emoji: nodes, critters, decorations, clouds, alerts, actions, the monkey | Artwork **CC-BY 4.0** © Twitter / X Corp. and contributors (attribution required: given in the README); font build by Mozilla |

Why a bundled emoji font and not the system's: emoji look different on every
OS, and some Linux browsers have none, so the game would show boxes. Bundling
guarantees the same art everywhere, and Twemoji's flat style suits the pixel tiles.
Why not Google's Noto Color Emoji: the owner's rule is no Google tooling where a
self-hosted alternative exists.

## Images

| File | Origin | Used for | Notes |
|---|---|---|---|
| `public/sprites/*.png` (15 files, 32×32) | Came with the original [v0.dev](https://v0.dev) export (commit `9e35d5d`) | Tile tops: grass ×4, water ×5, lava ×5, temple. Packed into `game/assets/tiles.png` by `make-textures.py` | **Provenance not recorded** (likely AI-generated with the export). Treat licence as unverified before any commercial use |
| `public/images/grid_background.png` | Original v0.dev export | The tiled "void" pattern behind the islands | Same caveat |
| `art/*.png` (1024×1024: jungle background, logo, four node illustrations) | Original v0.dev export | **Not used** and not served; kept for reference | Same caveat |
| `public/icon.svg` | Made for this project (commit `a10d835`) | Browser tab icon: 🐵 in a green rounded square | Ours |
| `game/assets/tiles.png`, `card.png`, `card_border.png` | Generated by `tools/make-textures.py` | Tile atlas; node card fill and outline | Derived from the sprites / ours |
| `dot`, `frame`, `pill`, `pill_border` | Drawn in code at start-up ([`fn dot`](source/main.md#fn_dot), [`fn frame`](source/main.md#fn_frame), [`fn pill`](source/main.md#fn_pill)) | Bubbles, glints, glows, data frames, clouds | Ours; no files |

## Build and test tools

| Tool | What | Why | Where |
|---|---|---|---|
| [Rust](https://www.rust-lang.org) via [rustup](https://rustup.rs) | Compiler, cargo | Builds the game | fridge `~/.cargo` |
| [wasm-bindgen-cli](https://github.com/wasm-bindgen/wasm-bindgen) 0.2.129 | Glue generator | Produces `luckymonkey.js`; must match the crate | fridge `~/.cargo/bin` |
| [Node.js](https://nodejs.org) | JS runtime | Runs `export-world.cjs` | freezer |
| [Pillow](https://python-pillow.org) | Python imaging | `make-textures.py` builds the atlas and cards | freezer (system Python) |
| [fontTools](https://github.com/fonttools/fonttools) (`pyftsubset`) | Font toolkit | `subset-fonts.py` | a venv (not installed system-wide) |
| [Firefox](https://www.mozilla.org/firefox/), Xvfb, [xdotool](https://github.com/jordansissel/xdotool), [ImageMagick](https://imagemagick.org) | Browser, virtual display, input automation, screenshots | `browser-test.sh` plays the game headless | freezer |
| rsync, ssh, Docker, [Caddy](https://caddyserver.com) | Copying, remote build, containers, reverse proxy | Deployment and hosting | freezer / fridge |

## Reference material (concepts the game is built on)

| Link | Why it is relevant |
|---|---|
| [Bevy learn page](https://bevyengine.org/learn/), [Bevy Cheat Book](https://bevy-cheatbook.github.io) | Learning the engine (check the Cheat Book matches 0.20) |
| [Isometric tiles math](https://clintbellanger.net/articles/isometric_math/) | The grid to screen formula in `world.rs` |
| [Dijkstra's algorithm](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm) | The monkey's routing |
| [Xorshift](https://en.wikipedia.org/wiki/Xorshift) | The built-in random generator |
| [Denial-of-service attack](https://en.wikipedia.org/wiki/Denial-of-service_attack), [Rate limiting](https://en.wikipedia.org/wiki/Rate_limiting) | 🐘 DDoS stampede and its fix |
| [Computer worm](https://en.wikipedia.org/wiki/Computer_worm) | 🐛 Worm |
| [Split-brain](https://en.wikipedia.org/wiki/Split-brain_(computing)) | ⚖️ Replica disparity |
| [Canary release](https://martinfowler.com/bliki/CanaryRelease.html) | 🐤 Preventing bad deploys |
| [Failover](https://en.wikipedia.org/wiki/Failover) | ⚡ Lightning strike |
| [Transport Layer Security](https://en.wikipedia.org/wiki/Transport_Layer_Security) | 🔐 Certificates |
| [Memory leak](https://en.wikipedia.org/wiki/Memory_leak) | 🍌 Memory leak |
| [Cascading failure](https://en.wikipedia.org/wiki/Cascading_failure) | Why problems spread to neighbours |

## What is what: the nodes

All 28 nodes (from `game/assets/world.json`; position is the grid cell; links =
connections to nodes that exist). Types decide colour and which failures can
hit: data stores (`database`, `banana-cache`, `vault`, `lake`, `queue`) get
replica disparity and disk-full; the `monitor` node powers forecasts.

| | Name | id | type | IP (fictional) | position | links |
|---|---|---|---|---|---|---|
| 🐵 | Lucky Monkey Guardian | `monkey-statue` | monkey | 192.168.1.1 | (17, 17) | 4 |
| 🌊 | Logical Lagoon API | `logical-lagoon` | river | 192.168.1.42 | (17, 7) | 2 |
| 🌊 | Digital River Stream | `digital-river` | river | 192.168.1.101 | (7, 12) | 2 |
| 🍌 | Banana Sector Cache | `banana-sector` | banana-cache | 192.168.1.99 | (27, 27) | 2 |
| 🏯 | Node Temple Alpha | `node-temple-1` | temple | 10.0.0.1 | (12, 27) | 2 |
| 🏯 | Node Temple Beta | `node-temple-2` | temple | 10.0.0.2 | (27, 12) | 2 |
| 💻 | Crashed Sector 404 | `crashed-sector` | server | 192.168.1.404 | (7, 27) | 2 |
| 💻 | Server Hub Cluster | `server-hub` | server | 192.168.1.200 | (37, 17) | 1 |
| 📡 | メインルーター Router | `main-router` | router | 192.168.0.1 | (21, 21) | 3 |
| 🔥 | 北ファイアウォール Firewall | `firewall-north` | firewall | 192.168.0.10 | (31, 6) | 2 |
| 🔥 | 南ファイアウォール Firewall | `firewall-south` | firewall | 192.168.0.11 | (6, 31) | 1 |
| 🗄️ | データベース Database | `database-cluster` | database | 192.168.0.50 | (36, 36) | 1 |
| 🌋 | Volcano Core Processor | `volcano-core` | volcano | 192.168.1.666 | (8, 8) | 1 |
| 🧠 | Neural Processor Alpha | `ai-core-01` | ai-core | 10.0.1.42 | (25, 25) | 2 |
| 🧱 | Backup Vault Node | `vault-01` | vault | 10.0.0.3 | (30, 15) | 1 |
| 🔀 | Junction Switch | `switch-01` | switch | 10.0.0.2 | (20, 30) | 1 |
| 🍌 | Humor Cache Alpha | `banana-cache-01` | banana-cache | 10.0.0.4 | (35, 20) | 2 |
| 💻 | Legacy Terminal | `terminal-01` | terminal | 10.0.0.5 | (15, 35) | 1 |
| ⚛️ | Quantum Nexus Core | `quantum-nexus` | server | 192.168.q.∞ | (10, 31) | 1 |
| 📚 | Ancient Data Archive | `data-archive` | database | 10.0.1.42 | (38, 24) | 1 |
| 🦎 | Reptile Protocol | `lizard-node` | lizard | 10.0.2.42 | (13, 12) | 2 |
| 🏞️ | Data Lake | `data-lake` | lake | 10.4.0.1 | (11, 38) | 3 |
| ⛩️ | DNS Shrine | `dns-shrine` | dns | 10.0.0.53 | (12, 20) | 4 |
| 🪷 | Lilypad Load Balancer | `lb-lilypads` | balancer | 10.1.0.10 | (20, 25) | 4 |
| 📨 | Message Queue Falls | `mq-falls` | queue | 10.2.0.5 | (24, 3) | 3 |
| 🔭 | Canopy Watchtower | `watchtower` | monitor | 10.9.0.1 | (31, 31) | 3 |
| 🛂 | Auth Gate | `auth-gate` | auth | 10.3.0.1 | (30, 21) | 4 |
| 🌐 | CDN Canopy | `cdn-canopy` | cdn | 10.5.0.1 | (28, 41) | 3 |

## What is what: symbols on the map

| Symbol | Meaning |
|---|---|
| 🐒 | You: the on-call monkey (yellow bar under it = working) |
| Coloured card, steady | Node online |
| Card pulsing amber | Node unstable (an incident is open) |
| Dark card | Node down |
| Red "⏳12s" over a card | Incident countdown until it gets worse |
| Amber "in 52s" | Forecast countdown until it happens |
| DOWN | Down with nothing more to count down |
| Pale green frames on a link | Healthy traffic, both directions |
| Amber, slow frames | A link touching an unstable node |
| Red frames racing one way | DDoS flood into a node |
| Dim red, stopped | Link to a down node |
| Frames floating on rivers | The rivers are data (decoration) |
| 🐘 🐛 ⚖️ 📦 ⚡ 🔐 🍌 💾 | Incident badges (see [Failure types](failures.md)) |
| 🔐 📅 💾 🧬 (amber countdown) | Forecast badges |
| ☁️ 🌧️ ⛈️ pills | Clouds; ⛈️ flashing yellow can strike a node |
| 🐵 🐒 👻 🍌 🦎 🐊 wandering | Critters (decoration) |
| 🌴 🌸 🌿 💎 🎋 🍄 🪷 🪨 | Decorations |

Next: [Decisions and known issues](decisions.md)
