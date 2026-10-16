#![allow(deprecated)] // Interaction/Button: Bevy 0.20 wants picking + ui_widgets; migrate later.

//! Screens and overlays: the welcome gate, zoom HUD, hint bar and node panel.

use bevy::prelude::*;

use crate::ops::Game;
use crate::view::{PointerOverUi, ViewState, ZoomBy};
use crate::world::World;
use crate::{Art, Screen};

pub const GREEN_300: Color = Color::srgb(0.525, 0.937, 0.675);
pub const GREEN_400: Color = Color::srgb(0.290, 0.871, 0.502);
pub const GREEN_500: Color = Color::srgb(0.133, 0.773, 0.369);
pub const GREEN_900: Color = Color::srgb(0.078, 0.325, 0.176);
pub const RED_400: Color = Color::srgb(0.973, 0.443, 0.443);
pub const RED_500: Color = Color::srgb(0.937, 0.267, 0.267);

#[derive(Component)]
struct WelcomeRoot;
#[derive(Component)]
struct Progress {
    fill: Entity,
    label: Entity,
    pct: Entity,
    button_label: Entity,
}
#[derive(Component)]
struct Proceed;
#[derive(Component)]
struct ZoomButton(f32);
#[derive(Component)]
struct ZoomLabel;
#[derive(Component)]
struct PanelRoot;
#[derive(Component)]
struct CloseButton;
#[derive(Component, Clone, Copy, PartialEq)]
enum Tab {
    Info,
    Assets,
}
/// Pointer over this node means "not the world".
#[derive(Component)]
pub struct Blocks;

#[derive(Resource)]
struct PanelState {
    shown: Option<usize>,
    tab: Tab,
    dirty: bool,
    net_version: u32,
}

const LOAD_SECS: f32 = 0.9;

pub fn plugin(app: &mut App) {
    app.insert_resource(PanelState { shown: None, tab: Tab::Info, dirty: false, net_version: 0 })
        .add_systems(Startup, spawn_welcome)
        .add_systems(Update, (welcome, proceed).run_if(in_state(Screen::Welcome)))
        .add_systems(OnEnter(Screen::Playing), spawn_hud)
        .add_systems(
            Update,
            (pointer_over_ui, buttons, zoom_label, panel, scroll_panels).run_if(in_state(Screen::Playing)),
        );
}

fn font(handle: &Handle<Font>, size: f32) -> TextFont {
    TextFont { font: handle.clone().into(), font_size: FontSize::Px(size), ..default() }
}

fn label(art: &Art, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (Text::new(s), art.text(false, size), TextColor(color))
}

fn bold(art: &Art, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (Text::new(s), art.text(true, size), TextColor(color))
}

fn emoji(art: &Art, s: impl Into<String>, size: f32) -> impl Bundle {
    (Text::new(s), font(&art.emoji, size))
}

