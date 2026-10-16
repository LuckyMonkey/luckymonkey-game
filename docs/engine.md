---
title: Lucky Monkey - the engine (Bevy)
---

# The engine: Bevy

> A complete game engine, compiled into a web page, drawing a jungle through
> your graphics card. Here is how it is put together.

## What Bevy is

[Bevy](https://bevyengine.org) is a free, open-source game engine written in
[Rust](https://www.rust-lang.org). Lucky Monkey uses **Bevy 0.20.0**.

- **Compiled for the browser:** the Rust code is compiled to
  [WebAssembly](https://webassembly.org), a compact binary format browsers run at
  near-native speed.
- **Drawing:** Bevy draws with
  [WebGL2](https://developer.mozilla.org/en-US/docs/Web/API/WebGL2RenderingContext)
  through its graphics layer [wgpu](https://wgpu.rs).
- **Window and input:** handled by [winit](https://github.com/rust-windowing/winit),
  which on the web means "the canvas and its DOM events".

Where to learn more:
- [Bevy's learning page and examples](https://bevyengine.org/learn/)
- [the 0.20 API reference](https://docs.rs/bevy/0.20.0/bevy/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/), if Rust itself is new
- the unofficial [Bevy Cheat Book](https://bevy-cheatbook.github.io): excellent
  explanations, but check each page matches 0.20, because Bevy changes a lot between versions.

## ECS in five minutes

Bevy is built on **ECS: Entity, Component, System.** Once it clicks, every file in
this game reads easily.

| Concept | What it is | In Lucky Monkey | Bevy docs |
|---|---|---|---|
| **Entity** | just an ID: a "thing" | a node card, a cloud, one data frame, a button | [`Entity`](https://docs.rs/bevy/0.20.0/bevy/?search=Entity) |
| **Component** | data attached to an entity | `Transform` (position), `Sprite`, `Text2d`, and our own [`LinkFrame`](source/ops.md#struct_linkframe), [`NodeCard`](source/world.md#struct_nodecard), [`Cloud`](source/clouds.md#struct_cloud) | [`Component`](https://docs.rs/bevy/0.20.0/bevy/?search=Component), [`Transform`](https://docs.rs/bevy/0.20.0/bevy/?search=Transform), [`Sprite`](https://docs.rs/bevy/0.20.0/bevy/?search=Sprite), [`Text2d`](https://docs.rs/bevy/0.20.0/bevy/?search=Text2d) |
| **System** | an ordinary function Bevy runs every frame (or once) | [`fn frames`](source/ops.md#fn_frames) moves ~940 data frames; [`fn drift`](source/clouds.md#fn_drift) moves clouds | [`System`](https://docs.rs/bevy/0.20.0/bevy/?search=System), [`Query`](https://docs.rs/bevy/0.20.0/bevy/?search=Query) |
| **Resource** | one global value | [`World`](source/world.md#struct_world) (the map), [`Net`](source/ops.md#struct_net) (network state), [`Shift`](source/ops.md#struct_shift), [`Monkey`](source/ops.md#struct_monkey), [`Art`](source/main.md#struct_art) | [`Resource`](https://docs.rs/bevy/0.20.0/bevy/?search=Resource), [`Res`](https://docs.rs/bevy/0.20.0/bevy/?search=Res) |
| **Message** | a one-frame notice from one system to another (called *events* in older Bevy) | [`Strike`](source/clouds.md#struct_strike): lightning → game; [`ZoomBy`](source/view.md#struct_zoomby): button → camera | [`Message`](https://docs.rs/bevy/0.20.0/bevy/?search=Message), [`MessageWriter`](https://docs.rs/bevy/0.20.0/bevy/?search=MessageWriter) |
| **State** | switches groups of systems on and off | [`Screen`](source/main.md#enum_screen): Welcome / Playing | [`States`](https://docs.rs/bevy/0.20.0/bevy/?search=States) |

A system's parameters say what it needs, and Bevy hands those things over. For example:

```rust
fn drift(time: Res<Time>, world: Res<World>, mut clouds: Query<(&Cloud, &mut Transform)>) {
    for (cloud, mut tf) in &mut clouds {
        // move every cloud on the wind
    }
}
```

This one means "give me the clock, the map, and every entity that has a `Cloud`
and a `Transform`, and let me change the `Transform`" ([the real code](source/clouds.md#fn_drift)).

**Why ECS fits this game:** there are thousands of small, similar things (about
940 link frames, 700 decorations, 100 river frames, hundreds of water effects),
each moved by one tiny system. ECS stores same-shaped data together, so updating
them is fast. Adding a behaviour means adding a system, not editing a class
hierarchy. The water effects were added in one file without touching anything else.

## How the app is assembled

The whole app is built in [`fn main`](source/main.md#fn_main):

```rust
App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin { /* canvas "#lm-canvas", fit to parent, opaque */ }))
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(Rng(0x9e37_79b9_7f4a_7c15))
    .init_state::<Screen>()
    .add_systems(PreStartup, load_art)
    .add_plugins((link::plugin, world::plugin, water::plugin, npc::plugin,
                  clouds::plugin, view::plugin, ui::plugin, ops::plugin))
    .run();
```

Each module is a **plugin**, a function that registers its resources and
systems ([Bevy plugins](https://docs.rs/bevy/0.20.0/bevy/?search=Plugin)):
[link](source/link.md#fn_plugin), [world](source/world.md#fn_plugin),
[water](source/water.md#fn_plugin), [npc](source/npc.md#fn_plugin),
[clouds](source/clouds.md#fn_plugin), [view](source/view.md#fn_plugin),
[ui](source/ui.md#fn_plugin), [ops](source/ops.md#fn_plugin).

`main.rs` also holds the shared pieces:

- **[`Art`](source/main.md#struct_art)**: handles to every font and texture.
  [`Art::text`](source/main.md#impl_art) returns a font *list* (DejaVu Sans Mono,
  then a Japanese subset, then emoji) so one string can mix all three; Bevy falls
  back glyph by glyph.
- **Textures drawn in code** at startup, so no image file is needed:
  - [`dot`](source/main.md#fn_dot): a soft white circle, used for bubbles, glints and glows;
  - [`frame`](source/main.md#fn_frame): a 24×14 data frame with a header bar and two payload lines;
  - [`pill`](source/main.md#fn_pill): the cloud body and its anti-aliased outline.
- **[`Rng`](source/main.md#struct_rng)**: a 12-line
  [xorshift64*](https://en.wikipedia.org/wiki/Xorshift) random generator. Why not
  the standard `rand` crate? On the web it needs `getrandom`, which needs
  JavaScript glue and feature flags. The game does not need secure randomness, so
  twelve lines of maths avoid all of that.

**Window settings**, explained:
- `canvas: "#lm-canvas"` attaches to the page's canvas.
- `fit_canvas_to_parent` follows its size.
- `prevent_default_event_handling` stops the browser scrolling or zooming the page
  when you use the wheel or pinch.
- `composite_alpha_mode: Opaque` tells the browser the canvas has no
  transparency. Without it, panels turned see-through
  ([the story](interface.md#traps_found_while_building_it)).

## When each system runs

Bevy runs systems in **schedules** ([docs](https://docs.rs/bevy/0.20.0/bevy/?search=Schedule)).
`PreStartup` → `Startup` → `PostStartup` run once; `Update` runs every frame.

| Schedule | Systems | Why there |
|---|---|---|
| PreStartup | [`load_art`](source/main.md#fn_load_art) | everything else needs fonts and textures |
| Startup | world: [terrain](source/world.md#fn_spawn_terrain), [islands](source/world.md#fn_spawn_islands), [backdrop](source/world.md#fn_spawn_backdrop), [decorations](source/world.md#fn_spawn_decorations), [nodes](source/world.md#fn_spawn_nodes) · [water](source/water.md#fn_spawn) · [critters](source/npc.md#fn_spawn_start) · [clouds](source/clouds.md#fn_spawn) · [camera](source/view.md#fn_spawn_camera) · [welcome screen](source/ui.md#fn_spawn_welcome) · [network](source/ops.md#fn_setup_net) | build the world once |
| PostStartup | [`spawn_overlay`](source/ops.md#fn_spawn_overlay): links, frames, badges, the monkey | needs the node cards from Startup to exist, to attach badges |
| OnEnter(Playing) | zoom buttons and hint bar ([ui](source/ui.md#fn_spawn_hud)) · shift HUD ([ops](source/ops.md#fn_spawn_hud)) · [`apply_link`](source/ops.md#fn_apply_link) | UI that exists only after the welcome screen; deep links |
| Update, always | water animation and river frames · critters · clouds · link colours, frames, cards, popups, monkey | the world stays alive behind the welcome screen |
| Update, Welcome | progress bar, [proceed](source/ui.md#fn_proceed) | |
| Update, Playing | camera input · panel · the game, **chained**: [`controls`](source/ops.md#fn_controls) → [`walk`](source/ops.md#fn_walk) → [`work`](source/ops.md#fn_work) → [`clock`](source/ops.md#fn_clock) → [`strikes`](source/ops.md#fn_strikes) → [`hud`](source/ops.md#fn_hud) → [`alert_list`](source/ops.md#fn_alert_list) → [`live_ttls`](source/ops.md#fn_live_ttls) | input, then logic, then the screen, in a fixed order every frame |

## Rendering: depth in a 2D isometric world

There is one 2D camera ([`Camera2d`](https://docs.rs/bevy/0.20.0/bevy/?search=Camera2d),
orthographic, so there is no perspective). In 2D, Bevy sorts what it draws by each
entity's **z**: higher z is drawn later, on top. Getting isometric overlap right is
entirely about choosing z well. The fridge's
[3D Math & Graphics lessons on projection](https://go.fridge.run/certs/learn/graphics3d?n=4)
and [visibility](https://go.fridge.run/certs/learn/graphics3d?n=5) are good background.

| z | What |
|---|---|
| −100 | the tiled grid backdrop: the void |
| −50 | translucent edge islands |
| x + y (about −10 to 90) | terrain, **one mesh per diagonal row** |
| row + 0.5, ± small offsets | things standing on a cell: decorations, critters, water effects, river frames |
| 150 | ghosts (float above everything on the ground) |
| 198 / 199 | link lines / link frames |
| 200 + (x+y)/100 | node cards (badges are children) |
| 230 | the monkey |
| 250 | score popups |
| 300+ | clouds |

**Why one mesh per diagonal row:** in an isometric view, cell (x, y) is in front of
everything with a smaller x + y. Each row's tiles go into one
[`Mesh2d`](https://docs.rs/bevy/0.20.0/bevy/?search=Mesh2d) at z = x + y
([`fn spawn_terrain`](source/world.md#fn_spawn_terrain)), and anything standing on
that row goes at z = x + y + 0.5 ([`fn row_z`](source/world.md#fn_row_z)). A tall
block in front then correctly hides a critter behind it. Per-tile sprites would
also work, but ~1,100 tiles as 72 meshes is far cheaper to draw.
[The world](world.md) has the details.

The **interface** (HUD, panels, buttons) is a separate system, `bevy_ui`
([`Node`](https://docs.rs/bevy/0.20.0/bevy/?search=Node)). It is drawn in its own
pass after the world and laid out with flexbox by
[Taffy](https://github.com/DioxusLabs/taffy).

## Bevy 0.20 specifics worth knowing

- **`Button` is deprecated** in 0.20 in favour of picking + `ui_widgets`. The code
  attaches [`Interaction`](https://docs.rs/bevy/0.20.0/bevy/?search=Interaction)
  to clickable nodes and reads `Changed<Interaction>` instead. That is why `ui.rs`
  and `ops.rs` start with `#![allow(deprecated)]`. Migrating is on the
  [roadmap](decisions.md#roadmap_ideas).
- **[`Single`](https://docs.rs/bevy/0.20.0/bevy/?search=Single)** is a query that
  expects exactly one match (the camera, the HUD text).
- **Messages** are registered with `add_message::<T>()`.
- **Fonts:** `FontSource::List` for fallback chains; sizes are `FontSize::Px`.
- **Only the features the game needs** are compiled in ([`Cargo.toml`](source/cargo-toml.md)):
  2D, UI, PNG, mouse/touch/keyboard, WebGL2, the default font. Leaving out 3D,
  audio and the rest keeps the wasm smaller and the build faster.

Next: [The world](world.md) · Back to the [documentation](index.md)
