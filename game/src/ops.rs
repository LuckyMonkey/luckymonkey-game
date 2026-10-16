#![allow(deprecated)] // Interaction: see ui.rs.

//! On-call shifts: input and presentation for the simulation in sim.rs.
//!
//! The simulation (faults, the incident director, workers, the event stream)
//! knows nothing about Bevy. This module owns one `Sim`, steps it every frame,
//! turns input into orders, and draws it: the Time to Loss timer, the worker
//! roster, open alerts, the shared event feed, outcome cards, the field guide,
//! the end-of-shift summary, and the map overlays (links, frames, badges,
//! workers, edge pins).

use bevy::prelude::*;

use crate::clouds::Strike;
use crate::data::tuning;
use crate::sim::{self, Cat, Health, Phase, Sim, SimNode, Stage};
use crate::ui::{Blocks, GREEN_300, GREEN_400, GREEN_500, GREEN_900, RED_400, RED_500};
use crate::view::ViewState;
use crate::world::{emoji_text, iso, node_color, NodeCard, World, NODE_Z};
use crate::{Art, Rng, Screen};

pub const AMBER: Color = Color::srgb(0.98, 0.75, 0.14);
pub const RED: Color = Color::srgb(0.94, 0.27, 0.27);
const SLATE: Color = Color::srgb(0.45, 0.52, 0.62);
const TEXT: Color = Color::srgb(0.88, 0.97, 0.9);
const PANEL_BG: Color = Color::srgb(0.0, 0.0, 0.0);

