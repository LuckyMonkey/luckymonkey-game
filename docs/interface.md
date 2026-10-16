---
title: Lucky Monkey - interface
---

# Interface

> Black glass, green monospace, Japanese labels: the cyber-jungle terminal from
> the original design, rebuilt inside the engine.

The interface is [Bevy UI](https://docs.rs/bevy/0.20.0/bevy/?search=Node): nodes
laid out with flexbox and drawn on top of the world. The code:
- [`ui.rs`](source/ui.md): welcome screen, zoom buttons, hint bar, node panel;
- [`ops.rs`](source/ops.md): shift HUD, alert list, the panel's alert box;
- [`view.rs`](source/view.md): camera and picking;
- [`link.rs`](source/link.md): deep links.

The look follows the original v0 design:
- black panels and thin green borders;
- green [DejaVu Sans Mono](https://dejavu-fonts.github.io) text;
- Japanese section labels (ステータス Status, 接続 Connections, セキュリティ Security,
  ファイル Files) for the mood.

The colours are constants at the top of `ui.rs` (`GREEN_300` …).

## Screens

- **Welcome** ([`fn spawn_welcome`](source/ui.md#fn_spawn_welcome)): the "LUCKY
  MONKEY MEDIA / WARNING" gate from the original. The progress bar is a 0.9 s boot
  animation; everything is already loaded and the bar is there for feel. PROCEED or
  Enter moves on ([`fn proceed`](source/ui.md#fn_proceed)). The world is already
  alive behind it.
- **Playing:** everything else.

## Layout

![An incident: Time to Loss at the top, the crew and alerts on the left, the node panel on the right](img/oncall-red-links.png)

Wide screens:

- **Left column** ([`fn spawn_hud`](source/ops.md#fn_spawn_hud)), top to bottom:
  - **HUD:** shift clock → 17:00 with a progress bar, ★ score, combo, 🍌,
    **NETWORK HEALTH** and **SHIFT SLA** (against the target), and a coach line
    that always says the next sensible step.
  - **Start row:** START / 📖 Field guide (B) / Practice (P) / Tour (T).
  - **Crew roster:** one row per worker with stamina dots and what they're doing
    ("→ DNS Shrine 4s, then 📛 Restore…"). Z/X/C/V or click to choose.
    Off-shift workers show "K: call in".
  - **Alert list:** WARN / INC / HELD (mitigated) / DOWN tags, live countdowns,
    "+N more". Click to fly there.
  - **NETWORK LOG:** the shared event stream, grouped (one line for "27 services
    unreachable"), urgent lines kept longer. Click a line to fly to its node.
- **Time to Loss** (top, next to the column, [`fn ttl_box`](source/ops.md#fn_ttl_box)):
  big red numbers on a dark red plate for an incident, amber for a warning, a
  small green **✓ ALL CLEAR** otherwise. It names the node and what happens at 0.
  Click to fly there.
- **Node panel** ([`fn panel`](source/ui.md#fn_panel)), right side, scrolls with
  the wheel: status header, then the urgent block
  ([`fn alert_box`](source/ops.md#fn_alert_box): symptom, the four responses with
  what's known about each, diagnose, who will go and when they arrive), then
  description, connections, **Role**, **impact if down**, **Lore**, About, Security.
- **Outcome card** (bottom centre): RESOLVED / MITIGATED / NO EFFECT / DIAGNOSED,
  with the cause, the action, and what to learn.
- **Tour card** (bottom left) during the tour; **field guide** and **shift
  summary** as overlays.
- **Edge pins** ([`fn pins`](source/ops.md#fn_pins)): every node that's off
  screen becomes its own icon pinned to the screen edge in its direction, with the
  border coloured by health. Pins on one edge are pushed apart so they never stack.
  Click one to fly there. Back on screen, the pin disappears and the card is
  the node again.
- **Zoom** (top right) and the **hint bar** (bottom left).

Map text (countdown badges, popups, worker names, island names) sits on dark
plates so it reads over grass, water and lava alike.

**Narrow screens (under 760 px):** the column is 300 px and Time to Loss moves to
the top right. **Phones (under 600 px):** Time to Loss runs across the top, the
column fills the width under it without the log, and the node panel takes at most
the lower 55% of the screen.

## Camera

[`view.rs`](source/view.md):

- **Pan:** drag with the mouse (any button) or one finger, or use the arrows/WASD
  at 600 px/s ([`fn pan`](source/view.md#fn_pan)).
- **Zoom:** 50%-250%.
  - The wheel zooms about the cursor and a pinch about its midpoint
    ([`fn zoom_about`](source/view.md#fn_zoom_about): the world point under your
    finger stays put).
  - The buttons and +/− keys zoom about the screen centre.
  - Narrow screens start at 60%.
- **Picking:** a click that moved less than 6 px selects the topmost card under
  the pointer, using an 80×80 box test with the highest z winning
  ([`fn pick`](source/view.md#fn_pick)). Screen to world:
  [`fn to_world`](source/view.md#fn_to_world).
- **Pointer over UI:** UI elements marked `Blocks` stop a press from panning or
  selecting the map underneath ([`fn pointer_over_ui`](source/ui.md#fn_pointer_over_ui)).
- **Follow:** the camera glides to a point (Space, alert rows, F, deep links),
  aiming 80 px above the node so the panel does not cover it. Any drag cancels it
  ([`fn follow`](source/view.md#fn_follow)).

## Deep links

The URL's `#hash` can open the game at a place, with trouble staged
([`link.rs`](source/link.md), parsed by [`fn parse`](source/link.md#fn_parse),
applied by [`fn apply_link`](source/ops.md#fn_apply_link)). Any deep link skips
the welcome screen.

| Hash | Does | Try |
|---|---|---|
| `#node=<id>` | fly to a node and open its panel | [`#node=data-lake`](https://go.fridge.run/luckymonkey#node=data-lake) |
| `&zoom=<0.5-2.5>` | set the zoom | [`#node=mq-falls&zoom=1.6`](https://go.fridge.run/luckymonkey#node=mq-falls&zoom=1.6) |
| `&incident=<kind>` | start a shift with that incident on that node | [`#node=banana-sector&incident=ddos`](https://go.fridge.run/luckymonkey#node=banana-sector&incident=ddos&zoom=1.2) |
| `&forecast=<kind>` | …or that forecast | [`#node=auth-gate&forecast=cert`](https://go.fridge.run/luckymonkey#node=auth-gate&forecast=cert&zoom=1.3) |
| `#shift` | skip the welcome screen and start a shift | [`#shift`](https://go.fridge.run/luckymonkey#shift) |

- **Incident kinds:** any fault key in `tuning.json`: `crash`, `dns`, `dhcp`,
  `cable`, `route`, `ddos`, `worm`, `cert`, `leak`, `disk`, `heat`, `power` (or
  `lightning`), `deploy`, `migration`, `disparity`, `loop`, `broadcast`, `vlan`
  ([all of them](failures.md)).
- **Forecast kinds:** faults with a warning stage: `worm`, `cert`, `disk`, `deploy`.
- A staged link starts **Shift 2** (no training script); `#shift` starts **training**.
- **Node ids:** see [the node table](assets.md#what_is_whatthe_nodes).
- The director's grace period gives a staged problem the stage before other trouble arrives.

These links are why every page in these docs can say "▶ live". They are also how
the screenshots were framed. And the hash is currently the **only** way a host page
can tell the game anything (see the [extraction report](extraction-report.md)).

## Traps found while building it

These cost real time in Bevy 0.20 on Firefox, and they explain code that might
otherwise look odd:

1. **UI box shadows broke the HUD.** When the clouds were UI nodes with
   [`BoxShadow`](https://docs.rs/bevy/0.20.0/bevy/?search=BoxShadow) glows, the HUD
   and panels rendered see-through, and a cloud with a *lower* z-index drew over the
   node panel. Moving the clouds into the world as sprites fixed it. **Avoid
   `BoxShadow` in this project.**
2. **UI background alpha renders far more transparent than set.** A black panel at
   alpha 0.85 looked about 50% transparent, so values are now 0.92-0.995. To make a
   panel look 85% opaque, set about 0.96.
3. **Nothing may sit behind the canvas.** With translucent UI, Firefox composited
   the page behind the canvas, and even stale frames, into the panels. So the host
   page ([`page.tsx`](source/page-tsx.md)) removes its "Generating World" text once
   the game starts, the canvas has a black background, and the window uses
   `composite_alpha_mode: Opaque` ([`fn main`](source/main.md#fn_main)).
4. **Do not pre-create the WebGL context with `alpha: false`** before Bevy starts,
   a common trick. It made the transparency worse here, and it was reverted.
5. **Emoji need the font subset.** Text only renders glyphs present in the
   embedded fonts. A new emoji in code shows as a box until the fonts are
   re-subset with [`tools/subset-fonts.py`](source/subset-fonts.md)
   ([Extending](extending.md#add_an_emoji_or_any_non-ascii_text)).

Next: [Build, deploy and test](build-deploy.md) · Back to the [documentation](index.md)