fn spawn_welcome(mut commands: Commands, art: Res<Art>) {
    let mut ids = (Entity::PLACEHOLDER, Entity::PLACEHOLDER, Entity::PLACEHOLDER, Entity::PLACEHOLDER);
    let root = commands
        .spawn((
            WelcomeRoot,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(px(16)),
                row_gap: px(16),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(100),
        ))
        .id();
    commands.entity(root).with_children(|root| {
        root.spawn((
            Node {
                width: percent(100),
                max_width: px(576),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(24)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BorderColor::all(GREEN_500.with_alpha(0.3)),
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    width: px(96),
                    height: px(96),
                    margin: UiRect::bottom(px(16)),
                    border: UiRect::all(px(4)),
                    border_radius: BorderRadius::MAX,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderColor::all(GREEN_500),
            ))
            .with_child(emoji(&art, "🐵", 40.0));
            card.spawn((bold(&art, "LUCKY MONKEY MEDIA", 28.0, GREEN_400), Node { margin: UiRect::bottom(px(8)), ..default() }));
            card.spawn((label(&art, "IT'S ALL IN YOUR HEAD | システムアクセス", 13.0, GREEN_300), Node { margin: UiRect::bottom(px(16)), ..default() }));
            card.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(px(14)),
                    margin: UiRect::bottom(px(16)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(6)),
                    row_gap: px(8),
                    ..default()
                },
                BorderColor::all(RED_500.with_alpha(0.4)),
            ))
            .with_children(|w| {
                w.spawn(bold(&art, "WARNING", 20.0, RED_500));
                w.spawn((
                    label(&art, "THE NODE YOU ARE ATTEMPTING TO ACCESS IS THE PROPERTY OF LUCKY MONKEY MEDIA. UNAUTHORIZED ACCESS BEYOND THIS POINT IS STRICTLY PROHIBITED.", 13.0, RED_400),
                    TextLayout::justify(Justify::Center),
                ));
                w.spawn((
                    label(&art, "IF YOU PROCEED YOU WILL BE IN VIOLATION OF INTERNATIONAL CORPORATE STATUTE TUF G3UTF9 AND WILL BE FORCIBLY DISCONNECTED BY THIS NODE'S ANTI-INTRUSION SECURITY SOFTWARE.", 13.0, RED_400),
                    TextLayout::justify(Justify::Center),
                ));
            });
            card.spawn(Node { width: percent(100), justify_content: JustifyContent::SpaceBetween, margin: UiRect::bottom(px(4)), ..default() })
                .with_children(|row| {
                    ids.1 = row.spawn(label(&art, "Initializing System...", 12.0, GREEN_400.with_alpha(0.7))).id();
                    ids.2 = row.spawn(label(&art, "0%", 12.0, GREEN_400.with_alpha(0.7))).id();
                });
            card.spawn((
                Node { width: percent(100), height: px(8), border: UiRect::all(px(1)), border_radius: BorderRadius::MAX, overflow: Overflow::clip(), ..default() },
                BorderColor::all(GREEN_500.with_alpha(0.3)),
            ))
            .with_children(|bar| {
                ids.0 = bar.spawn((Node { width: percent(0), height: percent(100), ..default() }, BackgroundColor(GREEN_500))).id();
            });
        });
        root.spawn((
            Proceed,
            Interaction::default(),
            Node {
                width: percent(100),
                max_width: px(576),
                padding: UiRect::vertical(px(12)),
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(Color::srgba(0.25, 0.25, 0.3, 0.5)),
            BackgroundColor(Color::srgba(0.07, 0.07, 0.09, 0.5)),
        ))
        .with_children(|b| {
            ids.3 = b.spawn(label(&art, "LOADING...", 14.0, Color::srgb(0.42, 0.45, 0.5))).id();
        });
    });
    commands.entity(root).insert(Progress { fill: ids.0, label: ids.1, pct: ids.2, button_label: ids.3 });
}

fn ready(time: &Time) -> bool {
    time.elapsed_secs() >= LOAD_SECS
}

/// Everything is compiled in, so "loading" is just a short boot animation.
fn welcome(
    time: Res<Time>,
    progress: Single<&Progress>,
    mut nodes: Query<&mut Node>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
    mut button: Single<(&mut BackgroundColor, &mut BorderColor), With<Proceed>>,
) {
    let pct = ((time.elapsed_secs() / LOAD_SECS).min(1.0) * 100.0).round();
    if let Ok(mut n) = nodes.get_mut(progress.fill) {
        n.width = percent(pct);
    }
    if let Ok((mut t, _)) = texts.get_mut(progress.pct) {
        t.0 = format!("{pct}%");
    }
    if ready(&time) {
        if let Ok((mut t, _)) = texts.get_mut(progress.label) {
            t.0 = "System ready".into();
        }
        if let Ok((mut t, mut c)) = texts.get_mut(progress.button_label) {
            t.0 = "PROCEED".into();
            c.0 = GREEN_400;
        }
        *button.0 = BackgroundColor(GREEN_900.with_alpha(0.5));
        *button.1 = BorderColor::all(GREEN_500.with_alpha(0.5));
    }
}

fn proceed(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    button: Query<&Interaction, (With<Proceed>, Changed<Interaction>)>,
    root: Single<Entity, With<WelcomeRoot>>,
    mut next: ResMut<NextState<Screen>>,
    link: Res<crate::link::DeepLink>,
) {
    let clicked = button.iter().any(|i| *i == Interaction::Pressed);
    // A deep link (#node=… etc.) goes straight in.
    if link.active() || (ready(&time) && (clicked || keys.just_pressed(KeyCode::Enter))) {
        commands.entity(*root).despawn();
        next.set(Screen::Playing);
    }
}

