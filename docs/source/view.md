---
title: Lucky Monkey source - view.rs
---

# Source: game/src/view.rs

**The camera: pan, zoom, touch, picking nodes, following, deep-link zoom.** Explained in [interface](../interface.md). 245 lines. [On GitHub](https://github.com/LuckyMonkey/luckymonkey/blob/main/game/src/view.rs) (needs repo access). This page is generated from the code on every deploy by `tools/wiki-source.py`; back to the [source index](index.md).

## What this file says about itself

Camera: drag to pan, wheel / pinch / buttons / keys to zoom about a point,
tap a node to select it.

## Contents

| Item | What it is |
|---|---|
| [const MIN_ZOOM](#const_min_zoom) |  |
| [const MAX_ZOOM](#const_max_zoom) |  |
| [const DRAG_THRESHOLD](#const_drag_threshold) |  |
| [struct ViewState](#struct_viewstate) | Current zoom (1 = 100%) and the node the player tapped. |
| [struct ZoomBy](#struct_zoomby) | Zoom requests from the HUD buttons, applied about the screen centre. |
| [struct Drag](#struct_drag) |  |
| [struct PointerOverUi](#struct_pointeroverui) | Set by the UI each frame when the pointer is over a panel or button. |
| [fn plugin](#fn_plugin) |  |
| [fn spawn_camera](#fn_spawn_camera) |  |
| [type Cam](#type_cam) |  |
| [fn to_world](#fn_to_world) | Screen point -> world point for the current camera. |
| [fn follow](#fn_follow) |  |
| [fn pan](#fn_pan) |  |
| [fn zoom_about](#fn_zoom_about) | Zoom by `factor`, keeping the world point under `at` (screen px) fixed. |
| [fn pick](#fn_pick) | Top-most node card under a world point (cards are 80x80). |
| [fn mouse](#fn_mouse) |  |
| [fn touch](#fn_touch) |  |
| [fn keys](#fn_keys) |  |
| [fn zoom_buttons](#fn_zoom_buttons) |  |

## Imports

```rust
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::touch::Touches;
use bevy::prelude::*;
use crate::Screen;
use crate::world::{NodeCard, World};
```

## Code

### const MIN_ZOOM

```rust
pub const MIN_ZOOM: f32 = 0.5;
```

### const MAX_ZOOM

```rust
pub const MAX_ZOOM: f32 = 2.5;
```

### const DRAG_THRESHOLD

```rust
const DRAG_THRESHOLD: f32 = 6.0;
```

### struct ViewState

Current zoom (1 = 100%) and the node the player tapped.

```rust
#[derive(Resource)]
pub struct ViewState {
    pub zoom: f32,
    pub selected: Option<usize>,
    /// Glide the camera here (cleared when reached or when the player pans).
    pub focus: Option<Vec2>,
    /// Set the zoom to this (deep links), applied once.
    pub zoom_to: Option<f32>,
}
```

### struct ZoomBy

Zoom requests from the HUD buttons, applied about the screen centre.

```rust
#[derive(Message)]
pub struct ZoomBy(pub f32);
```

### struct Drag

```rust
#[derive(Resource, Default)]
struct Drag {
    last: Option<Vec2>,
    moved: f32,
    pinch: Option<f32>,
    /// The press started over UI, so it must not pan or select.
    on_ui: bool,
}
```

### struct PointerOverUi

Set by the UI each frame when the pointer is over a panel or button.

```rust
#[derive(Resource, Default)]
pub struct PointerOverUi(pub bool);
```

### fn plugin

```rust
pub fn plugin(app: &mut App) {
    app.insert_resource(ViewState { zoom: 1.0, selected: None, focus: None, zoom_to: None })
        .init_resource::<Drag>()
        .init_resource::<PointerOverUi>()
        .add_message::<ZoomBy>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (mouse, touch, keys, zoom_buttons, follow).run_if(in_state(Screen::Playing)));
}
```

### fn spawn_camera

```rust
fn spawn_camera(mut commands: Commands, world: Res<World>, windows: Query<&Window>, mut view: ResMut<ViewState>) {
    let narrow = windows.iter().next().is_some_and(|w| w.width() < 768.0);
    view.zoom = if narrow { 0.6 } else { 1.0 };
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection { scale: 1.0 / view.zoom, ..OrthographicProjection::default_2d() }),
        Transform::from_translation(world.bounds.center().extend(0.0)),
    ));
}
```

### type Cam

```rust
type Cam<'w, 's> = Single<'w, 's, (&'static mut Transform, &'static mut Projection), With<Camera2d>>;
```

### fn to_world

Screen point -> world point for the current camera.

```rust
fn to_world(win: &Window, cam: &Transform, zoom: f32, p: Vec2) -> Vec2 {
    let off = p - win.size() / 2.0;
    cam.translation.truncate() + Vec2::new(off.x, -off.y) / zoom
}
```

### fn follow

```rust
fn follow(time: Res<Time>, cam: Cam, mut view: ResMut<ViewState>) {
    let (mut cam, mut proj) = cam.into_inner();
    if let Some(z) = view.zoom_to.take() {
        view.zoom = z.clamp(MIN_ZOOM, MAX_ZOOM);
        if let Projection::Orthographic(o) = &mut *proj {
            o.scale = 1.0 / view.zoom;
        }
    }
    let Some(target) = view.focus else { return };
    // Aim a little above the node so the panel at the bottom doesn't cover it.
    let target = target - Vec2::Y * 80.0 / view.zoom;
    let p = cam.translation.truncate();
    let k = 1.0 - (-time.delta_secs() * 8.0).exp();
    cam.translation = p.lerp(target, k).extend(cam.translation.z);
    if p.distance(target) < 2.0 {
        view.focus = None;
    }
}
```

### fn pan

```rust
fn pan(cam: &mut Transform, zoom: f32, screen_delta: Vec2) {
    cam.translation.x -= screen_delta.x / zoom;
    cam.translation.y += screen_delta.y / zoom;
}
```

### fn zoom_about

Zoom by `factor`, keeping the world point under `at` (screen px) fixed.

```rust
fn zoom_about(win: &Window, cam: &mut Transform, proj: &mut Projection, view: &mut ViewState, factor: f32, at: Vec2) {
    let before = to_world(win, cam, view.zoom, at);
    view.zoom = (view.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    if let Projection::Orthographic(o) = proj {
        o.scale = 1.0 / view.zoom;
    }
    let after = to_world(win, cam, view.zoom, at);
    cam.translation += (before - after).extend(0.0);
}
```

### fn pick

Top-most node card under a world point (cards are 80x80).

```rust
fn pick(world_pos: Vec2, cards: &Query<(&NodeCard, &GlobalTransform)>) -> Option<usize> {
    cards
        .iter()
        .filter(|(_, tf)| {
            let d = (world_pos - tf.translation().truncate()).abs();
            d.x <= 40.0 && d.y <= 40.0
        })
        .max_by(|a, b| a.1.translation().z.total_cmp(&b.1.translation().z))
        .map(|(c, _)| c.0)
}
```

### fn mouse

```rust
fn mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    window: Single<&Window>,
    cam: Cam,
    mut view: ResMut<ViewState>,
    mut drag: ResMut<Drag>,
    over_ui: Res<PointerOverUi>,
    cards: Query<(&NodeCard, &GlobalTransform)>,
) {
    let (mut tf, mut proj) = cam.into_inner();
    let cursor = window.cursor_position();
    let any = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
    if buttons.any_just_pressed(any) {
        drag.on_ui = over_ui.0;
        drag.last = cursor;
        drag.moved = 0.0;
        if !over_ui.0 {
            view.focus = None;
        }
    }
    if buttons.any_pressed(any) && !drag.on_ui {
        if let (Some(last), Some(cur)) = (drag.last, cursor) {
            let d = cur - last;
            drag.moved += d.x.abs() + d.y.abs();
            pan(&mut tf, view.zoom, d);
        }
        drag.last = cursor;
    }
    if buttons.just_released(MouseButton::Left) && !drag.on_ui && drag.moved <= DRAG_THRESHOLD {
        if let Some(c) = cursor {
            if let Some(i) = pick(to_world(&window, &tf, view.zoom, c), &cards) {
                view.selected = Some(i);
            }
        }
    }
    for ev in wheel.read() {
        if over_ui.0 {
            continue;
        }
        let px = match ev.unit {
            MouseScrollUnit::Line => ev.y * 40.0,
            MouseScrollUnit::Pixel => ev.y,
        };
        let at = cursor.unwrap_or(window.size() / 2.0);
        zoom_about(&window, &mut tf, &mut proj, &mut view, (px * 0.0015).exp(), at);
    }
}
```

### fn touch

```rust
fn touch(
    touches: Res<Touches>,
    window: Single<&Window>,
    cam: Cam,
    mut view: ResMut<ViewState>,
    mut drag: ResMut<Drag>,
    over_ui: Res<PointerOverUi>,
    cards: Query<(&NodeCard, &GlobalTransform)>,
) {
    let (mut tf, mut proj) = cam.into_inner();
    if touches.any_just_pressed() && touches.iter().count() == 1 {
        drag.on_ui = over_ui.0;
        drag.moved = 0.0;
        if !over_ui.0 {
            view.focus = None;
        }
    }
    let active: Vec<_> = touches.iter().collect();
    match active.as_slice() {
        [one] if !drag.on_ui => {
            let d = one.delta();
            drag.moved += d.x.abs() + d.y.abs();
            pan(&mut tf, view.zoom, d);
            drag.pinch = None;
        }
        [a, b, ..] => {
            let dist = a.position().distance(b.position());
            if let Some(prev) = drag.pinch.filter(|p| *p > 0.0) {
                let mid = (a.position() + b.position()) / 2.0;
                zoom_about(&window, &mut tf, &mut proj, &mut view, dist / prev, mid);
            }
            drag.pinch = Some(dist);
            drag.moved = DRAG_THRESHOLD + 1.0;
        }
        _ => drag.pinch = None,
    }
    for t in touches.iter_just_released() {
        if !drag.on_ui && drag.moved <= DRAG_THRESHOLD && touches.iter().count() == 0 {
            if let Some(i) = pick(to_world(&window, &tf, view.zoom, t.position()), &cards) {
                view.selected = Some(i);
            }
        }
    }
}
```

### fn keys

```rust
fn keys(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, window: Single<&Window>, cam: Cam, mut view: ResMut<ViewState>) {
    let (mut tf, mut proj) = cam.into_inner();
    let mut d = Vec2::ZERO;
    for (k, v) in [
        (KeyCode::KeyA, Vec2::X),
        (KeyCode::KeyD, -Vec2::X),
        (KeyCode::KeyW, Vec2::Y),
        (KeyCode::KeyS, -Vec2::Y),
    ] {
        if keys.pressed(k) {
            d += v;
        }
    }
    if d != Vec2::ZERO {
        view.focus = None;
        pan(&mut tf, 1.0, d.normalize() * 600.0 * time.delta_secs());
    }
    let centre = window.size() / 2.0;
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        zoom_about(&window, &mut tf, &mut proj, &mut view, 1.2, centre);
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        zoom_about(&window, &mut tf, &mut proj, &mut view, 1.0 / 1.2, centre);
    }
    if keys.just_pressed(KeyCode::Escape) {
        view.selected = None;
    }
}
```

### fn zoom_buttons

```rust
fn zoom_buttons(mut requests: MessageReader<ZoomBy>, window: Single<&Window>, cam: Cam, mut view: ResMut<ViewState>) {
    let (mut tf, mut proj) = cam.into_inner();
    for ZoomBy(f) in requests.read() {
        zoom_about(&window, &mut tf, &mut proj, &mut view, *f, window.size() / 2.0);
    }
}
```
