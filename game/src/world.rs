//! The seeded world: terrain mesh, decorations, floating edge islands, nodes.
//!
//! Terrain is one mesh per iso diagonal row (x + y), with z = row, so anything
//! standing on row k (decorations, NPCs at z = k + 0.5) is correctly hidden by
//! taller terrain in front of it.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use serde::Deserialize;

use crate::Art;

/// Used by data.rs to check a fetched world.json parses before trusting it.
pub(crate) type WorldFileCheck = WorldFile;

pub const TILE_W: f32 = 64.0;
pub const TILE_H: f32 = 32.0;
/// Screen pixels per unit of height.
pub const ELEV: f32 = 16.0;
/// Every island has a slab this thick (in height units) under its lowest tiles;
/// the first unit is the tile sprite's own side skirt.
const BASE: f32 = 1.0;
pub const NODE_Z: f32 = 200.0;
const PYRAMID: (i32, i32) = (17, 17);
const PYRAMID_HEIGHT: f32 = 5.0;

pub mod kind {
    pub const GRASS: u8 = 0;
    pub const WATER: u8 = 1;
    /// Flowing lava (the water sprites, inverted and hue-shifted).
    pub const LAVA: u8 = 2;
    pub const PLATFORM: u8 = 4;
    pub const SCORCHED: u8 = 5;
    pub const VOLCANO: u8 = 6;
    pub const TEMPLE: u8 = 7;
}

#[derive(Deserialize)]
pub(crate) struct WorldFile {
    tiles: Vec<TileIn>,
    nodes: Vec<NodeData>,
    decorations: Vec<Deco>,
    #[serde(default)]
    streams: Vec<Vec<Pos>>,
    /// Built by tools/gen-world.py: final terrain, skip the legacy clean-ups.
    #[serde(default)]
    baked: bool,
    #[serde(default)]
    islands: Vec<Island>,
    /// Faint far-off islands: [x, y, radius].
    #[serde(default)]
    backdrop: Vec<[i32; 3]>,
    /// Lava stream centrelines (Ember Isle).
    #[serde(default)]
    lava_streams: Vec<Vec<Pos>>,
    /// The story of the jungle (tour, docs).
    #[serde(default)]
    legend: String,
}

/// A named island of the archipelago (navigation, labels).
#[derive(Deserialize, Clone)]
pub struct Island {
    pub id: String,
    pub name: String,
    pub emoji: String,
    pub center: Pos,
    #[serde(default)]
    pub nodes: Vec<String>,
    #[serde(default)]
    pub lore: String,
}

#[derive(Deserialize)]
struct TileIn {
    x: f32,
    y: f32,
    #[serde(rename = "type")]
    kind: u8,
    height: f32,
    #[serde(default)]
    fall: bool,
    #[serde(default)]
    lagoon: bool,
    #[serde(default)]
    spill: u8,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub height: f32,
}

#[derive(Deserialize, Clone)]
pub struct NodeData {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "ipAddress")]
    pub ip: String,
    pub emoji: String,
    #[serde(default)]
    pub description: String,
    pub position: Pos,
    #[serde(default)]
    pub connections: Vec<String>,
    #[allow(dead_code)]
    pub status: Option<String>,
    #[serde(rename = "securityLevel")]
    pub security: Option<u8>,
    #[serde(default)]
    pub assets: Vec<String>,
    #[serde(default)]
    pub island: String,
    #[serde(default)]
    pub lore: String,
}

#[derive(Deserialize)]
struct Deco {
    #[serde(rename = "type")]
    kind: String,
    position: Pos,
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub kind: u8,
    pub height: f32,
    /// Part of the waterfall at the river's source.
    pub fall: bool,
    /// Part of the logical lagoon (bubbles).
    pub lagoon: bool,
    /// Pours off the island's edge into the void: 1 over its +x edge, 2 over +y.
    pub spill: u8,
}

impl Tile {
    fn ground(kind: u8, height: f32) -> Tile {
        Tile { kind, height, fall: false, lagoon: false, spill: 0 }
    }
}

