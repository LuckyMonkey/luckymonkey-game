//! Living water: streams pour down the waterfall, bubbles rise in the lagoon,
//! and the river surface glints.

use bevy::prelude::*;

use crate::world::{cell_hash, iso, kind, row_z, World};
use crate::Art;

#[derive(Component)]
struct Stream {
    base: Vec2,
    phase: f32,
}

#[derive(Component)]
struct Bubble {
    base: Vec2,
    phase: f32,
    size: f32,
}

/// A data frame floating down a river centreline.
#[derive(Component)]
struct RiverFrame {
    stream: usize,
    offset: f32,
}

/// A river centreline in screen space, with the grid position of each point (for depth).
struct Line {
    screen: Vec<Vec2>,
    grid: Vec<Vec2>,
    /// Cumulative length at each point.
    dist: Vec<f32>,
}

impl Line {
    fn len(&self) -> f32 {
        *self.dist.last().unwrap_or(&0.0)
    }
    /// Screen point and grid point `s` px along the line.
    fn at(&self, s: f32) -> (Vec2, Vec2) {
        let i = self.dist.partition_point(|d| *d <= s).clamp(1, self.dist.len() - 1);
        let span = (self.dist[i] - self.dist[i - 1]).max(0.001);
        let k = ((s - self.dist[i - 1]) / span).clamp(0.0, 1.0);
        (self.screen[i - 1].lerp(self.screen[i], k), self.grid[i - 1].lerp(self.grid[i], k))
    }
}

#[derive(Resource)]
struct Streams(Vec<Line>);

const FRAME_GAP: f32 = 46.0;
const FRAME_SPEED: f32 = 38.0;

#[derive(Component)]
struct Glint {
    phase: f32,
    period: f32,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn).add_systems(Update, (animate, flow));
}

fn spawn(mut commands: Commands, world: Res<World>, art: Res<Art>) {
    // The rivers are digital: a steady stream of frames rides each centreline downstream.
    let mut lines = Vec::new();
    for (si, stream) in world.streams.iter().enumerate() {
        let grid: Vec<Vec2> = stream.iter().map(|p| Vec2::new(p.x, p.y)).collect();
        let screen: Vec<Vec2> = stream.iter().map(|p| iso(p.x, p.y, world.ground(p.x, p.y).max(p.height))).collect();
        let mut dist = vec![0.0];
        for w in screen.windows(2) {
            dist.push(dist.last().unwrap() + w[0].distance(w[1]));
        }
        let line = Line { screen, grid, dist };
        if line.screen.len() < 2 {
            continue;
        }
        for i in 0..(line.len() / FRAME_GAP) as usize {
            commands.spawn((
                RiverFrame { stream: si, offset: i as f32 * FRAME_GAP + cell_hash(i as i32, si as i32) * 12.0 },
                Sprite { image: art.frame.clone(), color: Color::srgba(0.75, 0.95, 1.0, 0.0), custom_size: Some(Vec2::new(14.0, 8.0)), ..default() },
                Transform::default(),
            ));
        }
        lines.push(line);
    }
    commands.insert_resource(Streams(lines));

    for (&(x, y), t) in world.tiles.iter().filter(|(_, t)| t.kind == kind::WATER) {
        let c = iso(x as f32, y as f32, t.height);
        let z = row_z(x as f32, y as f32) - 0.2;
        let h = |a: i32, b: i32| cell_hash(x * 13 + a, y * 17 + b);
        if t.fall {
            // Three white streams per tile, like the original canvas waterfall.
            for i in 0..3 {
                commands.spawn((
                    Stream { base: c + Vec2::new((i as f32 - 1.0) * 14.0, 4.0), phase: h(i, 1) },
                    Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.0), Vec2::new(3.0, 14.0)),
                    Transform::from_translation(c.extend(z)),
                ));
            }
        }
        if t.lagoon && h(2, 3) < 0.6 {
            for i in 0..2 {
                let off = Vec2::new((h(i, 4) - 0.5) * 36.0, (h(i, 5) - 0.5) * 14.0);
                commands.spawn((
                    Bubble { base: c + off, phase: h(i, 6), size: 5.0 + h(i, 7) * 5.0 },
                    Sprite { image: art.dot.clone(), color: Color::srgba(1.0, 1.0, 1.0, 0.0), custom_size: Some(Vec2::splat(6.0)), ..default() },
                    Transform::from_translation(c.extend(z)),
                ));
            }
        }
        if !t.fall && h(8, 9) < 0.3 {
            let off = Vec2::new((h(1, 10) - 0.5) * 30.0, (h(1, 11) - 0.5) * 10.0);
            commands.spawn((
                Glint { phase: h(2, 12), period: 2.5 + h(3, 13) * 3.0 },
                Sprite { image: art.dot.clone(), color: Color::srgba(1.0, 1.0, 1.0, 0.0), custom_size: Some(Vec2::new(14.0, 5.0)), ..default() },
                Transform::from_translation((c + off).extend(z)),
            ));
        }
    }
}

fn animate(
    time: Res<Time>,
    mut streams: Query<(&Stream, &mut Transform, &mut Sprite), (Without<Bubble>, Without<Glint>)>,
    mut bubbles: Query<(&Bubble, &mut Transform, &mut Sprite), (Without<Stream>, Without<Glint>)>,
    mut glints: Query<(&Glint, &mut Sprite), (Without<Stream>, Without<Bubble>)>,
) {
    let t = time.elapsed_secs();
    for (s, mut tf, mut sprite) in &mut streams {
        // Pour down across the tile face and over its front edge, once a second.
        let k = (t + s.phase).fract();
        tf.translation.x = s.base.x;
        tf.translation.y = s.base.y - k * 30.0;
        sprite.color = sprite.color.with_alpha(0.55 * (1.0 - k) * (k * 6.0).min(1.0));
    }
    for (b, mut tf, mut sprite) in &mut bubbles {
        // Rise 20 px and pop over three seconds (the old CSS `bubble` keyframes).
        let k = (t / 3.0 + b.phase).fract();
        tf.translation.x = b.base.x + (t * 2.0 + b.phase * 6.0).sin() * 1.5;
        tf.translation.y = b.base.y + k * 20.0;
        let alpha = if k < 0.5 { 0.2 + k * 1.2 } else { 0.8 * (1.0 - k) * 2.0 };
        sprite.color = sprite.color.with_alpha(alpha * 0.8);
        sprite.custom_size = Some(Vec2::splat(b.size * (0.8 + k * 0.4)));
    }
    for (g, mut sprite) in &mut glints {
        let k = (t / g.period + g.phase).fract();
        let a = (1.0 - ((k - 0.5).abs() * 6.0)).max(0.0);
        sprite.color = sprite.color.with_alpha(a * 0.45);
    }
}

fn flow(time: Res<Time>, streams: Res<Streams>, mut frames: Query<(&RiverFrame, &mut Transform, &mut Sprite)>) {
    let t = time.elapsed_secs();
    for (f, mut tf, mut sprite) in &mut frames {
        let Some(line) = streams.0.get(f.stream) else { continue };
        let len = line.len();
        let s = (f.offset + t * FRAME_SPEED).rem_euclid(len);
        let (p, g) = line.at(s);
        let bob = (t * 2.5 + f.offset).sin() * 1.5;
        tf.translation = (p + Vec2::new(0.0, 6.0 + bob)).extend(g.x.round() + g.y.round() + 0.3);
        // Fade in at the source and out at the mouth.
        let edge = (s / 60.0).min((len - s) / 60.0).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(0.85 * edge);
    }
}
