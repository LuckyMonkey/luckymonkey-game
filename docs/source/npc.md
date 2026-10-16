---
title: Lucky Monkey source - npc.rs
---

# Source: game/src/npc.rs

**Wandering critters: monkeys, ghosts, bananas, lizards, crocodiles.** Explained in [world](../world.md). 195 lines. [On GitHub](https://github.com/LuckyMonkey/luckymonkey/blob/main/game/src/npc.rs) (needs repo access). This page is generated from the code on every deploy by `tools/wiki-source.py`; back to the [source index](index.md).

## What this file says about itself

Wandering critters: monkeys, ghosts, bananas, lizards, crocodiles.

Logic ticks every 80 ms like the original; sprites glide toward the logical
position every frame and stand on whatever terrain is under them.

## Contents

| Item | What it is |
|---|---|
| [const STEP_SECS](#const_step_secs) |  |
| [const MAX_ACTIVE](#const_max_active) |  |
| [const SPAWN_CHANCE](#const_spawn_chance) |  |
| [const START_COUNT](#const_start_count) |  |
| [const LIFESPANS](#const_lifespans) |  |
| [const DIRS](#const_dirs) |  |
| [const MAP_MIN](#const_map_min) |  |
| [const MAP_MAX](#const_map_max) |  |
| [struct NpcIn](#struct_npcin) |  |
| [struct Roster](#struct_roster) |  |
| [struct Npc](#struct_npc) |  |
| [enum Kind](#enum_kind) |  |
| [impl Kind](#impl_kind) |  |
| [struct Clock](#struct_clock) |  |
| [fn plugin](#fn_plugin) |  |
| [fn spawn](#fn_spawn) |  |
| [fn spawn_start](#fn_spawn_start) |  |
| [fn tick](#fn_tick) |  |
| [fn place](#fn_place) |  |

## Imports

```rust
use bevy::prelude::*;
use serde::Deserialize;
use crate::world::{iso, row_z, World, emoji_text, kind};
use crate::{Art, Rng};
```

## Code

### const STEP_SECS

```rust
const STEP_SECS: f32 = 0.08;
```

### const MAX_ACTIVE

```rust
const MAX_ACTIVE: usize = 20;
```

### const SPAWN_CHANCE

```rust
const SPAWN_CHANCE: f32 = 0.16;
```

### const START_COUNT

```rust
const START_COUNT: usize = 12;
```

### const LIFESPANS

```rust
const LIFESPANS: [f32; 3] = [30.0, 60.0, 90.0];
```

### const DIRS

```rust
const DIRS: [(f32, f32); 8] = [(0., -1.), (1., -1.), (1., 0.), (1., 1.), (0., 1.), (-1., 1.), (-1., 0.), (-1., -1.)];
```

### const MAP_MIN

```rust
const MAP_MIN: f32 = 5.0;
```

### const MAP_MAX

```rust
const MAP_MAX: f32 = 35.0;
```

### struct NpcIn

```rust
#[derive(Deserialize, Clone)]
struct NpcIn {
    #[serde(rename = "type")]
    kind: String,
    x: f32,
    y: f32,
    direction: usize,
    speed: f32,
    emoji: String,
    lifespan: f32,
}
```

### struct Roster

```rust
#[derive(Resource)]
struct Roster(Vec<NpcIn>);
```

### struct Npc

```rust
#[derive(Component)]
pub struct Npc {
    kind: Kind,
    x: f32,
    y: f32,
    dir: usize,
    speed: f32,
    born: f32,
    lifespan: f32,
    shown: Option<Vec2>,
}
```

### enum Kind

```rust
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Monkey,
    Ghost,
    Banana,
    Lizard,
    Crocodile,
}
```

### impl Kind

```rust
impl Kind {
    fn parse(s: &str) -> Kind {
        match s {
            "ghost" => Kind::Ghost,
            "banana" => Kind::Banana,
            "lizard" => Kind::Lizard,
            "crocodile" => Kind::Crocodile,
            _ => Kind::Monkey,
        }
    }
    /// Chance per tick of picking a new heading.
    fn wander(self) -> f32 {
        match self {
            Kind::Monkey => 0.02,
            Kind::Ghost | Kind::Banana => 0.05,
            Kind::Lizard => 0.03,
            Kind::Crocodile => 0.01,
        }
    }
}
```

### struct Clock

```rust
#[derive(Resource)]
struct Clock(Timer);
```

### fn plugin

```rust
pub fn plugin(app: &mut App) {
    let roster: Vec<NpcIn> = serde_json::from_str(crate::data::npcs_json()).expect("npcs.json");
    app.insert_resource(Roster(roster))
        .insert_resource(Clock(Timer::from_seconds(STEP_SECS, TimerMode::Repeating)))
        .add_systems(Startup, spawn_start)
        .add_systems(Update, (tick, place).chain());
}
```

### fn spawn

```rust
fn spawn(commands: &mut Commands, art: &Art, n: &NpcIn, now: f32, lifespan: f32) {
    commands.spawn((
        Npc { kind: Kind::parse(&n.kind), x: n.x, y: n.y, dir: n.direction % 8, speed: n.speed, born: now, lifespan, shown: None },
        emoji_text(art, &n.emoji, 26.0),
        TextColor(Color::WHITE),
        Transform::from_scale(Vec3::splat(0.5)),
        Visibility::Hidden,
    ));
}
```

### fn spawn_start

```rust
fn spawn_start(mut commands: Commands, art: Res<Art>, roster: Res<Roster>) {
    for n in roster.0.iter().take(START_COUNT.min(MAX_ACTIVE)) {
        spawn(&mut commands, &art, n, 0.0, n.lifespan / 1000.0);
    }
}
```

### fn tick

```rust
fn tick(
    mut commands: Commands,
    time: Res<Time>,
    mut clock: ResMut<Clock>,
    mut rng: ResMut<Rng>,
    world: Res<World>,
    art: Res<Art>,
    roster: Res<Roster>,
    mut npcs: Query<(Entity, &mut Npc)>,
) {
    clock.0.tick(time.delta());
    let now = time.elapsed_secs();
    let mut dead = Vec::new();
    for _ in 0..clock.0.times_finished_this_tick() {
        let mut alive = 0;
        for (e, mut npc) in &mut npcs {
            if dead.contains(&e) {
                continue;
            }
            if now - npc.born > npc.lifespan {
                commands.entity(e).despawn();
                dead.push(e);
                continue;
            }
            alive += 1;
            let (dx, dy) = DIRS[npc.dir];
            let (nx, ny) = (npc.x + dx * npc.speed, npc.y + dy * npc.speed);
            let in_bounds = (MAP_MIN..=MAP_MAX).contains(&nx) && (MAP_MIN..=MAP_MAX).contains(&ny);
            let ok = in_bounds
                && match npc.kind {
                    Kind::Banana => true,
                    Kind::Lizard | Kind::Crocodile => world.tile_at(nx, ny).is_some_and(|t| t.kind == kind::WATER),
                    _ => world.tile_at(nx, ny).is_some(),
                };
            if ok {
                npc.x = nx;
                npc.y = ny;
            } else {
                npc.dir = (npc.dir + 4) % 8;
            }
            if (ok || npc.kind == Kind::Banana) && rng.f32() < npc.kind.wander() {
                npc.dir = rng.below(8);
            }
        }
        if alive < MAX_ACTIVE && rng.f32() < SPAWN_CHANCE {
            let mut n = roster.0[rng.below(roster.0.len())].clone();
            n.x += rng.f32() * 4.0 - 2.0;
            n.y += rng.f32() * 4.0 - 2.0;
            n.direction = rng.below(8);
            spawn(&mut commands, &art, &n, now, LIFESPANS[rng.below(3)]);
        }
    }
}
```

### fn place

```rust
fn place(time: Res<Time>, world: Res<World>, mut npcs: Query<(&mut Npc, &mut Transform, &mut TextColor, &mut Visibility)>) {
    let t = time.elapsed_secs();
    let ease = 1.0 - (-time.delta_secs() * 14.0).exp();
    for (mut npc, mut tf, mut color, mut vis) in &mut npcs {
        let mut h = world.ground(npc.x, npc.y);
        if npc.kind == Kind::Banana {
            h += (t * 10.0).sin().abs() * 0.5;
        }
        let target = iso(npc.x, npc.y, h) + Vec2::Y * 10.0;
        let shown = match npc.shown {
            Some(s) => s.lerp(target, ease),
            None => target,
        };
        npc.shown = Some(shown);
        let mut p = shown;
        match npc.kind {
            Kind::Banana => p.y += (t * 10.0).sin() * 5.0,
            Kind::Ghost => {
                p.y += (t * 3.0).sin() * 4.0;
                color.0 = Color::srgba(1.0, 1.0, 1.0, 0.7 + (t * 2.0).sin() * 0.3);
            }
            _ => {}
        }
        // Fade in on spawn and out in the last second of life.
        let age = t - npc.born;
        let fade = (age / 0.6).min((npc.lifespan - age) / 1.0).clamp(0.0, 1.0);
        if npc.kind != Kind::Ghost {
            color.0 = Color::srgba(1.0, 1.0, 1.0, fade);
        } else {
            color.0 = color.0.with_alpha(color.0.alpha() * fade);
        }
        // Ghosts float above everything; the rest are depth-sorted with the terrain.
        let z = if npc.kind == Kind::Ghost { 150.0 } else { row_z(npc.x, npc.y) };
        tf.translation = p.extend(z);
        *vis = Visibility::Inherited;
    }
}
```