#[derive(Resource)]
pub struct World {
    pub tiles: HashMap<(i32, i32), Tile>,
    pub nodes: Vec<NodeData>,
    /// River centrelines, upstream first (the main river, the Digital River).
    pub streams: Vec<Vec<Pos>>,
    pub islands: Vec<Island>,
    /// Screen-space bounds of the whole map (Bevy coordinates, y up).
    pub bounds: Rect,
    /// Where the camera may go: the islands and their nodes, plus a margin.
    /// The river runs on beyond it in both directions, out of reach.
    pub pan: Rect,
    pub lava_streams: Vec<Vec<Pos>>,
    pub legend: String,
}

impl World {
    pub fn tile_at(&self, x: f32, y: f32) -> Option<Tile> {
        self.tiles.get(&(x.round() as i32, y.round() as i32)).copied()
    }
    pub fn ground(&self, x: f32, y: f32) -> f32 {
        self.tile_at(x, y).map_or(0.0, |t| t.height)
    }
}

/// Grid position -> Bevy world position of the top surface (y up).
pub fn iso(x: f32, y: f32, h: f32) -> Vec2 {
    Vec2::new((x - y) * TILE_W / 2.0, -((x + y) * TILE_H / 2.0) + h * ELEV)
}

/// Draw depth for something standing on grid cell (x, y).
pub fn row_z(x: f32, y: f32) -> f32 {
    x.round() + y.round() + 0.5
}

/// Stable pseudo-random number in [0,1) for a grid cell (same as the old TS cellHash).
pub fn cell_hash(x: i32, y: i32) -> f32 {
    let mut h = x.wrapping_mul(374_761_393).wrapping_add(y.wrapping_mul(668_265_263));
    h = (h ^ ((h as u32) >> 13) as i32).wrapping_mul(1_274_126_177);
    ((h ^ ((h as u32) >> 16) as i32) as u32) as f32 / 4_294_967_296.0
}

#[derive(Component)]
pub struct NodeCard(pub usize);

const EDGE_ISLANDS: [(i32, i32, i32); 10] = [
    (-5, 5, 4), (-8, 10, 3), (45, 5, 4), (48, 12, 3), (-6, 35, 4),
    (-3, 42, 3), (45, 38, 5), (50, 32, 3), (-12, 20, 3), (55, 22, 3),
];

