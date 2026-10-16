---
title: Lucky Monkey - extending the game
---

# Extending the game

> The game is small enough that one evening is enough to add a new failure, a new place or a new tool. Here is exactly where to put your hands.

Recipes for common changes. After any change: `tools/deploy.sh build`, test
with `tools/browser-test.sh` (on a local copy, see [Build](build-deploy.md)),
then `tools/deploy.sh`, then commit and push.

## Add a node (a new place)

1. In [`lib/world-data.ts`](source/world-data.md), add an object to `allNodes`:

   ```ts
   {
     id: "dns-shrine",            // unique; used in connections
     name: "DNS Shrine",
     type: "dns",                 // decides colour + which failures can hit
     ipAddress: "10.0.0.53",      // fictional
     emoji: "⛩️",
     description: "One or two sentences for the panel.",
     position: { x: 12, y: 20, height: 0 },
     connections: ["monkey-statue", "main-router"],   // ids; links are two-way
     status: "online",            // only used before a shift starts
     securityLevel: 4,            // 0-5 squares in the panel
     assets: ["zonefile.txt"],    // fictional files for the ASSETS tab
   },
   ```

   The terrain generator automatically builds a platform (radius 1, or 2-3 for
   special types) and a grass ring around it.
2. Keep it within 25 cells of (20, 20) (see `adjustNodePositions`) and at
   least ~3 cells from other nodes, or the cards overlap.
3. New `type`? Add a colour in [`node_color()`](source/world.md#fn_node_color). If it
   stores data, add it to [`is_store()`](source/ops.md#fn_is_store).
4. New emoji? Run `tools/subset-fonts.py` (step below).
5. `node tools/export-world.cjs` (deploy.sh does this too).

## Add a failure type

All in [`game/src/ops.rs`](source/ops.md):

1. Add a variant to [`enum Failure`](source/ops.md#enum_failure).
2. Fill in every `match` in [`impl Failure`](source/ops.md#impl_failure): `emoji`, `name`, `symptom` (must
   contain the clue that picks the fix), `fix`, `ttl`, and `forecast` if it can be
   seen coming (`Some((badge emoji, text, preventive action))`).
3. If it needs a new remedy, add an `Action` (below).
4. Make it happen: add it to the reactive list in [`clock()`](source/ops.md#fn_clock) (where incidents
   are rolled), or to the forecast list, or trigger it from an event (as
   [`strikes()`](source/ops.md#fn_strikes) does for lightning). Restrict by node type with `net.kinds[i]`.
5. Special behaviour on escalation (spreading, extra SLA loss) goes in the
   TTL-expiry branch of `clock()`; spreading types are listed in `spreads()`.
6. Document it in [Failure types](failures.md) and [How to play](gameplay.md).
7. Re-subset fonts if it has a new emoji.

## Add an action

1. Add a variant to [`enum Action`](source/ops.md#enum_action) and to `Action::ALL` (the distractor pool, in [`impl Action`](source/ops.md#impl_action)).
2. Fill in `emoji`, `label`, `secs` (work time).

## Add an emoji or any non-ASCII text

The game only has the glyphs in its subset fonts:

    python3 -m venv /tmp/ft && /tmp/ft/bin/pip install fonttools
    /tmp/ft/bin/python tools/subset-fonts.py

([the script](source/subset-fonts.md))

It scans `game/src/*.rs` and the JSON assets, rebuilds the three subset fonts,
and prints any character no font covers (it should say `uncovered: none`).
New Japanese text is the exception: the CJK subset is not rebuilt by the script.
Use pyftsubset on `/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc` with
`--font-number=5` and the characters needed.

## Tune the balance

All in [`ops.rs`](source/ops.md); [How to play](gameplay.md) lists the current values. To try a change on a specific situation, use a [deep link](interface.md#deep_links) such as `#node=banana-sector&incident=ddos`.

| To change | Where |
|---|---|
| Shift length | `SHIFT_SECS` |
| Monkey speed | `LINK_SPEED`, `HIKE_SPEED` |
| Time before escalation | `Failure::ttl()`; the 40 s after going down is in `clock()` |
| Work time | `Action::secs()`; the ×1.5 for down nodes is in `controls()` |
| Trouble frequency | the `26.0 - 14.0 * p` (incidents) and `30.0 - 14.0 * p` (forecasts) formulas in `clock()` |
| SLA drain | the `0.04` / `0.2` per-second values in `clock()` |
| Penalties | `shift.sla -= 2.0` (wrong fix), `-= 6.0` (lost data) |
| Points | `work()` (fix/prevent), `clock()` (survival bonus) |
| Lightning | distance and chance in `strikes()`; flash interval in `clouds.rs` |

## Change the look

- Tile art: edit `public/sprites/*.png` (32×32 isometric blocks) and run
  `python3 tools/make-textures.py`.
- Card shape: `make-textures.py`. Card colours: `node_color()`.
- Frames: `frame()` in `main.rs` (texture), `frames()` in `ops.rs` (speed, colour, lanes).
- Clouds: `clouds.rs` (sizes, colours, flash timing), `pill()` in `main.rs`.
- UI colours: the `GREEN_*`/`RED_*` constants at the top of `ui.rs`.

Back to the [documentation index](index.md)
