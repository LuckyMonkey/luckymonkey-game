//! Clouds look like the original floating "divs": translucent pills with a
//! rounded 1px border and a cloud emoji inside. They drift across the map on one
//! wind and wrap; rain clouds drizzle, storm clouds flash yellow and drop a ⚡ -
//! and in a shift, a strike on a node knocks its power out (see `ops`).
//!
//! They are world sprites (not UI nodes): UI box shadows broke the stacking and
//! blending of the HUD, and as sprites they pan and zoom with the map for free.

use bevy::prelude::*;

use crate::world::{emoji_text, World};
use crate::{Art, Rng};

const COUNT: usize = 10;
const CLOUD_Z: f32 = 300.0;
const FLASH_SECS: f32 = 0.6;

#[derive(Clone, Copy, PartialEq)]
enum Weather {
    Fair,
    Rain,
    Storm,
}

/// Lightning hit the ground at this world position.
#[derive(Message)]
pub struct Strike(pub Vec2);

#[derive(Component)]
struct Cloud {
    weather: Weather,
    speed: f32,
    next_flash: f32,
    /// Seconds into the current flash, if one is running.
    flash: Option<f32>,
}

/// Parts of a cloud that change during a flash.
#[derive(Component)]
struct Fill;
#[derive(Component)]
struct Bolt;
#[derive(Component)]
struct Drop {
    phase: f32,
    /// Bottom edge of the pill, local y.
    top: f32,
}

pub fn plugin(app: &mut App) {
    app.add_message::<Strike>().add_systems(Startup, spawn).add_systems(Update, (drift, rain, storm));
}

/// Wind direction (Bevy coordinates): left to right, sinking slightly.
fn wind() -> Vec2 {
    Vec2::new(1.0, -0.3).normalize()
}

fn fill(w: Weather) -> Color {
    match w {
        Weather::Fair => Color::srgba(1.0, 1.0, 1.0, 0.2),
        Weather::Rain => Color::srgba(0.376, 0.647, 0.98, 0.2),
        Weather::Storm => Color::srgba(0.42, 0.447, 0.502, 0.35),
    }
}

fn border(w: Weather) -> Color {
    match w {
        Weather::Fair => Color::srgba(1.0, 1.0, 1.0, 0.3),
        Weather::Rain => Color::srgba(0.576, 0.773, 0.992, 0.3),
        Weather::Storm => Color::srgba(0.612, 0.639, 0.686, 0.4),
    }
}


fn spawn(mut commands: Commands, world: Res<World>, art: Res<Art>, mut rng: ResMut<Rng>) {
    let b = world.bounds;
    for i in 0..COUNT {
        let weather = match i % 5 {
            0 | 2 | 4 => Weather::Fair,
            1 => Weather::Rain,
            _ => Weather::Storm,
        };
        // The old small / medium / large divs.
        let (size, emoji_px) = [(Vec2::new(80.0, 40.0), 24.0), (Vec2::new(120.0, 55.0), 30.0), (Vec2::new(160.0, 75.0), 36.0)][rng.below(3)];
        let glyph = match weather {
            Weather::Fair => "☁️",
            Weather::Rain => "🌧️",
            Weather::Storm => "⛈️",
        };
        let pos = Vec2::new(b.min.x + rng.f32() * b.width(), b.min.y + rng.f32() * b.height());
        commands
            .spawn((
                Cloud { weather, speed: 10.0 + rng.f32() * 14.0, next_flash: 2.0 + rng.f32() * 6.0, flash: None },
                Transform::from_translation(pos.extend(CLOUD_Z + i as f32 * 0.5)),
                Visibility::Inherited,
            ))
            .with_children(|c| {
                c.spawn((Fill, Sprite { image: art.pill.clone(), color: fill(weather), custom_size: Some(size), ..default() }, Transform::from_xyz(0.0, 0.0, -0.01)));
                c.spawn((Sprite { image: art.pill_border.clone(), color: border(weather), custom_size: Some(size), ..default() }, Transform::default()));
                c.spawn((emoji_text(&art, glyph, emoji_px), TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)), Transform::from_xyz(0.0, 0.0, 0.01).with_scale(Vec3::splat(0.5))));
                if weather == Weather::Storm {
                    c.spawn((
                        Bolt,
                        emoji_text(&art, "⚡", 30.0),
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                        Transform::from_xyz(0.0, -size.y / 2.0 - 16.0, 0.01).with_scale(Vec3::splat(0.5)),
                    ));
                }
                if weather == Weather::Rain {
                    for d in 0..5 {
                        c.spawn((
                            Drop { phase: d as f32 * 0.15, top: -size.y / 2.0 },
                            Sprite::from_color(Color::srgba(0.576, 0.773, 0.992, 0.0), Vec2::new(2.0, 16.0)),
                            Transform::from_xyz(size.x * (-0.35 + d as f32 * 0.18), -size.y / 2.0, -0.01),
                        ));
                    }
                }
            });
    }
}