pub fn plugin(app: &mut App) {
    let file: WorldFile = serde_json::from_str(crate::data::world_json()).expect("world.json");
    // Several generators write the same cell; keep the tallest so stacked
    // platforms read as solid ground instead of whichever was written last.
    let mut tiles: HashMap<(i32, i32), Tile> = HashMap::new();
    for t in &file.tiles {
        let key = (t.x.round() as i32, t.y.round() as i32);
        let new = Tile { kind: t.kind, height: t.height.max(0.0), fall: t.fall, lagoon: t.lagoon, spill: t.spill };
        tiles
            .entry(key)
            .and_modify(|old| {
                let (fall, lagoon) = (old.fall || new.fall, old.lagoon || new.lagoon);
                if new.height > old.height {
                    *old = new;
                }
                if old.kind == kind::WATER {
                    (old.fall, old.lagoon) = (fall, lagoon);
                }
            })
            .or_insert(new);
    }
    let mut bank_decos = Vec::new();
    if !file.baked {
    // The temple pyramid generator stacks its layers upside down (widest layer
    // highest). Rebuild it as a stepped pyramid: the more layers cover a cell,
    // the higher it is, so the centre (where the Guardian sits) is the summit.
    let mut layers: HashMap<(i32, i32), f32> = HashMap::new();
    for t in file.tiles.iter().filter(|t| t.kind == kind::TEMPLE) {
        let key = (t.x.round() as i32, t.y.round() as i32);
        if (key.0 - PYRAMID.0).abs() + (key.1 - PYRAMID.1).abs() <= 8 {
            let e = layers.entry(key).or_insert(f32::MAX);
            *e = e.min(t.height);
        }
    }
    for (key, lowest) in layers {
        tiles.insert(key, Tile::ground(kind::TEMPLE, (PYRAMID_HEIGHT + 1.0 - lowest).max(1.0)));
    }
    // The river was generated on its own and floated in the void. Give it banks:
    // a full ring of grass, plus a ragged second ring so the edge isn't a ruler line.
    let mut banks: HashMap<(i32, i32), f32> = HashMap::new();
    for (&(x, y), t) in tiles.iter().filter(|(_, t)| t.kind == kind::WATER) {
        for dx in -2..=2i32 {
            for dy in -2..=2i32 {
                let k = (x + dx, y + dy);
                let ring = dx.abs().max(dy.abs());
                if ring == 0 || tiles.contains_key(&k) || (ring == 2 && cell_hash(k.0 * 3 + 1, k.1 * 5 + 2) > 0.45) {
                    continue;
                }
                let h = banks.entry(k).or_insert(0.0);
                *h = h.max(t.height);
            }
        }
    }
    for (&k, &h) in &banks {
        tiles.insert(k, Tile::ground(kind::GRASS, h));
        let r = cell_hash(k.0 * 7 + 3, k.1 * 11 + 5);
        if r < 0.22 {
            let kind = ["bush", "tree", "flower", "bamboo", "bush", "rock"][((r / 0.22) * 6.0) as usize % 6];
            bank_decos.push(Deco { kind: kind.into(), position: Pos { x: k.0 as f32, y: k.1 as f32, height: h } });
        }
    }
    }
    let mut bounds = Rect::EMPTY;
    for (&(x, y), t) in &tiles {
        let p = iso(x as f32, y as f32, t.height);
        bounds = bounds.union_point(p + Vec2::new(-TILE_W, TILE_H)).union_point(p + Vec2::new(TILE_W, -TILE_H - BASE * ELEV));
    }
    let backdrop = if file.backdrop.is_empty() { EDGE_ISLANDS.to_vec() } else { file.backdrop.iter().map(|b| (b[0], b[1], b[2])).collect() };
    app.insert_resource(Backdrop(backdrop));
    let mut pan = Rect::EMPTY;
    for n in &file.nodes {
        pan = pan.union_point(iso(n.position.x, n.position.y, n.position.height));
    }
    let pan = if pan.is_empty() { bounds } else { pan.inflate(420.0) };
    app.insert_resource(World {
        tiles,
        nodes: file.nodes,
        streams: file.streams,
        islands: file.islands,
        bounds,
        pan,
        lava_streams: file.lava_streams,
        legend: file.legend,
    });
    let mut decorations = file.decorations;
    decorations.extend(bank_decos);
    app.insert_resource(Decos(decorations));
    app.add_systems(Startup, (spawn_terrain, spawn_islands, spawn_backdrop, spawn_decorations, spawn_nodes, spawn_island_labels));
}

#[derive(Resource)]
struct Decos(Vec<Deco>);

#[derive(Resource)]
struct Backdrop(Vec<(i32, i32, i32)>);