/// The simulation plus the player's UI choices.
#[derive(Resource)]
pub struct Game {
    pub sim: Sim,
    /// Worker picked in the roster (None: the best one for each job).
    pub chosen: Option<usize>,
    /// Difficulty of the next shift (training first).
    pub next_level: usize,
    guide_open: bool,
    /// Outcome card shown (its time), and dismissed.
    outcome_seen: f32,
    outcome_hidden: bool,
    /// Last order error, shown in the hint line for a moment.
    pub notice: Option<(String, f32)>,
    /// Network tour: position in `tour_order`.
    tour: Option<usize>,
    tour_order: Vec<usize>,
    /// Scores of every shift played (this browser), newest last.
    history: Vec<Played>,
    recorded: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Played {
    name: String,
    score: u32,
    sla: f32,
    met: bool,
}

const HISTORY_KEY: &str = "luckymonkey-scores";

fn load_history() -> Vec<Played> {
    #[cfg(target_arch = "wasm32")]
    {
        let s = web_sys::window().and_then(|w| w.local_storage().ok().flatten()).and_then(|st| st.get_item(HISTORY_KEY).ok().flatten());
        if let Some(s) = s {
            return serde_json::from_str(&s).unwrap_or_default();
        }
    }
    Vec::new()
}

fn save_history(h: &[Played]) {
    #[cfg(target_arch = "wasm32")]
    {
        if let (Some(st), Ok(s)) = (web_sys::window().and_then(|w| w.local_storage().ok().flatten()), serde_json::to_string(h)) {
            let _ = st.set_item(HISTORY_KEY, &s);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = h;
}

impl Game {
    pub fn vec(&self, i: usize) -> Vec2 {
        Vec2::from(self.sim.nodes[i].pos)
    }
    fn err(&mut self, s: impl Into<String>) {
        self.notice = Some((s.into(), self.sim.clock));
    }
}

/// One straight piece of a link (ropes between islands have many).
#[derive(Component)]
struct LinkLine {
    link: usize,
    len: f32,
    rope: bool,
}
/// Each link's path on screen: two points, or a sagging rope.
#[derive(Resource)]
struct LinkCurves(Vec<Vec<Vec2>>);
const ROPE_SEGS: usize = 14;
const ROPE: Color = Color::srgb(0.72, 0.57, 0.36);
#[derive(Component)]
struct TourCard;
#[derive(Component)]
struct TourText;
#[derive(Component)]
struct PracticeButton;
#[derive(Component)]
struct TourButton;
#[derive(Component)]
struct StartRow;
#[derive(Component)]
struct LinkFrame {
    link: usize,
    forward: bool,
    offset: f32,
}
#[derive(Component)]
struct Badge(usize);
#[derive(Component)]
struct BadgeTtl(usize);
#[derive(Component)]
struct WorkerToken(usize);
#[derive(Component)]
struct WorkBar(usize);
#[derive(Component)]
struct Popup {
    born: f32,
}
#[derive(Component)]
struct HudText;
#[derive(Component)]
struct HintText;
#[derive(Component)]
struct HudBar;
#[derive(Component)]
struct StartButton;
#[derive(Component)]
struct StartLabel;
#[derive(Component)]
struct TtlBox;
#[derive(Component)]
struct TtlTitle;
#[derive(Component)]
struct TtlTime;
#[derive(Component)]
struct TtlWhat;
#[derive(Component)]
struct RosterRow(usize);
#[derive(Component)]
struct RosterText(usize);
#[derive(Component)]
struct AlertList;

/// The left HUD column; edge pins keep out of it. `top` is its offset from the top.
#[derive(Component)]
struct HudColumn {
    top: f32,
}
#[derive(Component)]
struct AlertRow(usize);
#[derive(Component)]
struct Feed;
#[derive(Component)]
struct FeedRow(usize);
#[derive(Component)]
struct OutcomeCard;
#[derive(Component)]
struct OutcomeText;
#[derive(Component)]
struct GuideButton;
#[derive(Component)]
struct GuidePanel;
#[derive(Component)]
struct SummaryPanel;
#[derive(Component)]
struct SummaryText;
#[derive(Component)]
struct SummaryButton;
/// A node's stand-in pinned to the screen edge while the node is off screen.
#[derive(Component)]
struct EdgePin(usize);

/// A fix button in the node panel (built by `alert_box`), by option index.
#[derive(Component)]
pub struct ActionButton(pub usize);
#[derive(Component)]
pub struct DiagnoseButton;
#[derive(Component)]
pub struct GoButton;
#[derive(Component)]
pub struct SwingButton;
#[derive(Component)]
pub struct CancelButton(pub usize);
/// Live countdown text (alert list and node panel).
#[derive(Component)]
pub struct PanelTtl(pub usize);

#[derive(Resource, Default)]
struct Seen {
    alerts: Option<u32>,
    feed: Option<u32>,
    guide: Option<(usize, bool)>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Seen>()
        .add_systems(Startup, setup)
        .add_systems(PostStartup, spawn_overlay)
        .add_systems(OnEnter(Screen::Playing), (spawn_hud, spawn_pins, apply_link).chain())
        .add_systems(Update, (links, frames, cards, popups, worker_tokens))
        .add_systems(Update, (phone_column, pins, pin_clicks, outcome_card, guide, summary, tour_card).run_if(in_state(Screen::Playing)))
        .add_systems(
            Update,
            (navigate, controls, tick, strikes, hud, ttl_box, roster, alert_list, feed, live_ttls).chain().run_if(in_state(Screen::Playing)),
        );
}

fn setup(mut commands: Commands, world: Res<World>, mut rng: ResMut<Rng>) {
    let nodes: Vec<SimNode> = world
        .nodes
        .iter()
        .map(|n| {
            let p = &n.position;
            let at = iso(p.x, p.y, world.ground(p.x, p.y).max(p.height));
            SimNode { id: n.id.clone(), name: n.name.clone(), kind: n.kind.clone(), pos: at.into() }
        })
        .collect();
    let mut links = Vec::new();
    for (i, node) in world.nodes.iter().enumerate() {
        for c in &node.connections {
            if let Some(j) = world.nodes.iter().position(|o| &o.id == c) {
                let l = (i.min(j), i.max(j));
                if i != j && !links.contains(&l) {
                    links.push(l);
                }
            }
        }
    }
    // A different day every session (deterministic within it; tests pin seeds).
    #[cfg(target_arch = "wasm32")]
    let seed = js_sys::Date::now() as u64 ^ ((rng.f32() * 1e6) as u64) << 20;
    #[cfg(not(target_arch = "wasm32"))]
    let seed = ((rng.f32() * 1e6) as u64) << 16 | 0x5eed;
    let sim = Sim::new(tuning().rules.clone(), nodes, links, seed);
    // The tour walks the islands in order, node by node.
    let mut tour_order = Vec::new();
    for isl in &world.islands {
        for id in &isl.nodes {
            if let Some(i) = world.nodes.iter().position(|n| &n.id == id) {
                tour_order.push(i);
            }
        }
    }
    for i in 0..world.nodes.len() {
        if !tour_order.contains(&i) {
            tour_order.push(i);
        }
    }
    commands.insert_resource(Game {
        sim,
        chosen: None,
        next_level: 1,
        guide_open: false,
        outcome_seen: -1.0,
        outcome_hidden: false,
        notice: None,
        tour: None,
        tour_order,
        history: load_history(),
        recorded: false,
    });
}

fn clock_str(t: f32, shift: f32) -> String {
    let h = 9.0 + (t / shift).clamp(0.0, 1.0) * 8.0;
    format!("{:02}:{:02}", h as u32, (h.fract() * 60.0) as u32)
}

fn mmss(s: f32) -> String {
    let s = s.max(0.0).ceil() as u32;
    format!("{:02}:{:02}", s / 60, s % 60)
}

// ------------------------------------------------------------------ map overlay

const FRAME_GAP: f32 = 34.0;

/// A link's path: straight within an island, a slack rope between islands.
fn curve(a: Vec2, b: Vec2, rope: bool) -> Vec<Vec2> {
    if !rope {
        return vec![a, b];
    }
    let sag = (a.distance(b) * 0.12).clamp(24.0, 120.0);
    let c = (a + b) / 2.0 - Vec2::Y * sag * 2.0;
    (0..=ROPE_SEGS)
        .map(|k| {
            let t = k as f32 / ROPE_SEGS as f32;
            a * (1.0 - t) * (1.0 - t) + c * 2.0 * t * (1.0 - t) + b * t * t
        })
        .collect()
}

/// Point and direction at fraction `k` along a path.
fn along(pts: &[Vec2], k: f32) -> (Vec2, Vec2) {
    let n = pts.len() - 1;
    let x = k.clamp(0.0, 1.0) * n as f32;
    let i = (x as usize).min(n - 1);
    (pts[i].lerp(pts[i + 1], x - i as f32), (pts[i + 1] - pts[i]).normalize_or_zero())
}

fn path_len(pts: &[Vec2]) -> f32 {
    pts.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// A dark rounded plate behind map text so it reads against the jungle.
fn plate(art: &Art, size: Vec2, alpha: f32) -> impl Bundle {
    (Sprite { image: art.pill.clone(), color: Color::srgba(0.0, 0.0, 0.0, alpha), custom_size: Some(size), ..default() }, Transform::from_xyz(0.0, 0.0, -0.005))
}

fn spawn_overlay(mut commands: Commands, game: Res<Game>, art: Res<Art>, world: Res<World>, cards: Query<(Entity, &NodeCard)>) {
    let mut curves = Vec::new();
    for (i, &(a, b)) in game.sim.links.iter().enumerate() {
        let (pa, pb) = (game.vec(a), game.vec(b));
        let rope = world.nodes[a].island != world.nodes[b].island;
        let pts = curve(pa, pb, rope);
        for w in pts.windows(2) {
            let d = w[1] - w[0];
            // Pieces overlap a little so the rope has no gaps at the joints.
            let len = d.length() + if rope { 3.0 } else { 0.0 };
            commands.spawn((
                LinkLine { link: i, len, rope },
                Sprite::from_color(Color::WHITE, Vec2::new(len, 2.0)),
                Transform::from_translation(((w[0] + w[1]) / 2.0).extend(NODE_Z - 2.0)).with_rotation(Quat::from_rotation_z(d.to_angle())),
            ));
        }
        let count = (path_len(&pts) / FRAME_GAP).max(1.0) as usize;
        curves.push(pts);
        for forward in [true, false] {
            for k in 0..count {
                commands.spawn((
                    LinkFrame { link: i, forward, offset: (k as f32 + if forward { 0.0 } else { 0.5 }) / count as f32 },
                    Sprite { image: art.frame.clone(), color: Color::WHITE, custom_size: Some(Vec2::new(12.0, 7.0)), ..default() },
                    Transform::from_translation(pa.extend(NODE_Z - 1.0)),
                ));
            }
        }
    }
    commands.insert_resource(LinkCurves(curves));
    for (e, card) in &cards {
        commands.entity(e).with_children(|c| {
            c.spawn((Badge(card.0), emoji_text(&art, "⚠️", 30.0), Transform::from_xyz(0.0, 62.0, 0.01).with_scale(Vec3::splat(0.5)), Visibility::Hidden))
                .with_children(|b| {
                    b.spawn((BadgeTtl(card.0), Text2d::new(""), art.text(true, 22.0), TextColor(RED_400), Transform::from_xyz(0.0, 40.0, 0.01)))
                        .with_child(plate(&art, Vec2::new(104.0, 32.0), 0.85));
                });
        });
    }
    for (w, wk) in game.sim.workers.iter().enumerate() {
        commands
            .spawn((WorkerToken(w), emoji_text(&art, &wk.emoji, 34.0), Transform::from_translation(Vec2::from(wk.pos).extend(NODE_Z + 30.0)).with_scale(Vec3::splat(0.5))))
            .with_children(|c| {
                c.spawn((Text2d::new(wk.name.to_uppercase()), art.text(true, 16.0), TextColor(TEXT), Transform::from_xyz(0.0, -36.0, 0.01)))
                    .with_child(plate(&art, Vec2::new(wk.name.len() as f32 * 11.0 + 20.0, 24.0), 0.75));
                c.spawn((WorkBar(w), Sprite::from_color(Color::srgb(1.0, 0.85, 0.2), Vec2::new(60.0, 6.0)), Transform::from_xyz(0.0, -50.0, 0.01), Visibility::Hidden));
            });
    }
}

/// Is this link part of what the player is looking at (selection or trouble)?
fn link_focus(game: &Game, view: &ViewState, l: usize) -> bool {
    let (a, b) = game.sim.links[l];
    view.selected.is_some_and(|s| s == a || s == b) || [a, b].iter().any(|&n| game.sim.faults[n].as_ref().is_some_and(|f| f.stage != Stage::Warning))
}

fn link_health(sim: &Sim, l: usize) -> Health {
    let (a, b) = sim.links[l];
    let rank = |h: Health| match h {
        Health::Down => 3,
        Health::Unreachable => 2,
        Health::Degraded => 1,
        Health::Online => 0,
    };
    if rank(sim.health[a]) >= rank(sim.health[b]) { sim.health[a] } else { sim.health[b] }
}

/// Is a live incident (not a warning, not held by a mitigation) on either end?
fn link_incident(sim: &Sim, l: usize) -> bool {
    let (a, b) = sim.links[l];
    [a, b].iter().any(|&n| sim.visible(n) && sim.faults[n].as_ref().is_some_and(|f| f.stage != Stage::Warning && f.mitigated <= 0.0))
}

/// Ropes and lines: visible at rest, the selection highlighted, and any link
/// touching a live incident bright red, thick and fully opaque.
fn links(game: Res<Game>, view: Res<ViewState>, mut q: Query<(&LinkLine, &mut Sprite)>) {
    for (line, mut sprite) in &mut q {
        let (a, b) = game.sim.links[line.link];
        let selected = view.selected.is_some_and(|s| s == a || s == b);
        let (color, width) = if link_incident(&game.sim, line.link) {
            (RED, 5.0)
        } else {
            match link_health(&game.sim, line.link) {
                Health::Down => (RED.with_alpha(0.9), 4.0),
                Health::Degraded => (AMBER.with_alpha(0.8), 3.0),
                Health::Unreachable => (SLATE.with_alpha(0.45), 2.0),
                Health::Online if selected => (if line.rope { ROPE.lighter(0.15) } else { GREEN_400 }.with_alpha(0.95), 3.5),
                Health::Online if line.rope => (ROPE.with_alpha(0.7), 3.0),
                Health::Online => (GREEN_400.with_alpha(0.35), 2.0),
            }
        };
        sprite.color = color;
        sprite.custom_size = Some(Vec2::new(line.len, width));
    }
}

/// Frames ride each link both ways. A flood races red toward its victim; a down
/// link stalls; unreachable links barely move; frames dim unless in focus.
fn frames(game: Res<Game>, view: Res<ViewState>, time: Res<Time>, curves: Res<LinkCurves>, mut q: Query<(&LinkFrame, &mut Transform, &mut Sprite)>) {
    let t = time.elapsed_secs();
    let sim = &game.sim;
    for (f, mut tf, mut sprite) in &mut q {
        let (a, b) = sim.links[f.link];
        let health = link_health(sim, f.link);
        let flood = [a, b].into_iter().find(|&n| sim.faults[n].as_ref().is_some_and(|x| x.kind == "ddos" && x.stage != Stage::Warning && x.mitigated <= 0.0));
        let pts = &curves.0[f.link];
        let len = path_len(pts).max(1.0);
        let focus = link_focus(&game, &view, f.link);
        let dim = if focus { 1.0 } else { 0.55 };
        let (speed, color) = match (flood, health) {
            (Some(_), _) => (300.0, RED.with_alpha(0.9)),
            (None, _) if link_incident(sim, f.link) => (20.0, RED),
            (None, Health::Down) => (0.0, RED.with_alpha(0.3 * dim)),
            (None, Health::Unreachable) => (8.0, SLATE.with_alpha(0.35 * dim)),
            (None, Health::Degraded) => (25.0, AMBER.with_alpha(0.8 * dim)),
            (None, Health::Online) => (60.0, Color::srgba(0.7, 1.0, 0.85, 0.75 * dim)),
        };
        let k = (t * speed / len + f.offset).fract();
        // Along the path from a to b (k) or back (1 - k); floods always run toward the victim.
        let forward = match flood {
            Some(n) => n == b,
            None => f.forward,
        };
        let margin = (42.0 / len).min(0.45);
        let k = margin + k * (1.0 - 2.0 * margin);
        let (at, d) = along(pts, if forward { k } else { 1.0 - k });
        let dir = if forward { d } else { -d };
        let lane = Vec2::new(dir.y, -dir.x) * if flood.is_some() { 0.0 } else { 4.0 };
        tf.translation = (at + lane).extend(NODE_Z - 1.0);
        tf.rotation = Quat::from_rotation_z(dir.to_angle());
        sprite.color = color;
    }
}

fn badge_text(sim: &Sim, i: usize) -> Option<(String, String, Color)> {
    if let Some(f) = &sim.faults[i] {
        if !sim.visible(i) {
            return None;
        }
        let d = sim.rules.fault(&f.kind)?;
        if f.is_warning() {
            let g = d.warning.as_ref().map_or("🔮", |w| w.emoji.as_str());
            return Some((g.into(), format!("⚠ {}s", f.ttl.ceil() as i32), AMBER));
        }
        if f.mitigated > 0.0 {
            return Some((d.emoji.clone(), format!("⏱ {}s", f.mitigated.ceil() as i32), AMBER));
        }
        if f.stage == Stage::Down {
            return Some((d.emoji.clone(), "DOWN".into(), RED_400));
        }
        return Some((d.emoji.clone(), format!("⏳{}s", f.ttl.ceil() as i32), RED_400));
    }
    (sim.health[i] == Health::Unreachable).then(|| ("🔌".to_string(), "no path".to_string(), SLATE))
}

fn cards(
    game: Res<Game>,
    time: Res<Time>,
    world: Res<World>,
    mut cards: Query<(&NodeCard, &mut Sprite)>,
    mut badges: Query<(&Badge, &mut Text2d, &mut Transform, &mut Visibility), Without<BadgeTtl>>,
    mut ttls: Query<(&BadgeTtl, &mut Text2d, &mut TextColor), Without<Badge>>,
) {
    let t = time.elapsed_secs();
    let pulse = 0.5 + 0.5 * (t * 3.0).sin();
    let sim = &game.sim;
    for (card, mut sprite) in &mut cards {
        let base = node_color(&world.nodes[card.0].kind);
        sprite.color = match sim.health[card.0] {
            Health::Online => base.with_alpha(0.9),
            Health::Degraded => Color::Srgba(base.to_srgba().mix(&AMBER.to_srgba(), 0.35 + 0.3 * pulse)).with_alpha(0.95),
            Health::Unreachable => Color::Srgba(base.to_srgba().mix(&SLATE.to_srgba(), 0.7)).with_alpha(0.6),
            Health::Down => Color::srgba(0.22, 0.08, 0.08, 0.95),
        };
    }
    for (badge, mut text, mut tf, mut vis) in &mut badges {
        match badge_text(sim, badge.0) {
            Some((glyph, _, _)) => {
                if text.0 != glyph {
                    text.0 = glyph;
                }
                tf.translation.y = 62.0 + (t * 3.0 + badge.0 as f32).sin() * 2.0;
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }
    for (ttl, mut text, mut color) in &mut ttls {
        if let Some((_, s, c)) = badge_text(sim, ttl.0) {
            if text.0 != s {
                text.0 = s;
            }
            color.0 = c;
        }
    }
}

fn worker_tokens(
    time: Res<Time>,
    game: Res<Game>,
    mut tokens: Query<(&WorkerToken, &mut Transform, &mut Visibility), Without<WorkBar>>,
    mut bars: Query<(&WorkBar, &mut Transform, &mut Visibility), Without<WorkerToken>>,
) {
    let t = time.elapsed_secs();
    let sim = &game.sim;
    for (tok, mut tf, mut vis) in &mut tokens {
        let w = &sim.workers[tok.0];
        *vis = if w.active { Visibility::Inherited } else { Visibility::Hidden };
        let flies = sim.rules.workers[w.def].flies;
        let hop = if !w.walking() {
            if w.resting() { 0.0 } else { ((t * 2.0 + tok.0 as f32).sin() * 1.5).max(0.0) }
        } else if w.overland {
            (t * 12.0).sin().abs() * 6.0
        } else if flies {
            40.0 + (t * 8.0).sin() * 4.0
        } else {
            let peak = (w.leg_len * 0.22).clamp(18.0, if w.swing { 240.0 } else { 90.0 });
            (std::f32::consts::PI * w.leg).sin() * peak
        };
        // Stand beside the card; each worker on its own side so they don't overlap.
        let side = [Vec2::new(36.0, 30.0), Vec2::new(-36.0, 34.0), Vec2::new(0.0, -46.0), Vec2::new(40.0, -30.0)][tok.0 % 4];
        tf.translation = (Vec2::from(w.pos) + side + Vec2::Y * hop).extend(NODE_Z + 30.0 + tok.0 as f32 * 0.1);
        let sleepy: f32 = if w.resting() { 0.6 } else { 1.0 };
        tf.scale = Vec3::splat(0.5 * sleepy.max(0.85));
    }
    for (bar, mut tf, mut vis) in &mut bars {
        let w = &sim.workers[bar.0];
        match w.queue.front().filter(|t| t.started && t.total > 0.0) {
            Some(task) => {
                let done = 1.0 - (task.left / task.total).clamp(0.0, 1.0);
                tf.scale.x = done.max(0.02);
                tf.translation.x = -30.0 * (1.0 - done);
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

fn popup(commands: &mut Commands, art: &Art, at: Vec2, text: String, color: Color, now: f32) {
    let w = text.chars().count() as f32 * 13.0 + 28.0;
    commands.spawn((
        Popup { born: now },
        Text2d::new(text),
        art.text(true, 22.0),
        TextColor(color),
        Transform::from_translation((at + Vec2::Y * 50.0).extend(NODE_Z + 50.0)).with_scale(Vec3::splat(0.5)),
    ))
    .with_child(plate(art, Vec2::new(w, 34.0), 0.85));
}

fn popups(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &Popup, &mut Transform, &mut TextColor)>) {
    let now = time.elapsed_secs();
    for (e, p, mut tf, mut color) in &mut q {
        let age = now - p.born;
        if age > 2.5 {
            commands.entity(e).despawn();
            continue;
        }
        tf.translation.y += 20.0 * time.delta_secs();
        color.0 = color.0.with_alpha((2.5 - age).min(1.0));
    }
}

// ------------------------------------------------------------------ simulation + input

fn tick(mut commands: Commands, time: Res<Time>, art: Res<Art>, mut game: ResMut<Game>) {
    game.sim.step(time.delta_secs().min(0.25));
    // A new outcome: a floating note on the map too.
    if let Some(o) = game.sim.outcome.clone() {
        if o.t != game.outcome_seen {
            game.outcome_seen = o.t;
            game.outcome_hidden = false;
            let (text, color) = if o.title.starts_with("MITIGATED") {
                ("⏱ temporary".to_string(), AMBER)
            } else if o.ok {
                ("✓ restored".to_string(), GREEN_400)
            } else {
                ("no effect".to_string(), RED_400)
            };
            let at = game.vec(o.node);
            popup(&mut commands, &art, at, text, color, time.elapsed_secs());
        }
    }
}

fn strikes(mut reader: MessageReader<Strike>, mut rng: ResMut<Rng>, mut game: ResMut<Game>) {
    for Strike(at) in reader.read() {
        let Some((i, d)) = (0..game.sim.nodes.len()).map(|i| (i, game.vec(i).distance(*at))).min_by(|a, b| a.1.total_cmp(&b.1)) else { continue };
        if d <= tuning().lightning_range && rng.f32() < tuning().lightning_chance {
            game.sim.lightning(i);
        }
    }
}

/// Screen direction of an arrow key.
const ARROWS: [(KeyCode, Vec2); 4] =
    [(KeyCode::ArrowUp, Vec2::Y), (KeyCode::ArrowDown, Vec2::NEG_Y), (KeyCode::ArrowLeft, Vec2::NEG_X), (KeyCode::ArrowRight, Vec2::X)];

fn island_of(world: &World, node: usize) -> Option<usize> {
    world.islands.iter().position(|i| i.id == world.nodes[node].island)
}

fn island_entry(world: &World, island: usize) -> Option<usize> {
    let id = world.islands[island].nodes.first()?;
    world.nodes.iter().position(|n| &n.id == id)
}

/// Open visible faults, most urgent first (down, then incidents by TTL, then warnings).
fn urgent_order(sim: &Sim) -> Vec<usize> {
    let mut open: Vec<usize> = (0..sim.nodes.len()).filter(|&i| sim.visible(i)).collect();
    let key = |i: usize| {
        let f = sim.faults[i].as_ref().unwrap();
        match f.stage {
            Stage::Down => -1.0,
            Stage::Incident if f.mitigated > 0.0 => 500.0 + f.mitigated,
            Stage::Incident => f.ttl,
            Stage::Warning => 1000.0 + f.ttl,
        }
    };
    open.sort_by(|&a, &b| key(a).total_cmp(&key(b)));
    open
}

/// Moving the selection: arrows hop to the nearest node in that direction
/// (links preferred), Q/E step between islands, Tab cycles open alerts.
fn navigate(keys: Res<ButtonInput<KeyCode>>, world: Res<World>, game: Res<Game>, mut view: ResMut<ViewState>) {
    if game.tour.is_some() {
        return;
    }
    let from = view.selected.unwrap_or(game.sim.workers.first().map_or(0, |w| w.at));
    let mut pick = None;
    for (k, dir) in ARROWS {
        if !keys.just_pressed(k) {
            continue;
        }
        pick = (0..game.sim.nodes.len())
            .filter(|&j| j != from)
            .filter_map(|j| {
                let v = game.vec(j) - game.vec(from);
                let cos = v.normalize_or_zero().dot(dir);
                (cos > 0.45).then(|| {
                    let linked = if game.sim.neighbours[from].contains(&j) { 0.6 } else { 1.0 };
                    (j, v.length() * (2.0 - cos) * linked)
                })
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(j, _)| j);
    }
    let islands = world.islands.len();
    if islands > 0 {
        let here = island_of(&world, from).unwrap_or(0);
        if keys.just_pressed(KeyCode::KeyE) {
            pick = island_entry(&world, (here + 1) % islands);
        } else if keys.just_pressed(KeyCode::KeyQ) {
            pick = island_entry(&world, (here + islands - 1) % islands);
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        let open = urgent_order(&game.sim);
        if !open.is_empty() {
            let next = view.selected.and_then(|s| open.iter().position(|&i| i == s)).map_or(0, |p| (p + 1) % open.len());
            pick = Some(open[next]);
        }
    }
    if let Some(j) = pick {
        view.selected = Some(j);
        view.focus = Some(game.vec(j));
    }
}

#[allow(clippy::too_many_arguments)]
fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut view: ResMut<ViewState>,
    start_button: Query<&Interaction, (Or<(With<StartButton>, With<SummaryButton>)>, Changed<Interaction>)>,
    go_button: Query<&Interaction, (With<GoButton>, Changed<Interaction>)>,
    swing_button: Query<&Interaction, (With<SwingButton>, Changed<Interaction>)>,
    diagnose_button: Query<&Interaction, (With<DiagnoseButton>, Changed<Interaction>)>,
    cancel_buttons: Query<(&Interaction, &CancelButton), Changed<Interaction>>,
    action_buttons: Query<(&Interaction, &ActionButton), Changed<Interaction>>,
    rows: Query<(&Interaction, &AlertRow), Changed<Interaction>>,
    roster_rows: Query<(&Interaction, &RosterRow), Changed<Interaction>>,
    ttl_box: Query<&Interaction, (With<TtlBox>, Changed<Interaction>)>,
    guide_button: Query<&Interaction, (With<GuideButton>, Changed<Interaction>)>,
    practice_button: Query<&Interaction, (With<PracticeButton>, Changed<Interaction>)>,
    tour_button: Query<&Interaction, (With<TourButton>, Changed<Interaction>)>,
) {
    // The network tour: outside shifts, step through every node.
    if let Some(i) = game.tour {
        let n = game.tour_order.len();
        let step = if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
            Some(1)
        } else if keys.just_pressed(KeyCode::ArrowLeft) {
            Some(n - 1)
        } else {
            None
        };
        if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyT) || tour_button.iter().any(|i| *i == Interaction::Pressed) {
            game.tour = None;
            return;
        }
        let i = step.map_or(i, |s| (i + s) % n);
        if game.tour != Some(i) || view.selected != Some(game.tour_order[i]) {
            game.tour = Some(i);
            let node = game.tour_order[i];
            view.selected = Some(node);
            view.focus = Some(game.vec(node));
        }
        return;
    }
    if keys.just_pressed(KeyCode::KeyB) || guide_button.iter().any(|i| *i == Interaction::Pressed) {
        game.guide_open = !game.guide_open;
    }
    // Worker choice: Z/X/C or a roster row (again to clear).
    let mut choose = None;
    for (n, k) in [KeyCode::KeyZ, KeyCode::KeyX, KeyCode::KeyC, KeyCode::KeyV].into_iter().enumerate() {
        if keys.just_pressed(k) && n < game.sim.workers.len() {
            choose = Some(n);
        }
    }
    for (i, r) in &roster_rows {
        if *i == Interaction::Pressed {
            choose = Some(r.0);
        }
    }
    if let Some(n) = choose {
        game.chosen = if game.chosen == Some(n) { None } else { Some(n) };
        let at = Vec2::from(game.sim.workers[n].pos);
        if game.chosen.is_some() {
            view.focus = Some(at);
        }
    }
    let running = game.sim.phase == Phase::Running;
    if !running {
        if keys.just_pressed(KeyCode::KeyT) || tour_button.iter().any(|i| *i == Interaction::Pressed) {
            let first = game.tour_order[0];
            game.tour = Some(0);
            view.selected = Some(first);
            view.focus = Some(game.vec(first));
            return;
        }
        let practice = keys.just_pressed(KeyCode::KeyP) || practice_button.iter().any(|i| *i == Interaction::Pressed);
        if practice || start_button.iter().any(|i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter) {
            let level = if practice { 0 } else { game.next_level };
            game.sim.start(level);
            game.chosen = None;
            game.outcome_hidden = true;
            game.recorded = false;
        }
        return;
    }
    if keys.just_pressed(KeyCode::KeyK) {
        let w = game.chosen.filter(|&w| !game.sim.workers[w].active).or_else(|| game.sim.workers.iter().position(|w| !w.active));
        match w {
            Some(w) => {
                if let Err(e) = game.sim.call_in(w) {
                    game.err(e);
                }
            }
            None => game.err("Everyone is already on shift"),
        }
    }
    for (i, row) in &rows {
        if *i == Interaction::Pressed {
            view.selected = Some(row.0);
            view.focus = Some(game.vec(row.0));
        }
    }
    if keys.just_pressed(KeyCode::Space) || ttl_box.iter().any(|i| *i == Interaction::Pressed) {
        let target = game.sim.deadline().map(|d| d.node).or_else(|| urgent_order(&game.sim).first().copied());
        if let Some(i) = target {
            view.selected = Some(i);
            view.focus = Some(game.vec(i));
        }
    }
    if keys.just_pressed(KeyCode::KeyF) {
        let w = game.chosen.unwrap_or(0);
        view.focus = Some(Vec2::from(game.sim.workers[w].pos));
    }
    if keys.just_pressed(KeyCode::KeyR) {
        let w = game.chosen.or_else(|| game.sim.workers.iter().position(|w| w.resting()));
        if let Some(w) = w {
            if let Err(e) = game.sim.rally(w) {
                game.err(format!("Rally: {e}"));
            }
        }
    }
    if keys.just_pressed(KeyCode::Backspace) {
        let w = game.chosen.or_else(|| game.sim.workers.iter().rposition(|w| w.queue.len() > 1)).unwrap_or(0);
        if !game.sim.cancel_last(w) {
            game.err("Nothing queued to cancel");
        }
    }
    for (i, c) in &cancel_buttons {
        if *i == Interaction::Pressed {
            game.sim.cancel_last(c.0);
        }
    }
    let Some(node) = view.selected else { return };
    let chosen = game.chosen;
    if keys.just_pressed(KeyCode::KeyJ) || swing_button.iter().any(|i| *i == Interaction::Pressed) {
        // Only those who don't fly swing; default to Mango.
        let w = chosen.filter(|&w| !game.sim.rules.workers[game.sim.workers[w].def].flies).unwrap_or(0);
        if let Err(e) = game.sim.swing(w, node) {
            game.err(format!("Swing: {e}"));
        }
    }
    if keys.just_pressed(KeyCode::KeyG) || keys.just_pressed(KeyCode::Enter) || go_button.iter().any(|i| *i == Interaction::Pressed) {
        if let Err(e) = game.sim.order(chosen, node, None) {
            game.err(e);
        }
    }
    if game.sim.faults[node].is_none() || !game.sim.visible(node) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyI) || diagnose_button.iter().any(|i| *i == Interaction::Pressed) {
        if let Err(e) = game.sim.order(chosen, node, Some("diagnose")) {
            game.err(e);
        }
    }
    let mut choice = None;
    for (n, k) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].into_iter().enumerate() {
        if keys.just_pressed(k) {
            choice = Some(n);
        }
    }
    for (i, b) in &action_buttons {
        if *i == Interaction::Pressed {
            choice = Some(b.0);
        }
    }
    let Some(choice) = choice else { return };
    let f = game.sim.faults[node].as_ref().unwrap();
    if f.tried & (1 << choice) != 0 || choice >= f.options.len() {
        return;
    }
    let action = f.options[choice].clone();
    if let Err(e) = game.sim.order(chosen, node, Some(&action)) {
        game.err(e);
    }
}

// ------------------------------------------------------------------ HUD

fn boxed() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(px(10), px(8)),
            row_gap: px(5),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BorderColor::all(GREEN_500.with_alpha(0.4)),
        BackgroundColor(PANEL_BG),
    )
}

fn spawn_hud(mut commands: Commands, art: Res<Art>, game: Res<Game>, windows: Query<&Window>) {
    let width = windows.iter().next().map_or(1280.0, |w| w.width());
    let narrow = width < 760.0;
    // Phones: the Time to Loss timer runs across the top, the column sits under it
    // without the log, and the node panel (ui.rs) keeps to the lower half.
    let phone = width < 600.0;
    // Left column: shift status, roster, open alerts, event feed.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(if phone { 100.0 } else { 12.0 }),
                left: px(12),
                right: if phone { px(12) } else { Val::Auto },
                width: if phone { Val::Auto } else { px(if narrow { 300.0 } else { 380.0 }) },
                max_width: percent(92),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            HudColumn { top: if phone { 100.0 } else { 12.0 } },
            GlobalZIndex(40),
        ))
        .with_children(|col| {
            col.spawn((Blocks, Interaction::default(), boxed())).with_children(|b| {
                b.spawn((HudText, Text::new("🐵 LUCKY MONKEY ON-CALL"), art.text(true, 13.0), TextColor(GREEN_400)));
                b.spawn((Node { width: percent(100), height: px(4), border_radius: BorderRadius::MAX, overflow: Overflow::clip(), ..default() }, BackgroundColor(GREEN_900.with_alpha(0.6))))
                    .with_child((HudBar, Node { width: percent(0), height: percent(100), ..default() }, BackgroundColor(GREEN_500)));
                b.spawn((HintText, Text::new(""), art.text(true, 12.0), TextColor(AMBER)));
                b.spawn((StartRow, Node { column_gap: px(6), row_gap: px(6), flex_wrap: FlexWrap::Wrap, ..default() })).with_children(|r| {
                    r.spawn((
                        StartButton,
                        Interaction::default(),
                        Node { padding: UiRect::axes(px(10), px(6)), flex_grow: 1.0, justify_content: JustifyContent::Center, border: UiRect::all(px(1)), border_radius: BorderRadius::MAX, ..default() },
                        BorderColor::all(GREEN_500.with_alpha(0.6)),
                        BackgroundColor(GREEN_900.with_alpha(0.5)),
                        children![(StartLabel, Text::new("START SHIFT (Enter)"), art.text(true, 12.0), TextColor(GREEN_400))],
                    ));
                    r.spawn((
                        GuideButton,
                        Interaction::default(),
                        Node { padding: UiRect::axes(px(10), px(6)), border: UiRect::all(px(1)), border_radius: BorderRadius::MAX, ..default() },
                        BorderColor::all(GREEN_500.with_alpha(0.6)),
                        BackgroundColor(GREEN_900.with_alpha(0.3)),
                        children![(Text::new("📖 Field guide (B)"), art.text(true, 12.0), TextColor(GREEN_300))],
                    ));
                    for (label, practice) in [("🧪 Practice (P)", true), ("🧭 Tour the network (T)", false)] {
                        let mut e = r.spawn((
                            Interaction::default(),
                            Node { padding: UiRect::axes(px(10), px(6)), border: UiRect::all(px(1)), border_radius: BorderRadius::MAX, ..default() },
                            BorderColor::all(Color::srgb(0.55, 0.8, 1.0).with_alpha(0.6)),
                            BackgroundColor(Color::srgba(0.0, 0.1, 0.2, 0.6)),
                            children![(Text::new(label), art.text(true, 12.0), TextColor(Color::srgb(0.7, 0.88, 1.0)))],
                        ));
                        if practice {
                            e.insert(PracticeButton);
                        } else {
                            e.insert(TourButton);
                        }
                    }
                });
            });
            // Roster.
            col.spawn((Blocks, Interaction::default(), boxed())).with_children(|b| {
                b.spawn((Text::new("CREW · Z/X/C/V picks who goes (again: auto) · K calls in help"), art.text(true, 10.0), TextColor(GREEN_400.with_alpha(0.8))));
                for w in 0..game.sim.workers.len() {
                    b.spawn((
                        RosterRow(w),
                        Interaction::default(),
                        Node { padding: UiRect::axes(px(6), px(3)), border: UiRect::all(px(1)), border_radius: BorderRadius::all(px(4)), ..default() },
                        BorderColor::all(Color::NONE),
                        BackgroundColor(Color::NONE),
                        children![(RosterText(w), Text::new(""), art.text(false, 12.0), TextColor(TEXT))],
                    ));
                }
            });
            col.spawn((AlertList, Blocks, Interaction::default(), Node { flex_direction: FlexDirection::Column, row_gap: px(3), ..default() }));
            let mut feed = col.spawn((Feed, Blocks, Interaction::default(), boxed()));
            if phone {
                feed.insert(Node { display: Display::None, ..default() });
            }
        });
    // Top centre: the Time to Loss timer.
    commands
        .spawn((
            // Wide: between the left column and the node panel. Narrow: under the left column's top box.
            if phone {
                Node { position_type: PositionType::Absolute, top: px(8), left: px(0), right: px(0), justify_content: JustifyContent::Center, ..default() }
            } else if narrow {
                Node { position_type: PositionType::Absolute, top: px(12), right: px(60), ..default() }
            } else {
                Node { position_type: PositionType::Absolute, top: px(12), left: px(404), ..default() }
            },
            GlobalZIndex(41),
        ))
        .with_children(|c| {
            c.spawn((
                TtlBox,
                Blocks,
                Interaction::default(),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(px(16), px(8)),
                    // Fits between the left column (404) and the node panel (right 400) on mid-width screens.
                    min_width: px(if narrow { 220.0 } else { (width - 812.0).clamp(180.0, 220.0) }),
                    max_width: px(if narrow { 330.0 } else { (width - 812.0).clamp(180.0, 330.0) }),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BorderColor::all(GREEN_500.with_alpha(0.5)),
                BackgroundColor(PANEL_BG),
            ))
            .with_children(|b| {
                b.spawn((TtlTitle, Text::new("✓ ALL CLEAR"), art.text(true, 12.0), TextColor(GREEN_300)));
                b.spawn((TtlTime, Text::new(""), art.text(true, 34.0), TextColor(GREEN_300)));
                b.spawn((TtlWhat, Text::new("No immediate threats"), art.text(false, 12.0), TextColor(TEXT), TextLayout::justify(Justify::Center)));
            });
        });
    // Outcome card: bottom centre.
    commands
        .spawn((Node { position_type: PositionType::Absolute, bottom: px(56), left: px(0), right: px(0), justify_content: JustifyContent::Center, ..default() }, GlobalZIndex(45)))
        .with_children(|c| {
            c.spawn((
                OutcomeCard,
                Blocks,
                Interaction::default(),
                Node { display: Display::None, max_width: px(460), padding: UiRect::axes(px(12), px(8)), border: UiRect::all(px(1)), border_radius: BorderRadius::all(px(8)), ..default() },
                BorderColor::all(GREEN_500),
                BackgroundColor(PANEL_BG),
                children![(OutcomeText, Text::new(""), art.text(false, 12.0), TextColor(TEXT))],
            ));
        });
    // Tour card.
    commands
        .spawn((Node { position_type: PositionType::Absolute, bottom: px(56), left: px(12), right: px(12), max_width: px(380), ..default() }, GlobalZIndex(46)))
        .with_children(|c| {
            c.spawn((
                TourCard,
                Blocks,
                Interaction::default(),
                Node { display: Display::None, padding: UiRect::axes(px(14), px(10)), border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(10)), ..default() },
                BorderColor::all(Color::srgb(0.55, 0.8, 1.0)),
                BackgroundColor(Color::srgb(0.0, 0.04, 0.08)),
                children![(TourText, Text::new(""), art.text(false, 13.0), TextColor(TEXT))],
            ));
        });
    // Field guide.
    commands.spawn((
        GuidePanel,
        Blocks,
        Interaction::default(),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: px(90),
            left: px(if narrow { 12.0 } else { 404.0 }),
            right: px(12),
            max_width: px(520),
            max_height: percent(75),
            overflow: Overflow::scroll_y(),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::all(px(14)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BorderColor::all(GREEN_500.with_alpha(0.6)),
        BackgroundColor(Color::srgb(0.0, 0.03, 0.01)),
        GlobalZIndex(60),
    ));
    // End-of-shift summary.
    commands
        .spawn((Node { position_type: PositionType::Absolute, top: px(0), bottom: px(0), left: px(0), right: px(0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, GlobalZIndex(70)))
        .with_children(|c| {
            c.spawn((
                SummaryPanel,
                Blocks,
                Interaction::default(),
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    max_width: px(640),
                    max_height: percent(92),
                    overflow: Overflow::scroll_y(),
                    margin: UiRect::all(px(12)),
                    padding: UiRect::all(px(18)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(12)),
                    ..default()
                },
                BorderColor::all(GREEN_500),
                BackgroundColor(Color::srgb(0.0, 0.02, 0.01)),
            ))
            .with_children(|p| {
                p.spawn((SummaryText, Text::new(""), art.text(false, 13.0), TextColor(TEXT)));
                p.spawn((
                    SummaryButton,
                    Interaction::default(),
                    Node { padding: UiRect::axes(px(12), px(8)), justify_content: JustifyContent::Center, border: UiRect::all(px(1)), border_radius: BorderRadius::MAX, ..default() },
                    BorderColor::all(GREEN_500),
                    BackgroundColor(GREEN_900.with_alpha(0.6)),
                    children![(Text::new("NEXT SHIFT (Enter)"), art.text(true, 13.0), TextColor(GREEN_400))],
                ));
                p.spawn(Node { column_gap: px(8), ..default() }).with_children(|r| {
                    r.spawn((
                        PracticeButton,
                        Interaction::default(),
                        Node { flex_grow: 1.0, padding: UiRect::axes(px(12), px(8)), justify_content: JustifyContent::Center, border: UiRect::all(px(1)), border_radius: BorderRadius::MAX, ..default() },
                        BorderColor::all(Color::srgb(0.55, 0.8, 1.0)),
                        BackgroundColor(Color::srgba(0.0, 0.1, 0.2, 0.6)),
                        children![(Text::new("🧪 PRACTICE (P)"), art.text(true, 12.0), TextColor(Color::srgb(0.7, 0.88, 1.0)))],
                    ));
                    r.spawn((
                        TourButton,
                        Interaction::default(),
                        Node { flex_grow: 1.0, padding: UiRect::axes(px(12), px(8)), justify_content: JustifyContent::Center, border: UiRect::all(px(1)), border_radius: BorderRadius::MAX, ..default() },
                        BorderColor::all(Color::srgb(0.55, 0.8, 1.0)),
                        BackgroundColor(Color::srgba(0.0, 0.1, 0.2, 0.6)),
                        children![(Text::new("🧭 TOUR (T)"), art.text(true, 12.0), TextColor(Color::srgb(0.7, 0.88, 1.0)))],
                    ));
                });
            });
        });
}

fn tour_card(game: Res<Game>, world: Res<World>, mut card: Single<&mut Node, With<TourCard>>, text: Single<Entity, With<TourText>>, mut texts: Query<&mut Text>) {
    let Some(i) = game.tour else {
        card.display = Display::None;
        return;
    };
    card.display = Display::Flex;
    let sim = &game.sim;
    let node = game.tour_order[i];
    let n = &world.nodes[node];
    let isl = world.islands.iter().find(|x| x.id == n.island).map_or(String::new(), |x| format!("{} {}", x.emoji, x.name));
    let role = sim.rules.roles.get(&n.kind).cloned().unwrap_or_default();
    let links: Vec<&str> = sim.neighbours[node].iter().map(|&j| sim.nodes[j].name.as_str()).collect();
    let impact = if node == sim.core {
        "It IS the core router: if it goes down, every other service loses its path.".to_string()
    } else {
        match sim.impact_if_down(node) {
            0 => "If it goes down, nothing else is cut off: other paths route around it.".to_string(),
            k => format!("If it goes down, {k} other service(s) lose their path to the core router."),
        }
    };
    let s = format!(
        "🧭 NETWORK TOUR {}/{} · {isl}\n{} {} ({})\n{role}\nLinks to: {}\n{impact}\n\n→ / Space: next   ← back   Esc: end the tour",
        i + 1,
        game.tour_order.len(),
        n.emoji,
        n.name,
        n.kind,
        links.join(", ")
    );
    set(&mut texts, *text, s);
}

fn set(texts: &mut Query<&mut Text>, e: Entity, s: String) {
    if let Ok(mut t) = texts.get_mut(e) {
        if t.0 != s {
            t.0 = s;
        }
    }
}

/// One line telling the player what to do now.
fn next_step(game: &Game, view: &ViewState) -> String {
    let sim = &game.sim;
    if let Some((msg, t)) = &game.notice {
        if sim.clock - t < 4.0 {
            return format!("⚠ {msg}");
        }
    }
    match sim.phase {
        Phase::Idle if game.tour.is_some() => return "🧭 Touring: → next, ← back, Esc to stop. Nothing breaks during the tour.".into(),
        Phase::Idle => return "▶ Enter: training shift · P: practice, no pressure · T: tour the network first".into(),
        Phase::Done | Phase::Collapsed if game.tour.is_some() => return "🧭 Touring: → next, ← back, Esc to stop.".into(),
        Phase::Done | Phase::Collapsed => return "▶ Shift over: read the summary, then Enter for the next one (P: practice, T: tour).".into(),
        Phase::Running => {}
    }
    if let Some(c) = sim.coach() {
        return format!("🎓 {c}");
    }
    if let Some(sel) = view.selected.filter(|&s| sim.visible(s)) {
        let f = sim.faults[sel].as_ref().unwrap();
        if sim.workers.iter().any(|w| w.queue.iter().any(|t| t.node == sel && t.fault == Some(f.id))) {
            return "▶ Someone is on it. Tab: next alert".into();
        }
        if !f.diagnosed && !f.is_warning() {
            return "▶ Pick a response 1-4, or I to diagnose first (shows which one fixes the cause)".into();
        }
        return if f.is_warning() { "▶ Prevent it: pick the matching action 1-4 before Time to Loss runs out".into() } else { "▶ Pick a response 1-4".into() };
    }
    match urgent_order(sim).first() {
        Some(&i) => format!("▶ {} needs attention: Space jumps there", sim.nodes[i].name),
        None => "▶ All quiet. Enjoy the jungle; arrows hop nodes, B opens the field guide.".into(),
    }
}

fn hud(
    game: Res<Game>,
    view: Res<ViewState>,
    mut texts: Query<&mut Text>,
    hud_text: Single<Entity, With<HudText>>,
    hint_text: Single<Entity, With<HintText>>,
    start_label: Single<Entity, With<StartLabel>>,
    mut bar: Single<(&mut Node, &mut BackgroundColor), With<HudBar>>,
    mut start_node: Single<&mut Node, (With<StartRow>, Without<HudBar>)>,
) {
    let sim = &game.sim;
    let line = match sim.phase {
        Phase::Idle => "🐵 LUCKY MONKEY ON-CALL\nKeep the jungle network running until 17:00.".to_string(),
        _ => format!(
            "🕘 SHIFT {} → 17:00   ★ {}{}   🍌 {}\n💚 NETWORK HEALTH {:.0}%   📈 SHIFT SLA {:.1}% (target {:.0}%)",
            clock_str(sim.clock, sim.rules.shift_secs),
            sim.score,
            if sim.combo > 1 { format!(" 🔥×{}", sim.combo) } else { String::new() },
            sim.bananas,
            sim.health_now,
            sim.sla,
            sim.rules.sla_target,
        ),
    };
    set(&mut texts, *hud_text, line);
    set(&mut texts, *hint_text, next_step(&game, &view));
    start_node.display = if sim.phase == Phase::Running { Display::None } else { Display::Flex };
    let label = match sim.phase {
        Phase::Idle => format!("START {} (Enter)", sim.rules.director.levels[game.next_level.min(sim.rules.director.levels.len() - 1)].name.to_uppercase()),
        _ => "NEXT SHIFT (Enter)".into(),
    };
    set(&mut texts, *start_label, label);
    let (node, bg) = &mut *bar;
    node.width = percent(sim.progress() * 100.0);
    bg.0 = if sim.health_now < 50.0 { RED } else { GREEN_500 };
}

/// The big timer: the next consequence, calm when there is none. Red for a live
/// incident, amber for a warning or a mitigation wearing off. No blinking:
/// urgency shows as colour, a label and the number itself.
fn ttl_box(
    game: Res<Game>,
    windows: Query<&Window>,
    mut texts: Query<&mut Text>,
    mut colors: Query<&mut TextColor>,
    mut fonts: Query<&mut TextFont>,
    title: Single<Entity, With<TtlTitle>>,
    time_e: Single<Entity, With<TtlTime>>,
    what: Single<Entity, With<TtlWhat>>,
    mut boxq: Single<(&mut BorderColor, &mut Node, &mut BackgroundColor), With<TtlBox>>,
) {
    let sim = &game.sim;
    let phone = windows.iter().next().is_some_and(|w| w.width() < 600.0);
    let (border, node, bg) = &mut *boxq;
    node.display = if sim.phase == Phase::Running && game.tour.is_none() { Display::Flex } else { Display::None };
    let (t, n, w, c, back, big) = match sim.deadline() {
        Some(d) => {
            let f = sim.faults[d.node].as_ref();
            let warning = f.is_some_and(|f| f.is_warning() || f.mitigated > 0.0);
            let (c, back) = if warning { (AMBER, Color::srgb(0.14, 0.09, 0.0)) } else { (RED, Color::srgb(0.17, 0.02, 0.02)) };
            let label = if warning { "⏳ TIME TO LOSS · WARNING" } else if d.secs <= 10.0 { "⏳ TIME TO LOSS · NOW" } else { "⏳ TIME TO LOSS" };
            let more = urgent_order(sim).len().saturating_sub(1);
            let more = if more > 0 { format!("\n+{more} more open · Tab cycles") } else { String::new() };
            (label.to_string(), mmss(d.secs), format!("Next impact: {}\n{}{more}", sim.nodes[d.node].name, d.what), c, back, if phone { 34.0 } else { 56.0 })
        }
        None => ("✓ ALL CLEAR".to_string(), String::new(), "No immediate threats".to_string(), GREEN_300, Color::BLACK, 18.0),
    };
    set(&mut texts, *title, t);
    set(&mut texts, *time_e, n);
    set(&mut texts, *what, w);
    for e in [*title, *time_e] {
        if let Ok(mut col) = colors.get_mut(e) {
            col.0 = c;
        }
    }
    if let Ok(mut f) = fonts.get_mut(*time_e) {
        if f.font_size != FontSize::Px(big) {
            f.font_size = FontSize::Px(big);
        }
    }
    **border = BorderColor::all(c);
    node.border = UiRect::all(px(if c == GREEN_300 { 2.0 } else { 3.0 }));
    bg.0 = back;
}

fn worker_line(sim: &Sim, w: usize) -> String {
    let wk = &sim.workers[w];
    let d = &sim.rules.workers[wk.def];
    if !wk.active {
        return format!("{} {:<5} off shift · K: call in ({} 🍌)", wk.emoji, wk.name.to_uppercase(), sim.rules.call_cost);
    }
    let stamina: String = (0..d.stamina).map(|i| if i < wk.stamina { '●' } else { '○' }).collect();
    let state = if wk.resting() {
        format!("💤 resting {:.0}s (R: 🍌 rally)", wk.rest)
    } else if let Some(t) = wk.queue.front() {
        let place = &sim.nodes[t.node].name;
        let what = match t.action.as_deref() {
            None => "going".to_string(),
            Some("diagnose") => "🔍 diagnose".to_string(),
            Some(a) => sim.rules.action(a).map_or(a.to_string(), |x| format!("{} {}", x.emoji, x.label)),
        };
        if t.started {
            format!("🔧 {what} · {place} {:.0}s", t.left.max(0.0))
        } else {
            format!("→ {place} {:.0}s, then {what}", sim.eta(w, t.node).max(0.0))
        }
    } else if wk.walking() {
        "→ moving".into()
    } else {
        format!("✓ free at {}", sim.nodes[wk.at].name)
    };
    let queued = if wk.queue.len() > 1 { format!("  +{} queued", wk.queue.len() - 1) } else { String::new() };
    format!("{} {:<5} {stamina}  {state}{queued}", wk.emoji, wk.name.to_uppercase())
}

fn roster(game: Res<Game>, mut texts: Query<(&RosterText, &mut Text, &mut TextColor)>, mut rows: Query<(&RosterRow, &mut BorderColor, &mut BackgroundColor)>) {
    for (r, mut text, mut color) in &mut texts {
        let s = worker_line(&game.sim, r.0);
        if text.0 != s {
            text.0 = s;
        }
        color.0 = if game.sim.workers[r.0].active { TEXT } else { Color::srgb(0.55, 0.6, 0.58) };
    }
    for (r, mut border, mut bg) in &mut rows {
        let on = game.chosen == Some(r.0);
        *border = BorderColor::all(if on { AMBER } else { Color::NONE });
        bg.0 = if on { Color::srgba(0.3, 0.22, 0.0, 0.5) } else { Color::NONE };
    }
}

fn stage_tag(sim: &Sim, i: usize) -> (&'static str, Color) {
    let f = sim.faults[i].as_ref().unwrap();
    match f.stage {
        Stage::Warning => ("WARN", AMBER),
        _ if f.mitigated > 0.0 => ("HELD", AMBER),
        Stage::Incident => ("INC", RED_400),
        Stage::Down => ("DOWN", RED_500),
    }
}

/// Open alerts, most urgent first, clickable. Rebuilt when the sim changes.
fn alert_list(mut commands: Commands, art: Res<Art>, game: Res<Game>, mut seen: ResMut<Seen>, list: Single<(Entity, Option<&Children>), With<AlertList>>) {
    let sim = &game.sim;
    if seen.alerts == Some(sim.version) {
        return;
    }
    seen.alerts = Some(sim.version);
    let (root, children) = *list;
    if let Some(children) = children {
        for c in children.iter() {
            commands.entity(c).despawn();
        }
    }
    if sim.phase != Phase::Running {
        return;
    }
    let open = urgent_order(sim);
    commands.entity(root).with_children(|list| {
        if !sim.sighted() {
            list.spawn((
                Node { padding: UiRect::axes(px(8), px(3)), border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(PANEL_BG),
                children![(Text::new("🔭 Watchtower unreachable: warnings are hidden"), art.text(true, 11.0), TextColor(AMBER))],
            ));
        }
        let cut = sim.unreachable();
        if cut > 0 {
            list.spawn((
                Node { padding: UiRect::axes(px(8), px(3)), border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(PANEL_BG),
                children![(Text::new(format!("🔌 {cut} unreachable (impact, not a fault: fix the down node)")), art.text(false, 11.0), TextColor(SLATE))],
            ));
        }
        for &i in open.iter().take(5) {
            let f = sim.faults[i].as_ref().unwrap();
            let Some(d) = sim.rules.fault(&f.kind) else { continue };
            let (tag, color) = stage_tag(sim, i);
            let glyph = if f.is_warning() { d.warning.as_ref().map_or("🔮", |w| w.emoji.as_str()) } else { d.emoji.as_str() };
            let who: String = sim.workers.iter().filter(|w| w.queue.iter().any(|t| t.node == i)).map(|w| w.emoji.as_str()).collect();
            list.spawn((
                AlertRow(i),
                Interaction::default(),
                Node { column_gap: px(6), align_items: AlignItems::Center, padding: UiRect::axes(px(8), px(3)), border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(PANEL_BG),
            ))
            .with_children(|r| {
                r.spawn((Text::new(tag), art.text(true, 11.0), TextColor(color), Node { min_width: px(34), ..default() }));
                r.spawn((PanelTtl(i), Text::new(""), art.text(true, 11.0), TextColor(color), Node { min_width: px(36), ..default() }));
                r.spawn((Text::new(glyph), TextFont { font: art.emoji.clone().into(), font_size: FontSize::Px(12.0), ..default() }));
                r.spawn((Text::new(format!("{} · {}{}", sim.nodes[i].name, if f.is_warning() { "warning" } else { d.name.as_str() }, if who.is_empty() { String::new() } else { format!("  ← {who}") })), art.text(false, 11.0), TextColor(TEXT)));
            });
        }
        if open.len() > 5 {
            list.spawn((Text::new(format!("  +{} more (Tab cycles)", open.len() - 5)), art.text(false, 11.0), TextColor(GREEN_300)));
        }
    });
}

fn cat_color(c: Cat) -> Color {
    match c {
        Cat::Critical => RED_500,
        Cat::Incident | Cat::Cascade => RED_400,
        Cat::Warning | Cat::Mitigated => AMBER,
        Cat::NoEffect => Color::srgb(0.95, 0.6, 0.45),
        Cat::Resolved | Cat::Recover => GREEN_400,
        Cat::Learn => Color::srgb(0.55, 0.8, 1.0),
        Cat::Notice | Cat::Rest => Color::srgb(0.6, 0.7, 0.65),
        _ => GREEN_300,
    }
}

/// The shared stream: incidents and worker activity together, urgent lines kept in view.
fn feed(mut commands: Commands, art: Res<Art>, game: Res<Game>, mut seen: ResMut<Seen>, root: Single<(Entity, Option<&Children>), With<Feed>>, rows: Query<(&Interaction, &FeedRow), Changed<Interaction>>, mut view: ResMut<ViewState>) {
    for (i, r) in &rows {
        if *i == Interaction::Pressed {
            view.selected = Some(r.0);
            view.focus = Some(game.vec(r.0));
        }
    }
    let sim = &game.sim;
    if seen.feed == Some(sim.version) {
        return;
    }
    seen.feed = Some(sim.version);
    let (root, children) = *root;
    if let Some(children) = children {
        for c in children.iter() {
            commands.entity(c).despawn();
        }
    }
    commands.entity(root).with_children(|b| {
        b.spawn((Text::new("NETWORK LOG"), art.text(true, 10.0), TextColor(GREEN_400.with_alpha(0.8))));
        for e in sim::feed(&sim.log, sim.clock, 8, 40.0) {
            let count = if e.count > 1 { format!(" ×{}", e.count) } else { String::new() };
            let line = format!("{} {} {} {}{count}", clock_str(e.t, sim.rules.shift_secs), e.icon, e.cat.tag(), e.text);
            let mut row = b.spawn((Text::new(line), art.text(false, 11.0), TextColor(cat_color(e.cat))));
            if let Some(n) = e.node {
                row.insert((FeedRow(n), Interaction::default()));
            }
        }
    });
}

/// Countdown texts in the alert list and node panel.
fn live_ttls(game: Res<Game>, mut q: Query<(&PanelTtl, &mut Text)>) {
    for (p, mut text) in &mut q {
        let s = game.sim.faults[p.0].as_ref().map_or(String::new(), |f| {
            if f.mitigated > 0.0 {
                format!("{:.0}s", f.mitigated)
            } else if f.stage == Stage::Down {
                "—".into()
            } else {
                format!("{:.0}s", f.ttl.max(0.0))
            }
        });
        if text.0 != s {
            text.0 = s;
        }
    }
}

fn outcome_card(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut card: Single<(&mut Node, &mut BorderColor, &Interaction), With<OutcomeCard>>,
    text: Single<Entity, With<OutcomeText>>,
    mut texts: Query<&mut Text>,
) {
    let (node, border, interaction) = &mut *card;
    if **interaction == Interaction::Pressed || keys.just_pressed(KeyCode::Escape) {
        game.outcome_hidden = true;
    }
    let Some(o) = game.sim.outcome.clone().filter(|o| !game.outcome_hidden && game.tour.is_none() && game.sim.clock - o.t < 9.0 && game.sim.phase == Phase::Running) else {
        node.display = Display::None;
        return;
    };
    node.display = Display::Flex;
    **border = BorderColor::all(if o.title.starts_with("MITIGATED") { AMBER } else if o.ok { GREEN_500 } else { RED_400 });
    let mut s = format!("{}\nCause: {}\nAction: {}", o.title, o.cause, o.action);
    if let Some(l) = &o.learned {
        s += &format!("\nLearned: {l}");
    }
    s += "\n(click or Esc to dismiss)";
    set(&mut texts, *text, s);
}

fn guide(mut commands: Commands, art: Res<Art>, game: Res<Game>, mut seen: ResMut<Seen>, panel: Single<(Entity, &mut Node, Option<&Children>), With<GuidePanel>>) {
    let (root, mut node, children) = panel.into_inner();
    node.display = if game.guide_open { Display::Flex } else { Display::None };
    let key = (game.sim.learned.len(), game.guide_open);
    if seen.guide == Some(key) {
        return;
    }
    seen.guide = Some(key);
    if let Some(children) = children {
        for c in children.iter() {
            commands.entity(c).despawn();
        }
    }
    let sim = &game.sim;
    commands.entity(root).with_children(|p| {
        p.spawn((Text::new("📖 JUNGLE FIELD GUIDE  (B to close)"), art.text(true, 14.0), TextColor(GREEN_400)));
        p.spawn((Text::new("Real networking terms, learned on the job. The jungle is fantasy; the terminology is real."), art.text(false, 11.0), TextColor(GREEN_300)));
        let mut entries: Vec<(&str, String)> = vec![("timeloss", "always in effect".into())];
        entries.extend(sim.learned.iter().map(|(k, ex)| (k.as_str(), ex.clone())));
        for (k, example) in entries {
            if let Some(c) = sim.rules.concepts.get(k) {
                p.spawn((Text::new(format!("■ {}", c.name)), art.text(true, 12.0), TextColor(Color::srgb(0.55, 0.8, 1.0))));
                p.spawn((Text::new(format!("{}\nSeen: {example}", c.text)), art.text(false, 11.0), TextColor(TEXT)));
            }
        }
        let left = sim.rules.concepts.len().saturating_sub(sim.learned.len() + 1);
        p.spawn((Text::new(format!("{left} more to discover: resolve or diagnose incidents to learn them.")), art.text(false, 11.0), TextColor(GREEN_300.with_alpha(0.7))));
        p.spawn((Text::new("Workers"), art.text(true, 12.0), TextColor(GREEN_400)));
        for d in &sim.rules.workers {
            p.spawn((Text::new(format!("{} {} the {}: {}", d.emoji, d.name, d.species, d.role)), art.text(false, 11.0), TextColor(TEXT)));
        }
    });
}

fn summary(mut game: ResMut<Game>, mut panel: Single<&mut Node, With<SummaryPanel>>, text: Single<Entity, With<SummaryText>>, mut texts: Query<&mut Text>) {
    let over = matches!(game.sim.phase, Phase::Done | Phase::Collapsed);
    panel.display = if over && game.tour.is_none() { Display::Flex } else { Display::None };
    if !over {
        return;
    }
    // Record the shift once, in this browser's score history.
    if !game.recorded {
        game.recorded = true;
        let sim = &game.sim;
        let entry = Played { name: sim.level_def().name.clone(), score: sim.score, sla: sim.sla, met: sim.phase == Phase::Done && sim.sla >= sim.rules.sla_target };
        game.history.push(entry);
        let keep = game.history.len().saturating_sub(20);
        game.history.drain(0..keep);
        save_history(&game.history);
        let levels = game.sim.rules.director.levels.len();
        // Practice doesn't move you along; a completed shift does.
        let met = game.sim.phase == Phase::Done && game.sim.sla >= game.sim.rules.sla_target;
        if game.sim.level > 0 && met {
            game.next_level = (game.sim.level + 1).min(levels - 1);
        }
    }
    let sim = &game.sim;
    let title = if sim.phase == Phase::Done && sim.sla >= sim.rules.sla_target {
        format!("🌅 SHIFT COMPLETE · SLA met · {}", sim.level_def().name)
    } else if sim.phase == Phase::Done {
        format!("🌇 SHIFT OVER · SLA target missed ({:.1}% of {:.0}%) · {}", sim.sla, sim.rules.sla_target, sim.level_def().name)
    } else {
        format!("💥 GAME OVER · major outage at {} · {}", clock_str(sim.clock, sim.rules.shift_secs), sim.level_def().name)
    };
    let st = &sim.stats;
    let mut s = format!(
        "{title}\n\n★ SCORE {}  =  fixes {}  +  prevention {}  +  combo bonus {}\n",
        sim.score, st.points_fix, st.points_prevent, st.points_combo
    );
    for l in sim.summary() {
        s += &format!("{l}\n");
    }
    s += "\n✅ WHAT WENT RIGHT\n";
    if sim.notes_good.is_empty() {
        s += "  • Nothing fixed or prevented yet: Space jumps to an alert, I diagnoses.\n";
    }
    for n in sim.notes_good.iter().take(6) {
        s += &format!("  • {n}\n");
    }
    s += "\n⚠ WHAT WENT WRONG\n";
    if sim.notes_bad.is_empty() {
        s += "  • Nothing. Clean shift.\n";
    }
    for n in sim.notes_bad.iter().take(6) {
        s += &format!("  • {n}\n");
    }
    s += "\n🏆 YOUR SHIFTS (this browser)\n";
    let best = game.history.iter().map(|p| p.score).max().unwrap_or(0);
    for (k, p) in game.history.iter().enumerate().rev().take(6) {
        let mark = if p.score == best && best > 0 { " ← best" } else { "" };
        s += &format!("  #{:<2} {:<22} ★ {:>5}  SLA {:>5.1}% {}{mark}\n", k + 1, p.name, p.score, p.sla, if p.met { "✓" } else { "✗" });
    }
    let levels = sim.rules.director.levels.len();
    s += &format!("\nNext: {} (levels advance when the SLA target is met)", sim.rules.director.levels[game.next_level.min(levels - 1)].name);
    set(&mut texts, *text, s);
}

// ------------------------------------------------------------------ node panel (called from ui.rs)

fn button(art: &Art, marker: impl Bundle, label: String, fg: Color, bg: Color) -> impl Bundle {
    (
        marker,
        Interaction::default(),
        Node {
            align_items: AlignItems::Center,
            column_gap: px(8),
            padding: UiRect::axes(px(8), px(7)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        BorderColor::all(fg.with_alpha(0.5)),
        BackgroundColor(bg),
        children![(Text::new(label), art.text(false, 12.0), TextColor(fg))],
    )
}

/// The urgent part of the node panel: incident, time to loss, who's on it, responses.
pub fn alert_box(body: &mut ChildSpawnerCommands, art: &Art, game: &Game, node: usize) {
    let sim = &game.sim;
    if sim.phase != Phase::Running {
        return;
    }
    let crew: Vec<usize> = (0..sim.workers.len()).filter(|&w| sim.workers[w].queue.iter().any(|t| t.node == node)).collect();
    let Some(f) = sim.faults[node].as_ref().filter(|_| sim.visible(node)) else {
        if sim.health[node] == Health::Unreachable {
            body.spawn((
                Text::new("🔌 UNREACHABLE: this node is fine, but its path to the core router runs through a down node. Fix that node and this one recovers by itself."),
                art.text(false, 12.0),
                TextColor(SLATE),
                Node { margin: UiRect::bottom(px(8)), ..default() },
            ));
        }
        row_buttons(body, art, sim, &crew);
        return;
    };
    let Some(d) = sim.rules.fault(&f.kind) else { return };
    let warning = f.is_warning();
    let (accent, bg) = if warning || f.mitigated > 0.0 { (AMBER, Color::srgba(0.3, 0.2, 0.0, 0.5)) } else { (RED_400, Color::srgba(0.3, 0.04, 0.04, 0.5)) };
    let (glyph, title, text) = if warning {
        let w = d.warning.as_ref().unwrap();
        (w.emoji.as_str(), format!("WARNING · {}", d.name), w.text.clone())
    } else {
        let stage = match f.stage {
            Stage::Down => "DOWN",
            _ if f.mitigated > 0.0 => "MITIGATED",
            _ => "INCIDENT",
        };
        (d.emoji.as_str(), format!("{stage} · {}", d.name), d.symptom.clone())
    };
    let who = if crew.is_empty() {
        "Nobody assigned yet".to_string()
    } else {
        crew.iter().map(|&w| format!("{} {} (ETA {:.0}s)", sim.workers[w].emoji, sim.workers[w].name, sim.eta(w, node))).collect::<Vec<_>>().join(", ")
    };
    let worker = game.chosen.or_else(|| sim.pick_worker(node, None)).unwrap_or(0);
    body.spawn((
        Node { flex_direction: FlexDirection::Column, padding: UiRect::all(px(8)), margin: UiRect::bottom(px(8)), row_gap: px(6), border: UiRect::all(px(1)), border_radius: BorderRadius::all(px(6)), ..default() },
        BorderColor::all(accent.with_alpha(0.6)),
        BackgroundColor(bg),
    ))
    .with_children(|b| {
        b.spawn(Node { align_items: AlignItems::Center, column_gap: px(6), flex_wrap: FlexWrap::Wrap, ..default() }).with_children(|r| {
            r.spawn((Text::new(glyph), TextFont { font: art.emoji.clone().into(), font_size: FontSize::Px(16.0), ..default() }));
            r.spawn((Text::new(title), art.text(true, 12.0), TextColor(accent)));
        });
        b.spawn(Node { align_items: AlignItems::Center, column_gap: px(6), ..default() }).with_children(|r| {
            let label = if f.mitigated > 0.0 { "⏱ relief ends in" } else if f.stage == Stage::Down { "⛔ down: same fix, 1.5× slower" } else { "⏳ time to loss" };
            r.spawn((Text::new(label), art.text(true, 12.0), TextColor(accent)));
            r.spawn((PanelTtl(node), Text::new(""), art.text(true, 14.0), TextColor(accent)));
        });
        b.spawn((Text::new(text), art.text(false, 12.0), TextColor(Color::srgb(1.0, 0.92, 0.86))));
        b.spawn((Text::new(format!("👥 {who}")), art.text(false, 11.0), TextColor(GREEN_300)));
        let chooser = if game.chosen.is_some() { "" } else { " (auto)" };
        b.spawn((Text::new(format!("Responses for {} {}{chooser}:", sim.workers[worker].emoji, sim.workers[worker].name)), art.text(true, 10.0), TextColor(GREEN_400)));
        for (i, action) in f.options.iter().enumerate() {
            let tried = f.tried & (1 << i) != 0;
            let a = sim.rules.action(action);
            let (emoji, label) = a.map_or(("🔧", action.as_str()), |a| (a.emoji.as_str(), a.label.as_str()));
            let able = sim.can_do(worker, action);
            let secs = sim.work_secs(worker, action, node);
            // After a diagnosis (or a failed try), say what each option does.
            let tag = if f.diagnosed || tried {
                let fixes = sim.fixes_for(node);
                if fixes.contains(action) {
                    "  ✓ fixes the cause".to_string()
                } else {
                    match sim.response(node, action) {
                        Some(r) if r.effect == "mitigate" && !warning => format!("  ⏱ temporary ({:.0}s)", r.secs),
                        _ => "  ✗ won't help".to_string(),
                    }
                }
            } else {
                String::new()
            };
            let fg = if tried || !able { Color::srgb(0.5, 0.5, 0.5) } else { GREEN_300 };
            let heavy = if able { String::new() } else { " · too heavy for this worker".to_string() };
            b.spawn((
                ActionButton(i),
                Interaction::default(),
                Node { align_items: AlignItems::Center, column_gap: px(8), padding: UiRect::axes(px(8), px(7)), border: UiRect::all(px(1)), border_radius: BorderRadius::all(px(4)), ..default() },
                BorderColor::all(if tried { Color::srgba(0.4, 0.4, 0.4, 0.4) } else { GREEN_500.with_alpha(0.5) }),
                BackgroundColor(if tried { Color::NONE } else { GREEN_900.with_alpha(0.4) }),
            ))
            .with_children(|r| {
                r.spawn((Text::new(format!("{}", i + 1)), art.text(true, 12.0), TextColor(fg)));
                r.spawn((Text::new(emoji), TextFont { font: art.emoji.clone().into(), font_size: FontSize::Px(14.0), ..default() }));
                r.spawn((Text::new(format!("{label} · {secs:.0}s{tag}{heavy}")), art.text(false, 12.0), TextColor(fg)));
            });
        }
        if !f.diagnosed && !warning {
            let secs = sim.work_secs(worker, "diagnose", node);
            b.spawn(button(art, DiagnoseButton, format!("🔍 Diagnose first (I) · {secs:.0}s: shows what each option does"), Color::srgb(0.55, 0.8, 1.0), Color::srgba(0.0, 0.1, 0.2, 0.5)));
        }
    });
    row_buttons(body, art, sim, &crew);
}

fn row_buttons(body: &mut ChildSpawnerCommands, art: &Art, sim: &Sim, crew: &[usize]) {
    body.spawn(Node { column_gap: px(6), flex_wrap: FlexWrap::Wrap, row_gap: px(6), margin: UiRect::bottom(px(8)), ..default() }).with_children(|r| {
        r.spawn(button(art, GoButton, "🐾 Send here (G)".into(), GREEN_300, GREEN_900.with_alpha(0.4)));
        let can = sim.bananas >= sim.rules.swing_cost;
        r.spawn(button(art, SwingButton, format!("🌿 Swing (J) 🍌{}", sim.rules.swing_cost), if can { AMBER } else { Color::srgb(0.45, 0.45, 0.45) }, Color::srgba(0.25, 0.2, 0.0, 0.4)));
        for &w in crew {
            r.spawn(button(art, CancelButton(w), format!("✖ cancel {}'s last job", sim.workers[w].name), RED_400, Color::NONE));
        }
    });
}

// ------------------------------------------------------------------ edge pins

const PIN: f32 = 38.0;
const PIN_MARGIN: f32 = 10.0;

fn spawn_pins(mut commands: Commands, art: Res<Art>, world: Res<World>) {
    for (i, node) in world.nodes.iter().enumerate() {
        commands.spawn((
            EdgePin(i),
            Blocks,
            Interaction::default(),
            Node {
                position_type: PositionType::Absolute,
                width: px(PIN),
                height: px(PIN),
                display: Display::None,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BorderColor::all(GREEN_400),
            BackgroundColor(node_color(&node.kind).with_alpha(0.92)),
            GlobalZIndex(30),
            children![(Text::new(node.emoji.clone()), TextFont { font: art.emoji.clone().into(), font_size: FontSize::Px(20.0), ..default() })],
        ));
    }
}

/// Nodes out of view turn into their icon, stuck to the screen edge in the
/// direction they lie; back in view, the pin hides and the card is the node again.
fn pins(
    game: Res<Game>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    column: Query<(&HudColumn, &ComputedNode)>,
    mut q: Query<(&EdgePin, &mut Node, &mut BorderColor)>,
) {
    let (cam, cam_tf) = *camera;
    let size = window.size();
    let (lo, hi) = (Vec2::splat(PIN_MARGIN + PIN / 2.0), size - Vec2::new(PIN_MARGIN + PIN / 2.0, 40.0 + PIN / 2.0));
    let centre = (lo + hi) / 2.0;
    let half = (hi - lo) / 2.0;
    // Where each off-screen node meets the edge, grouped by edge (0 left, 1 right, 2 top, 3 bottom).
    let mut edges: [Vec<(usize, f32)>; 4] = Default::default();
    for (pin, _, _) in &q {
        let Ok(p) = cam.world_to_viewport(cam_tf, game.vec(pin.0).extend(0.0)) else { continue };
        if p.x > -20.0 && p.y > -20.0 && p.x < size.x + 20.0 && p.y < size.y + 20.0 {
            continue;
        }
        let d = p - centre;
        let t = (half.x / d.x.abs().max(1e-3)).min(half.y / d.y.abs().max(1e-3));
        let e = centre + d * t;
        if (e.x - lo.x).abs() < 0.5 {
            edges[0].push((pin.0, e.y));
        } else if (e.x - hi.x).abs() < 0.5 {
            edges[1].push((pin.0, e.y));
        } else if (e.y - lo.y).abs() < 0.5 {
            edges[2].push((pin.0, e.x));
        } else {
            edges[3].push((pin.0, e.x));
        }
    }
    // Pins on one edge never stack: push them apart along it, then back inside.
    let gap = PIN + 4.0;
    // Pins under the HUD column couldn't be clicked: the left edge starts below it, the top edge right of it.
    let (col_right, col_bottom) = column.iter().next().map_or((0.0, 0.0), |(c, n)| {
        let sz = n.size() * n.inverse_scale_factor();
        (12.0 + sz.x, c.top + sz.y)
    });
    let mut at: Vec<Option<Vec2>> = vec![None; game.sim.health.len()];
    for (k, list) in edges.iter_mut().enumerate() {
        let (min, max) = match k {
            0 => ((col_bottom + 8.0 + PIN / 2.0).max(lo.y), hi.y),
            1 => (lo.y, hi.y),
            2 => ((col_right + 8.0 + PIN / 2.0).max(lo.x), hi.x),
            _ => (lo.x, hi.x),
        };
        let min = min.min(max);
        list.sort_by(|a, b| a.1.total_cmp(&b.1));
        for l in list.iter_mut() {
            l.1 = l.1.max(min);
        }
        for j in 1..list.len() {
            list[j].1 = list[j].1.max(list[j - 1].1 + gap);
        }
        if let Some(last) = list.last().map(|l| l.1) {
            let over = (last - max).max(0.0);
            for l in list.iter_mut() {
                l.1 = (l.1 - over).max(min);
            }
        }
        for &(i, v) in list.iter() {
            let c = match k {
                0 => Vec2::new(lo.x, v),
                1 => Vec2::new(hi.x, v),
                2 => Vec2::new(v, lo.y),
                _ => Vec2::new(v, hi.y),
            };
            at[i] = Some(c - Vec2::splat(PIN / 2.0));
        }
    }
    for (pin, mut node, mut border) in &mut q {
        let Some(p) = at.get(pin.0).copied().flatten() else {
            node.display = Display::None;
            continue;
        };
        node.display = Display::Flex;
        node.left = px(p.x);
        node.top = px(p.y);
        let sim = &game.sim;
        let c = match sim.health[pin.0] {
            Health::Down => RED,
            Health::Degraded => RED_400,
            Health::Unreachable => SLATE,
            Health::Online if sim.visible(pin.0) => AMBER,
            Health::Online => GREEN_400.with_alpha(0.7),
        };
        *border = BorderColor::all(c);
    }
}

/// Phones: the node panel takes the lower half and the column would show through
/// it, so the column steps aside while a node is open (Time to Loss stays on top).
fn phone_column(view: Res<ViewState>, window: Single<&Window>, mut q: Query<&mut Node, With<HudColumn>>) {
    let hide = window.width() < 600.0 && view.selected.is_some();
    for mut node in &mut q {
        let d = if hide { Display::None } else { Display::Flex };
        if node.display != d {
            node.display = d;
        }
    }
}

fn pin_clicks(game: Res<Game>, mut view: ResMut<ViewState>, q: Query<(&Interaction, &EdgePin), Changed<Interaction>>) {
    for (i, pin) in &q {
        if *i == Interaction::Pressed {
            view.selected = Some(pin.0);
            view.focus = Some(game.vec(pin.0));
        }
    }
}

// ------------------------------------------------------------------ deep links

/// A deep link (see link.rs): fly to a node, open its panel, maybe stage trouble.
fn apply_link(link: Res<crate::link::DeepLink>, mut game: ResMut<Game>, mut view: ResMut<ViewState>, world: Res<World>) {
    let node = link.node.as_deref().and_then(|id| world.nodes.iter().position(|n| n.id == id));
    view.zoom_to = link.zoom;
    if let Some(i) = node {
        view.selected = Some(i);
        view.focus = Some(game.vec(i));
    }
    let key = |k: &str| match k {
        "lightning" | "surge" => "power".to_string(),
        other => other.to_string(),
    };
    let incident = link.incident.as_deref().map(key).filter(|k| game.sim.rules.faults.contains_key(k));
    let warning = link.forecast.as_deref().map(key).filter(|k| game.sim.rules.faults.get(k).is_some_and(|f| f.warning.is_some()));
    if !(link.shift || incident.is_some() || warning.is_some()) {
        return;
    }
    // Staged scenes start a regular shift (no training script); #shift starts training.
    let level = if incident.is_some() || warning.is_some() { 2 } else { 1 };
    game.sim.start(level);
    if let Some(i) = node {
        if let Some(k) = incident {
            game.sim.raise(i, &k, false, None);
        } else if let Some(k) = warning {
            game.sim.raise(i, &k, true, None);
        }
    }
}