fn drift(time: Res<Time>, world: Res<World>, mut clouds: Query<(&Cloud, &mut Transform)>) {
    let b = world.bounds;
    let margin = 300.0;
    for (cloud, mut tf) in &mut clouds {
        let mut p = tf.translation.truncate() + wind() * cloud.speed * time.delta_secs();
        // Wrap so the sky never empties: leave on the right/bottom, return on the left/top.
        if p.x > b.max.x + margin {
            p.x = b.min.x - margin;
        }
        if p.y < b.min.y - margin {
            p.y = b.max.y + margin;
        }
        tf.translation = p.extend(tf.translation.z);
    }
}

fn rain(time: Res<Time>, mut drops: Query<(&Drop, &mut Transform, &mut Sprite)>) {
    let t = time.elapsed_secs();
    for (drop, mut tf, mut sprite) in &mut drops {
        // The old `rain-drop` keyframes: fall 25 px, fade in by 30%, out by the end.
        let k = (t + drop.phase).fract();
        tf.translation.y = drop.top - 8.0 - k * 25.0;
        let a = if k < 0.3 { k / 0.3 * 0.8 } else { 0.8 * (1.0 - k) / 0.7 };
        sprite.color = sprite.color.with_alpha(a * 0.6);
    }
}

fn storm(
    time: Res<Time>,
    mut rng: ResMut<Rng>,
    mut strikes: MessageWriter<Strike>,
    mut clouds: Query<(&mut Cloud, &Transform, &Children)>,
    mut fills: Query<&mut Sprite, With<Fill>>,
    mut bolts: Query<&mut TextColor, With<Bolt>>,
) {
    let dt = time.delta_secs();
    for (mut cloud, tf, children) in &mut clouds {
        if cloud.weather != Weather::Storm {
            continue;
        }
        cloud.next_flash -= dt;
        if cloud.next_flash <= 0.0 {
            cloud.flash = Some(0.0);
            cloud.next_flash = 4.0 + rng.f32() * 5.0;
        }
        // Ramp up, hold, ramp down: the pill itself lights up yellow.
        let lit = match cloud.flash {
            Some(before) => {
                let f = before + dt;
                if before < FLASH_SECS / 2.0 && f >= FLASH_SECS / 2.0 {
                    strikes.write(Strike(tf.translation.truncate() - Vec2::Y * 60.0));
                }
                cloud.flash = (f < FLASH_SECS).then_some(f);
                (1.0 - ((f / FLASH_SECS - 0.5).abs() * 2.0 - 0.34).max(0.0) / 0.66).clamp(0.0, 1.0)
            }
            None => 0.0,
        };
        let gray = fill(Weather::Storm).to_srgba();
        for c in children.iter() {
            if let Ok(mut s) = fills.get_mut(c) {
                s.color = Color::Srgba(gray.mix(&Srgba::new(0.992, 0.878, 0.278, 0.5), lit));
            }
            if let Ok(mut b) = bolts.get_mut(c) {
                b.0 = b.0.with_alpha(lit);
            }
        }
    }
}