/// Each island's name floats over its far (top) edge, faint, like a map label.
fn spawn_island_labels(mut commands: Commands, world: Res<World>, art: Res<Art>) {
    for isl in &world.islands {
        // The highest-on-screen tile within reach of the centre marks the far edge.
        let (cx, cy) = (isl.center.x as i32, isl.center.y as i32);
        let top = world
            .tiles
            .iter()
            .filter(|((x, y), _)| (x - cx).abs() <= 9 && (y - cy).abs() <= 9)
            .map(|(&(x, y), t)| iso(x as f32, y as f32, t.height).y)
            .fold(f32::MIN, f32::max);
        let at = Vec2::new(iso(isl.center.x, isl.center.y, 0.0).x, top + 46.0);
        commands.spawn((
            Text2d::new(format!("{} {}", isl.emoji, isl.name.to_uppercase())),
            art.text(true, 36.0),
            TextColor(Color::srgba(0.85, 1.0, 0.9, 0.95)),
            Transform::from_translation(at.extend(NODE_Z - 5.0)).with_scale(Vec3::splat(0.5)),
        ))
        .with_child((
            Sprite {
                image: art.pill.clone(),
                color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                custom_size: Some(Vec2::new(isl.name.chars().count() as f32 * 23.0 + 80.0, 56.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.01),
        ));
    }
}

/// Atlas cell (4x4 grid of 34px cells, 32px payload) for a tile.
fn atlas_cell(t: Tile, x: i32, y: i32) -> usize {
    let pick = |first: usize, n: usize| first + ((cell_hash(x, y) * n as f32) as usize).min(n - 1);
    match t.kind {
        kind::WATER => pick(4, 5),
        kind::LAVA => pick(16, 5),
        kind::VOLCANO => pick(9, 5),
        kind::TEMPLE => 14,
        _ => pick(0, 4),
    }
}
const WHITE_CELL: usize = 15;

fn cell_uv(cell: usize) -> (Vec2, Vec2) {
    // tiles.png: 4 columns x 6 rows of 34 px cells (make-textures.py).
    let (cx, cy) = ((cell % 4) as f32, (cell / 4) as f32);
    let (px, py) = (1.0 / 136.0, 1.0 / 204.0);
    (Vec2::new((cx * 34.0 + 1.0) * px, (cy * 34.0 + 1.0) * py), Vec2::new((cx * 34.0 + 33.0) * px, (cy * 34.0 + 33.0) * py))
}

fn wall_color(kind: u8) -> Color {
    match kind {
        kind::WATER => Color::srgb(0.16, 0.36, 0.62),
        kind::LAVA => Color::srgb(0.62, 0.20, 0.05),
        kind::VOLCANO => Color::srgb(0.48, 0.16, 0.10),
        kind::TEMPLE => Color::srgb(0.72, 0.56, 0.16),
        kind::SCORCHED => Color::srgb(0.30, 0.22, 0.17),
        _ => Color::srgb(0.30, 0.40, 0.18),
    }
}

fn top_tint(t: Tile) -> Color {
    let lift = 1.0 + t.height * 0.025;
    let base = match t.kind {
        kind::SCORCHED => Vec3::new(0.78, 0.62, 0.50),
        kind::PLATFORM => Vec3::new(0.90, 1.0, 0.94),
        _ => Vec3::ONE,
    } * lift;
    Color::srgb(base.x.min(1.0), base.y.min(1.0), base.z.min(1.0))
}

#[derive(Default)]
struct MeshBuf {
    pos: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    col: Vec<[f32; 4]>,
    idx: Vec<u32>,
}

impl MeshBuf {
    fn quad(&mut self, v: [Vec2; 4], uv: [Vec2; 4], c: Color) {
        self.quad4(v, uv, [c; 4]);
    }
    /// A quad with its own colour at each corner (gradients).
    fn quad4(&mut self, v: [Vec2; 4], uv: [Vec2; 4], c: [Color; 4]) {
        let base = self.pos.len() as u32;
        for i in 0..4 {
            self.pos.push([v[i].x, v[i].y, 0.0]);
            self.uv.push(uv[i].into());
            self.col.push(c[i].to_linear().to_f32_array());
        }
        self.idx.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    fn mesh(self) -> Mesh {
        let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.pos);
        m.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uv);
        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.col);
        m.insert_indices(Indices::U32(self.idx));
        m
    }
}