fn hud_button(art: &Art, text: &str) -> impl Bundle {
    (
        Interaction::default(),
        Blocks,
        Node {
            width: px(36),
            height: px(36),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderColor::all(GREEN_500.with_alpha(0.5)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
        children![label(art, text, 18.0, GREEN_400)],
    )
}

fn spawn_hud(mut commands: Commands, art: Res<Art>, windows: Query<&Window>) {
    commands.spawn((
        Node { position_type: PositionType::Absolute, top: px(16), right: px(16), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(8), ..default() },
        GlobalZIndex(40),
        children![
            (hud_button(&art, "+"), ZoomButton(1.2)),
            (
                Node { padding: UiRect::axes(px(5), px(2)), border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
                children![(ZoomLabel, label(&art, "100%", 12.0, GREEN_400))],
            ),
            (hud_button(&art, "−"), ZoomButton(1.0 / 1.2)),
        ],
    ));
    let touch = windows.iter().next().is_some_and(|w| w.width() < 768.0);
    let hint = if touch { "Drag to pan · pinch to zoom · tap a node, pick a fix" } else { "WASD/drag: pan · arrows: hop nodes · Q/E: islands · Space/Tab: alerts · 1-4: fix · G: go · J: swing 🍌 · F: find 🐒" };
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(8),
            left: px(8),
            padding: UiRect::axes(px(8), px(4)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BorderColor::all(GREEN_500.with_alpha(0.3)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.92)),
        GlobalZIndex(40),
        children![label(&art, hint, 11.0, GREEN_400)],
    ));
}

/// Mouse wheel over a scrollable panel scrolls it (the map ignores the wheel over UI).
fn scroll_panels(mut wheel: MessageReader<bevy::input::mouse::MouseWheel>, mut q: Query<(&Interaction, &Node, &mut ScrollPosition)>) {
    let dy: f32 = wheel
        .read()
        .map(|e| match e.unit {
            bevy::input::mouse::MouseScrollUnit::Line => e.y * 40.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => e.y,
        })
        .sum();
    if dy == 0.0 {
        return;
    }
    for (i, node, mut pos) in &mut q {
        if *i != Interaction::None && node.overflow.y == OverflowAxis::Scroll {
            pos.0.y = (pos.0.y - dy).max(0.0);
        }
    }
}

fn pointer_over_ui(blockers: Query<&Interaction, With<Blocks>>, mut over: ResMut<PointerOverUi>) {
    over.0 = blockers.iter().any(|i| *i != Interaction::None);
}

fn buttons(
    zooms: Query<(&Interaction, &ZoomButton), Changed<Interaction>>,
    closes: Query<&Interaction, (With<CloseButton>, Changed<Interaction>)>,
    tabs: Query<(&Interaction, &Tab), Changed<Interaction>>,
    mut zoom: MessageWriter<ZoomBy>,
    mut view: ResMut<ViewState>,
    mut panel: ResMut<PanelState>,
) {
    for (i, z) in &zooms {
        if *i == Interaction::Pressed {
            zoom.write(ZoomBy(z.0));
        }
    }
    if closes.iter().any(|i| *i == Interaction::Pressed) {
        view.selected = None;
    }
    for (i, tab) in &tabs {
        if *i == Interaction::Pressed && panel.tab != *tab {
            panel.tab = *tab;
            panel.dirty = true;
        }
    }
}

fn zoom_label(view: Res<ViewState>, mut label: Single<&mut Text, With<ZoomLabel>>) {
    if view.is_changed() {
        label.0 = format!("{}%", (view.zoom * 100.0).round());
    }
}

fn status_color(s: &str) -> Color {
    match s {
        "online" => Color::srgb(0.29, 0.87, 0.5),
        "unstable" | "degraded" => Color::srgb(0.98, 0.8, 0.08),
        "unreachable" => Color::srgb(0.45, 0.52, 0.62),
        "erupting" => Color::srgb(0.98, 0.57, 0.24),
        _ => Color::srgb(0.97, 0.44, 0.44),
    }
}

fn asset_icon(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("") {
        "json" => "{}",
        "bin" => "01",
        "dat" => "📊",
        "sys" => "⚙️",
        "log" => "📝",
        "txt" => "📄",
        "js" => "JS",
        "gif" => "🎞️",
        _ => "",
    }
}

fn section(art: &Art, title: &str) -> impl Bundle {
    (label(art, title, 10.0, GREEN_400.with_alpha(0.7)), Node { margin: UiRect::new(px(0), px(0), px(10), px(4)), ..default() })
}

/// Rebuild the node panel whenever the selection or tab changes.
fn panel(
    mut commands: Commands,
    art: Res<Art>,
    world: Res<World>,
    view: Res<ViewState>,
    game: Res<Game>,
    mut state: ResMut<PanelState>,
    roots: Query<Entity, With<PanelRoot>>,
    windows: Query<&Window>,
) {
    if state.shown == view.selected && !state.dirty && state.net_version == game.sim.version {
        return;
    }
    state.net_version = game.sim.version;
    if state.shown != view.selected {
        state.tab = Tab::Info;
    }
    state.shown = view.selected;
    state.dirty = false;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let Some(index) = view.selected.filter(|&i| i < world.nodes.len()) else { return };
    let node = &world.nodes[index];
    let tab = state.tab;
    let wide = windows.iter().next().is_some_and(|w| w.width() >= 900.0);

    commands
        .spawn((
            PanelRoot,
            Blocks,
            Interaction::default(),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(40),
                // Wide screens: hug the right edge (the left is the shift HUD and alert list).
                left: if wide { Val::Auto } else { px(16) },
                right: px(16),
                width: if wide { px(384) } else { Val::Auto },
                max_height: Val::Vh(if windows.iter().next().is_some_and(|w| w.width() < 600.0) { 55.0 } else { 86.0 }),
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BorderColor::all(GREEN_500.with_alpha(0.3)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.995)),
            GlobalZIndex(50),
        ))
        .with_children(|p| {
            // Header: emoji + name + IP, close button.
            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::all(px(12)), ..default() })
                .with_children(|h| {
                    h.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|t| {
                        t.spawn(Node { align_items: AlignItems::Center, column_gap: px(6), ..default() }).with_children(|n| {
                            n.spawn(emoji(&art, node.emoji.clone(), 20.0));
                            n.spawn(bold(&art, node.name.clone(), 14.0, GREEN_400));
                        });
                        let status = game.sim.health[index].label();
                        t.spawn(Node { align_items: AlignItems::Center, column_gap: px(6), ..default() }).with_children(|r| {
                            r.spawn((Node { width: px(8), height: px(8), border_radius: BorderRadius::MAX, ..default() }, BackgroundColor(status_color(status))));
                            r.spawn(label(&art, format!("{} · {}", status.to_uppercase(), node.ip), 10.0, GREEN_300));
                        });
                    });
                    h.spawn((CloseButton, Interaction::default(), Node { width: px(32), height: px(32), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                        .with_child(label(&art, "×", 20.0, GREEN_400));
                });
            // Tabs.
            p.spawn((Node { border: UiRect::vertical(px(1)), ..default() }, BorderColor::all(GREEN_500.with_alpha(0.2)))).with_children(|tabs| {
                for (t, name) in [(Tab::Info, "INFO"), (Tab::Assets, "ASSETS")] {
                    let on = t == tab;
                    tabs.spawn((
                        t,
                        Interaction::default(),
                        Node { flex_grow: 1.0, justify_content: JustifyContent::Center, padding: UiRect::vertical(px(5)), ..default() },
                        BackgroundColor(if on { GREEN_900.with_alpha(0.3) } else { Color::NONE }),
                    ))
                    .with_child(bold(&art, name, 10.0, if on { GREEN_400 } else { GREEN_500.with_alpha(0.7) }));
                }
            });
            p.spawn(Node { flex_direction: FlexDirection::Column, padding: UiRect::all(px(12)), ..default() }).with_children(|body| match tab {
                Tab::Info => {
                    // Urgent first: incident, time to loss, crew, responses. Flavour text last
                    // when something is wrong; first when all is calm.
                    crate::ops::alert_box(body, &art, &game, index);
                    let calm = game.sim.faults[index].is_none();
                    let desc = if node.description.is_empty() { "No description available." } else { &node.description };
                    if calm {
                        body.spawn(label(&art, desc, 12.0, GREEN_300));
                    }
                    if !node.connections.is_empty() {
                        body.spawn(section(&art, "接続 Connections"));
                        body.spawn(Node { flex_wrap: FlexWrap::Wrap, row_gap: px(4), ..default() }).with_children(|g| {
                            for c in node.connections.iter().filter_map(|id| world.nodes.iter().find(|n| &n.id == id)) {
                                g.spawn(Node { width: percent(50), align_items: AlignItems::Center, column_gap: px(4), overflow: Overflow::clip(), ..default() })
                                    .with_children(|r| {
                                        r.spawn(emoji(&art, c.emoji.clone(), 13.0));
                                        r.spawn(label(&art, c.name.clone(), 11.0, GREEN_300));
                                    });
                            }
                        });
                    }
                    let sim = &game.sim;
                    if let Some(role) = sim.rules.roles.get(&node.kind) {
                        body.spawn(section(&art, "役割 Role"));
                        body.spawn(label(&art, role.clone(), 11.0, GREEN_300));
                    }
                    let impact = if index == sim.core {
                        "This is the core router: every path runs through it.".to_string()
                    } else {
                        match sim.impact_if_down(index) {
                            0 => "If it goes down, other paths route around it.".to_string(),
                            k => format!("If it goes down, {k} other service(s) lose their path to the core."),
                        }
                    };
                    body.spawn(label(&art, impact, 11.0, Color::srgb(0.55, 0.8, 1.0)));
                    if !node.lore.is_empty() {
                        body.spawn(section(&art, "伝説 Lore"));
                        body.spawn(label(&art, node.lore.clone(), 11.0, Color::srgb(0.85, 0.82, 0.68)));
                    }
                    if !calm {
                        body.spawn(section(&art, "説明 About"));
                        body.spawn(label(&art, desc, 11.0, GREEN_300.with_alpha(0.8)));
                    }
                    if let Some(level) = node.security {
                        body.spawn(section(&art, "セキュリティ Security"));
                        body.spawn(Node { align_items: AlignItems::Center, column_gap: px(2), ..default() }).with_children(|r| {
                            for i in 0..5 {
                                let on = i < level;
                                r.spawn((
                                    Node { width: px(12), height: px(12), border_radius: BorderRadius::all(px(2)), ..default() },
                                    BackgroundColor(if on { GREEN_500 } else { GREEN_900.with_alpha(0.5) }),
                                ));
                            }
                            r.spawn((label(&art, format!("Level {level}"), 12.0, GREEN_300), Node { margin: UiRect::left(px(8)), ..default() }));
                        });
                    }
                }
                Tab::Assets => {
                    body.spawn(section(&art, "ファイル Files"));
                    if node.assets.is_empty() {
                        body.spawn(label(&art, "No assets found", 12.0, GREEN_500.with_alpha(0.5)));
                    }
                    for a in &node.assets {
                        body.spawn(Node { align_items: AlignItems::Center, column_gap: px(8), margin: UiRect::bottom(px(6)), ..default() }).with_children(|r| {
                            let icon = asset_icon(a);
                            let is_emoji = !icon.is_ascii();
                            r.spawn((
                                Node {
                                    width: px(24),
                                    height: px(24),
                                    border: UiRect::all(px(1)),
                                    border_radius: BorderRadius::all(px(2)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BorderColor::all(GREEN_500.with_alpha(0.3)),
                                BackgroundColor(GREEN_900.with_alpha(0.5)),
                            ))
                            .with_children(|b| {
                                if is_emoji {
                                    b.spawn(emoji(&art, icon, 12.0));
                                } else {
                                    b.spawn(bold(&art, icon, 10.0, GREEN_300));
                                }
                            });
                            r.spawn(label(&art, a.clone(), 12.0, GREEN_300));
                        });
                    }
                }
            });
        });
}