fn spawn_terrain(
    mut commands: Commands,
    world: Res<World>,
    art: Res<Art>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material = materials.add(ColorMaterial::from(art.tiles.clone()));
    let mut rows: HashMap<i32, Vec<(i32, i32, Tile)>> = HashMap::new();
    for (&(x, y), &t) in &world.tiles {
        rows.entry(x + y).or_default().push((x, y, t));
    }
    let (w0, w1) = cell_uv(WHITE_CELL);
    let white = [(w0 + w1) / 2.0; 4];
    let nodes: Vec<(i32, i32)> = world.nodes.iter().map(|n| (n.position.x.round() as i32, n.position.y.round() as i32)).collect();
    let near_node = |x: i32, y: i32| nodes.iter().any(|&(nx, ny)| (nx - x).abs() + (ny - y).abs() <= 2);
    for (row, cells) in rows {
        let mut buf = MeshBuf::default();
        for (x, y, t) in cells {
            let c = iso(x as f32, y as f32, t.height);
            let (e, s, w) = (c + Vec2::X * 32.0, c - Vec2::Y * 16.0, c - Vec2::X * 32.0);
            // Front walls drop to the neighbour in front, or to the bottom of the island slab.
            let drop = |nx: i32, ny: i32| match world.tiles.get(&(nx, ny)) {
                Some(o) => (t.height - o.height).max(0.0) * ELEV,
                None => (t.height + BASE) * ELEV,
            };
            let wall = wall_color(t.kind);
            // Walls are lit at the lip and fall into shadow toward the bottom; deep
            // drops (island edges) fade almost to black so islands float.
            let shade = |c: Color, depth: f32| c.darker((0.12 + depth / (ELEV * 6.0)).min(0.42));
            let dl = drop(x, y + 1);
            if dl > 0.0 {
                let d = Vec2::Y * dl;
                let top = wall.darker(0.16);
                buf.quad4([w, s, s - d, w - d], white, [top, top, shade(top, dl), shade(top, dl)]);
            }
            let dr = drop(x + 1, y);
            if dr > 0.0 {
                let d = Vec2::Y * dr;
                let top = wall.lighter(0.03);
                buf.quad4([s, e, e - d, s - d], white, [top, top, shade(top, dr), shade(top, dr)]);
            }
            if !near_node(x, y) {
                vines(&mut buf, t, x, y, (w, s), dl, world.tiles.contains_key(&(x, y + 1)), 0);
                vines(&mut buf, t, x, y, (s, e), dr, world.tiles.contains_key(&(x + 1, y)), 1);
            }
            // The sprites are 32px iso blocks: top diamond at rows 8..24, an 8px side
            // skirt below. Drawn at 2x, the diamond lands exactly on the grid cell.
            let (a, b) = cell_uv(atlas_cell(t, x, y));
            let (tl, br) = (c + Vec2::new(-32.0, 32.0), c + Vec2::new(32.0, -32.0));
            // Each top gets a little brightness variation, and a soft contact shadow
            // on its back edge when a taller block stands behind it.
            let jitter = 1.0 + (cell_hash(x * 31 + 7, y * 17 + 3) - 0.5) * 0.08;
            let taller = |nx: i32, ny: i32| world.tiles.get(&(nx, ny)).is_some_and(|o| o.height > t.height);
            let (back_l, back_r) = (taller(x - 1, y), taller(x, y - 1));
            let base = top_tint(t).to_srgba();
            let tint = |k: f32| Color::srgba((base.red * k).min(1.0), (base.green * k).min(1.0), (base.blue * k).min(1.0), base.alpha);
            let lit = tint(jitter);
            let dim = tint(jitter * 0.72);
            // Corners: top-left, top-right, bottom-right, bottom-left of the 2x quad.
            let corners = [
                if back_l || back_r { dim } else { lit },
                if back_r { dim } else { lit },
                lit,
                if back_l { dim } else { lit },
            ];
            buf.quad4(
                [tl, Vec2::new(br.x, tl.y), br, Vec2::new(tl.x, br.y)],
                [a, Vec2::new(b.x, a.y), b, Vec2::new(a.x, b.y)],
                corners,
            );
        }
        commands.spawn((
            Mesh2d(meshes.add(buf.mesh())),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(0.0, 0.0, row as f32),
        ));
    }
}

const VINE_STEM: Color = Color::srgb(0.15, 0.36, 0.11);
const VINE_LEAF: Color = Color::srgb(0.29, 0.56, 0.19);
const VINE_LEAF2: Color = Color::srgb(0.43, 0.70, 0.25);

/// Little pixel vines hanging from exposed island edges: a 2px stem that jogs
/// sideways now and then, small leaves on alternating sides, a curl at the tip.
/// Placement is hashed from the cell, sparse and clustered (lush cells only),
/// and the quads go into the row's terrain mesh: no extra entities, no cost per frame.
fn vines(buf: &mut MeshBuf, t: Tile, x: i32, y: i32, edge: (Vec2, Vec2), depth: f32, covered: bool, face: i32) {
    if covered || depth < ELEV || !matches!(t.kind, kind::GRASS | kind::PLATFORM | kind::TEMPLE) {
        return;
    }
    // Clusters: a coarse hash picks lush stretches, a fine one picks cells within them.
    let lush = cell_hash(x.div_euclid(3) * 5 + face, y.div_euclid(3) * 9 + 1) < 0.5;
    let pick = cell_hash(x * 13 + 5 + face * 7, y * 7 + 11);
    if !lush || pick > 0.5 {
        return;
    }
    let white = [{
        let (a, b) = cell_uv(WHITE_CELL);
        (a + b) / 2.0
    }; 4];
    let snap = |v: Vec2| Vec2::new((v.x / 2.0).round() * 2.0, (v.y / 2.0).round() * 2.0);
    let rect = |buf: &mut MeshBuf, at: Vec2, w: f32, h: f32, c: Color| {
        let at = snap(at);
        buf.quad([at, at + Vec2::X * w, at + Vec2::new(w, -h), at - Vec2::Y * h], white, c);
    };
    let strands = 1 + (pick * 7.0) as i32 % 2;
    for k in 0..strands {
        let r = cell_hash(x * 3 + k * 17 + face, y * 5 + k * 29);
        let f = 0.18 + 0.64 * r;
        let mut at = snap(edge.0.lerp(edge.1, f));
        let len = (depth * (0.5 + 0.5 * r) + 6.0 + 22.0 * cell_hash(x + k * 31, y - face * 13)).min(depth + 26.0);
        let mut y_off = 0.0;
        let mut seg = 0;
        while y_off < len {
            let h = 6.0f32.min(len - y_off);
            rect(buf, at - Vec2::Y * y_off, 2.0, h, VINE_STEM);
            if seg % 2 == 0 && y_off + 4.0 < len {
                let side = if (seg / 2 + k) % 2 == 0 { -4.0 } else { 2.0 };
                let leaf = if cell_hash(x * 7 + seg, y * 3 + k) < 0.5 { VINE_LEAF } else { VINE_LEAF2 };
                rect(buf, at + Vec2::new(side, -(y_off + 2.0)), 4.0, 2.0, leaf);
            }
            y_off += 6.0;
            seg += 1;
            // Jog sideways now and then.
            if cell_hash(x * 11 + seg * 3 + k, y * 13 - seg) < 0.3 {
                at.x += if face == 0 { -2.0 } else { 2.0 };
            }
        }
        // Curl at the tip.
        let dir = if k % 2 == 0 { 2.0 } else { -2.0 };
        rect(buf, at + Vec2::new(dir, -len), 2.0, 2.0, VINE_STEM);
        rect(buf, at + Vec2::new(dir * 2.0, -(len - 2.0)), 2.0, 2.0, VINE_LEAF);
    }
}

fn spawn_islands(mut commands: Commands, backdrop: Res<Backdrop>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mut buf = MeshBuf::default();
    for &(cx, cy, r) in &backdrop.0 {
        for dx in -r..=r {
            for dy in -r..=r {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > r as f32 {
                    continue;
                }
                let alpha = 0.15 - dist / r as f32 * 0.1;
                let c = iso((cx + dx) as f32, (cy + dy) as f32, 0.0);
                let v = [c + Vec2::Y * 15.0, c + Vec2::X * 30.0, c - Vec2::Y * 15.0, c - Vec2::X * 30.0];
                buf.quad(v, [Vec2::ZERO; 4], Color::srgba(0.0, 1.0, 0.8, alpha * 1.6));
            }
        }
    }
    commands.spawn((
        Mesh2d(meshes.add(buf.mesh())),
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        Transform::from_xyz(0.0, 0.0, -50.0),
    ));
}

fn spawn_backdrop(mut commands: Commands, world: Res<World>, art: Res<Art>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    // The void: a large, faint grid whose hue drifts from jungle green through teal
    // and blue to violet across the map, with slow waves so it never reads as bands.
    // One mesh with vertex colours over a repeating texture.
    let area = Rect::from_center_size(world.bounds.center(), world.bounds.size() + Vec2::splat(4000.0));
    const N: usize = 28;
    const TILE: f32 = 512.0 * 3.2;
    let mut buf = MeshBuf { pos: vec![], uv: vec![], col: vec![], idx: vec![] };
    let hue = |p: Vec2| {
        let t = ((p - area.min) / area.size()).clamp(Vec2::ZERO, Vec2::ONE);
        let k = (t.x * 0.55 + (1.0 - t.y) * 0.45 + 0.08 * (t.x * 9.0).sin() * (t.y * 7.0).cos()).clamp(0.0, 1.0);
        Color::hsla(140.0 + k * 150.0, 0.85, 0.66, 0.62)
    };
    for i in 0..N {
        for j in 0..N {
            let a = area.min + area.size() * Vec2::new(i as f32, j as f32) / N as f32;
            let b = area.min + area.size() * Vec2::new((i + 1) as f32, (j + 1) as f32) / N as f32;
            let v = [Vec2::new(a.x, a.y), Vec2::new(b.x, a.y), Vec2::new(b.x, b.y), Vec2::new(a.x, b.y)];
            let uv = v.map(|p| Vec2::new(p.x / TILE, -p.y / TILE));
            buf.quad4(v, uv, v.map(hue));
        }
    }
    commands.spawn((
        Mesh2d(meshes.add(buf.mesh())),
        MeshMaterial2d(materials.add(ColorMaterial::from(art.grid.clone()))),
        Transform::from_xyz(0.0, 0.0, -100.0),
    ));
}

fn deco_emoji(kind: &str) -> &'static str {
    match kind {
        "tree" => "🌴",
        "flower" => "🌸",
        "bush" => "🌿",
        "crystal" => "💎",
        "bamboo" => "🎋",
        "mushroom" => "🍄",
        "lotus" => "🪷",
        "rock" => "🪨",
        _ => "🌱",
    }
}

/// Emoji text rendered at 2x and scaled down, so it stays crisp when zoomed in.
pub fn emoji_text(art: &Art, emoji: &str, size: f32) -> impl Bundle {
    (
        Text2d::new(emoji),
        TextFont { font: art.emoji.clone().into(), font_size: FontSize::Px(size * 2.0), ..default() },
    )
}

fn spawn_decorations(mut commands: Commands, world: Res<World>, decos: Res<Decos>, art: Res<Art>) {
    for d in &decos.0 {
        let Some(t) = world.tile_at(d.position.x, d.position.y) else { continue };
        // Lotus floats on water; everything else grows on land.
        if (d.kind == "lotus") != (t.kind == kind::WATER) {
            continue;
        }
        let jitter = Vec2::new(cell_hash(d.position.x as i32, 7) - 0.5, cell_hash(9, d.position.y as i32) - 0.5) * 0.5;
        let (x, y) = (d.position.x + jitter.x, d.position.y + jitter.y);
        let p = iso(x, y, t.height);
        commands.spawn((
            emoji_text(&art, deco_emoji(&d.kind), 18.0),
            Anchor::BOTTOM_CENTER,
            Transform::from_translation(p.extend(row_z(d.position.x, d.position.y)) + Vec3::Y * 2.0).with_scale(Vec3::splat(0.5)),
        ));
    }
}

pub fn node_color(kind: &str) -> Color {
    crate::data::tuning().node_colors.get(kind).map_or(Color::srgb(0.42, 0.45, 0.5), |h| crate::data::hex_color(h))
}

pub const ACCENT: Color = Color::srgb(0.0, 1.0, 0.533);

fn spawn_nodes(mut commands: Commands, world: Res<World>, art: Res<Art>) {
    for (i, node) in world.nodes.iter().enumerate() {
        let p = &node.position;
        let h = world.ground(p.x, p.y).max(p.height);
        let at = iso(p.x, p.y, h);
        let label = if node.name.chars().count() > 12 {
            format!("{}..", node.name.chars().take(10).collect::<String>())
        } else {
            node.name.clone()
        };
        // A soft shadow on the ground grounds the floating card.
        commands.spawn((
            Sprite { image: art.dot.clone(), color: Color::srgba(0.0, 0.0, 0.0, 0.45), custom_size: Some(Vec2::new(86.0, 30.0)), ..default() },
            Transform::from_translation((at - Vec2::Y * 38.0).extend(NODE_Z - 3.0)),
        ));
        commands
            .spawn((
                NodeCard(i),
                Sprite { image: art.card.clone(), color: node_color(&node.kind).with_alpha(0.9), custom_size: Some(Vec2::splat(80.0)), ..default() },
                Transform::from_translation(at.extend(NODE_Z + (p.x + p.y) * 0.01)),
            ))
            .with_children(|c| {
                c.spawn((
                    Sprite { image: art.card_border.clone(), color: ACCENT.with_alpha(0.8), custom_size: Some(Vec2::splat(80.0)), ..default() },
                    Transform::from_xyz(0.0, 0.0, 0.001),
                ));
                c.spawn((emoji_text(&art, &node.emoji, 32.0), Transform::from_xyz(0.0, 5.0, 0.002).with_scale(Vec3::splat(0.5))));
                c.spawn((
                    Text2d::new(label),
                    art.text(true, 20.0),
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, -28.0, 0.002).with_scale(Vec3::splat(0.5)),
                ));
            });
    }
}
