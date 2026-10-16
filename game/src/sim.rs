//! The on-call simulation, free of Bevy: the network, faults, the incident
//! director, the animal workers and the event stream.
//!
//! The game (ops.rs) owns one `Sim`, steps it every frame and draws it; the
//! native test crate (game/simtest) runs the same code headless for tests and
//! balancing. Everything tunable comes from `Rules` (tuning.json).
//!
//! Key ideas:
//! - **Root causes vs impacts.** A fault lives on one node. Nodes that lose
//!   their path to the core router because of a down node are *unreachable*:
//!   an impact, not a fault. They recover by themselves when the path returns.
//! - **The director** decides when new root causes appear: never during the
//!   grace period after a fix, never above the root budget, never while the
//!   network is already badly hurt. Spreading faults spread at most once.
//! - **Mitigation vs remediation.** Each fault lists which actions fix it,
//!   which only buy time (and for how long), and why the others don't help.
//! - **Time to Loss** is a game timer: seconds until the next consequence. It
//!   is not IP TTL (a hop count) or DNS TTL (a cache lifetime).

use std::collections::{HashMap, VecDeque};

use serde::Deserialize;

// ---------------------------------------------------------------- rules

#[derive(Deserialize, Clone, Debug)]
pub struct WarningDef {
    pub emoji: String,
    pub text: String,
    /// Actions that prevent it while it is still a warning.
    pub prevent: Vec<String>,
    /// Seconds before the warning becomes the incident.
    pub ttl: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Response {
    /// "fix" (permanent), "mitigate" (temporary), or "none".
    pub effect: String,
    /// Mitigations: seconds of relief.
    #[serde(default)]
    pub secs: f32,
    /// One-line explanation of what happened.
    pub note: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FaultDef {
    pub emoji: String,
    pub name: String,
    /// Taxonomy group: network, security, systems, operations, advanced.
    #[allow(dead_code)]
    pub group: String,
    pub symptom: String,
    /// Seconds from incident to the node going down.
    pub ttl: f32,
    #[serde(default)]
    pub warning: Option<WarningDef>,
    /// Action key -> what it does to this fault. Unlisted actions do nothing.
    pub responses: HashMap<String, Response>,
    /// Extra wrong-but-plausible actions to offer.
    #[serde(default)]
    pub decoys: Vec<String>,
    /// Field-guide concept learned by resolving or diagnosing it.
    pub concept: String,
    /// Node kinds it can hit (empty: any).
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Worms and floods may reach one neighbour once the node is down.
    #[serde(default)]
    pub spreads: bool,
    /// Director weight (0: only scripted or caused by something else).
    #[serde(default = "one")]
    pub weight: f32,
    /// Not before this difficulty level.
    #[serde(default)]
    pub min_level: usize,
    /// Not before this fraction of the shift.
    #[serde(default)]
    pub min_progress: f32,
}

fn one() -> f32 {
    1.0
}

#[derive(Deserialize, Clone, Debug)]
pub struct ActionDef {
    pub emoji: String,
    pub label: String,
    pub secs: f32,
    /// Hands-on work the parrot can't do.
    #[serde(default)]
    pub heavy: bool,
    /// Preventive maintenance (the sloth's speciality).
    #[serde(default)]
    pub maintenance: bool,
    /// Physical hardware work (the tree frog's speciality).
    #[serde(default)]
    pub hardware: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WorkerDef {
    pub name: String,
    pub species: String,
    pub emoji: String,
    pub role: String,
    /// Node id it starts at.
    pub home: String,
    /// px/s along links (or straight, if it flies).
    pub speed: f32,
    #[serde(default)]
    pub flies: bool,
    /// Work time multiplier (2 = twice as slow).
    pub work: f32,
    /// Multiplier for maintenance actions instead of `work`.
    #[serde(default)]
    pub maintenance_work: Option<f32>,
    /// Multiplier for diagnosis instead of `work`.
    #[serde(default)]
    pub diagnose_work: Option<f32>,
    /// Multiplier for hardware actions instead of `work`.
    #[serde(default)]
    pub hardware_work: Option<f32>,
    #[serde(default = "yes")]
    pub can_heavy: bool,
    /// Jobs before a rest.
    pub stamina: u32,
    pub rest: f32,
}

fn yes() -> bool {
    true
}

#[derive(Deserialize, Clone, Debug)]
pub struct Level {
    pub name: String,
    /// Scripted opening (the first shift).
    #[serde(default)]
    pub tutorial: bool,
    /// Seconds before the first root cause (when not scripted).
    pub first: f32,
    /// [morning, evening] seconds between new root causes.
    pub interval: [f32; 2],
    /// [morning, evening] most root causes open at once.
    pub max_roots: [f32; 2],
    /// Share of new root causes that start as warnings (preventable).
    pub warning_share: f32,
    /// Multiplier on fault TTLs.
    #[serde(default = "one")]
    pub ttl_mult: f32,
    /// Worker names on shift (empty: everyone). Others can be called in for bananas.
    #[serde(default)]
    pub crew: Vec<String>,
    /// No early end and no spreading: a place to learn.
    #[serde(default)]
    pub practice: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Director {
    /// Calm seconds after any fix before a new root cause.
    pub grace: f32,
    /// No new root causes below this network health.
    pub health_floor: f32,
    /// Seconds a spreading fault must stay down before it spreads (once).
    pub spread_after: f32,
    /// Seconds between flavour notices in calm periods.
    pub notice_every: f32,
    pub levels: Vec<Level>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TutorialStep {
    pub fault: String,
    pub node: String,
    #[serde(default)]
    pub warning: bool,
    /// Coach line for the hint bar while this step is open.
    pub coach: String,
    /// Spawn together with the previous step.
    #[serde(default)]
    pub with_previous: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Concept {
    pub name: String,
    pub text: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Notice {
    pub text: String,
    #[serde(default)]
    pub kinds: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Rules {
    pub shift_secs: f32,
    /// Shift SLA target (0-100).
    pub sla_target: f32,
    /// The shift ends early if network health stays below this...
    pub outage_floor: f32,
    /// ...for this many seconds.
    pub outage_secs: f32,
    pub hike_speed: f32,
    pub points_fix: f32,
    pub points_prevent: f32,
    pub points_per_ttl_sec: f32,
    pub bananas_start: u32,
    pub banana_fix: u32,
    pub banana_prevent: u32,
    pub swing_cost: u32,
    pub swing_speed: f32,
    pub rally_cost: u32,
    pub diagnose_secs: f32,
    pub queue_max: usize,
    pub faults: HashMap<String, FaultDef>,
    pub actions: HashMap<String, ActionDef>,
    pub workers: Vec<WorkerDef>,
    pub director: Director,
    pub tutorial: Vec<TutorialStep>,
    pub concepts: HashMap<String, Concept>,
    #[serde(default)]
    pub notices: Vec<Notice>,
    /// Node kind -> weight in network health (default 1).
    #[serde(default)]
    pub criticality: HashMap<String, f32>,
    /// Node kind -> what it does, in plain words (tour, panel).
    #[serde(default)]
    pub roles: HashMap<String, String>,
    /// Bananas to call an off-shift worker in.
    #[serde(default = "three")]
    pub call_cost: u32,
}

fn three() -> u32 {
    3
}

impl Rules {
    pub fn fault(&self, k: &str) -> Option<&FaultDef> {
        self.faults.get(k)
    }
    pub fn action(&self, k: &str) -> Option<&ActionDef> {
        self.actions.get(k)
    }
}

// ---------------------------------------------------------------- state

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Health {
    Online,
    /// Impaired by its own fault.
    Degraded,
    /// Down by its own fault.
    Down,
    /// Fine itself, but cut off from the core (a dependency impact).
    Unreachable,
}

impl Health {
    pub fn label(self) -> &'static str {
        match self {
            Health::Online => "online",
            Health::Degraded => "degraded",
            Health::Down => "down",
            Health::Unreachable => "unreachable",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    /// Functional; will become an incident (preventable).
    Warning,
    /// Impaired; will go down.
    Incident,
    Down,
}

#[derive(Clone, Debug)]
pub struct Fault {
    pub id: u32,
    /// Root cause id (itself, or the fault it spread from).
    pub root: u32,
    pub kind: String,
    pub stage: Stage,
    /// Seconds to the next consequence (warning→incident, incident→down).
    pub ttl: f32,
    /// Seconds of temporary relief left (mitigation).
    pub mitigated: f32,
    pub mitigation: Option<String>,
    pub diagnosed: bool,
    /// The four actions offered.
    pub options: Vec<String>,
    /// Options already tried that didn't help (bitmask).
    pub tried: u8,
    pub since: f32,
    pub down_for: f32,
    pub spread_done: bool,
}

impl Fault {
    pub fn is_warning(&self) -> bool {
        self.stage == Stage::Warning
    }
}

#[derive(Clone, Debug)]
pub struct Task {
    pub node: usize,
    /// None: just go there.
    pub action: Option<String>,
    /// The fault it was ordered against.
    pub fault: Option<u32>,
    pub left: f32,
    pub total: f32,
    pub started: bool,
}

#[derive(Clone, Debug)]
pub struct Worker {
    pub def: usize,
    /// On shift. Off-shift workers can be called in (`call_in`).
    pub active: bool,
    pub name: String,
    pub emoji: String,
    pub at: usize,
    pub path: VecDeque<usize>,
    pub leg: f32,
    pub leg_len: f32,
    /// Start of the current leg when it isn't a node (swings, flights from mid-air).
    pub origin: Option<[f32; 2]>,
    pub overland: bool,
    pub swing: bool,
    pub queue: VecDeque<Task>,
    pub stamina: u32,
    pub rest: f32,
    pub pos: [f32; 2],
    pub busy_secs: f32,
    pub idle_secs: f32,
}

impl Worker {
    pub fn walking(&self) -> bool {
        !self.path.is_empty()
    }
    #[allow(dead_code)]
    pub fn working(&self) -> bool {
        self.queue.front().is_some_and(|t| t.started)
    }
    pub fn resting(&self) -> bool {
        self.rest > 0.0
    }
    #[allow(dead_code)]
    pub fn idle(&self) -> bool {
        !self.walking() && self.queue.is_empty() && !self.resting()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Notice,
    Warning,
    Incident,
    Critical,
    Dispatch,
    Arrive,
    Work,
    Resolved,
    Mitigated,
    NoEffect,
    Recover,
    Cascade,
    Learn,
    Rest,
    Shift,
}

impl Cat {
    /// Urgent categories are never collapsed away by worker chatter.
    pub fn urgent(self) -> bool {
        matches!(self, Cat::Warning | Cat::Incident | Cat::Critical | Cat::Cascade)
    }
    pub fn tag(self) -> &'static str {
        match self {
            Cat::Notice => "NOTICE",
            Cat::Warning => "WARNING",
            Cat::Incident => "INCIDENT",
            Cat::Critical => "CRITICAL",
            Cat::Dispatch => "DISPATCH",
            Cat::Arrive => "ARRIVED",
            Cat::Work => "WORKING",
            Cat::Resolved => "RESOLVED",
            Cat::Mitigated => "MITIGATED",
            Cat::NoEffect => "NO EFFECT",
            Cat::Recover => "RECOVERED",
            Cat::Cascade => "IMPACT",
            Cat::Learn => "LEARNED",
            Cat::Rest => "RESTING",
            Cat::Shift => "SHIFT",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub t: f32,
    pub cat: Cat,
    pub icon: String,
    pub text: String,
    pub node: Option<usize>,
    #[allow(dead_code)]
    pub worker: Option<usize>,
    /// Same group within a few seconds merges into one line with a count.
    pub group: Option<String>,
    pub count: u32,
}

/// Merge window for grouped events.
pub const GROUP_SECS: f32 = 6.0;
const LOG_MAX: usize = 400;

/// Append an event, merging it into a recent one of the same group.
pub fn push_event(log: &mut Vec<Event>, e: Event) {
    if let Some(g) = &e.group {
        if let Some(prev) = log.iter_mut().rev().take(12).find(|p| p.group.as_ref() == Some(g) && e.t - p.t <= GROUP_SECS) {
            prev.count += 1;
            prev.t = e.t;
            prev.text = e.text;
            return;
        }
    }
    log.push(e);
    if log.len() > LOG_MAX {
        log.drain(0..log.len() - LOG_MAX);
    }
}

/// What to show in a feed of `rows` lines: the newest events, but urgent ones
/// from the last `keep` seconds always stay in view ahead of chatter.
pub fn feed(log: &[Event], now: f32, rows: usize, keep: f32) -> Vec<&Event> {
    let mut urgent: Vec<&Event> = log.iter().rev().filter(|e| e.cat.urgent() && now - e.t <= keep).take(rows / 2).collect();
    let rest: Vec<&Event> = log.iter().rev().filter(|e| !urgent.iter().any(|u| std::ptr::eq(*u, *e))).take(rows - urgent.len()).collect();
    urgent.extend(rest);
    urgent.sort_by(|a, b| b.t.total_cmp(&a.t));
    urgent
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Phase {
    #[default]
    Idle,
    Running,
    Done,
    /// Ended early: the network stayed below the outage floor.
    Collapsed,
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub roots: u32,
    pub warnings: u32,
    pub resolved: u32,
    pub prevented: u32,
    pub mitigations: u32,
    pub no_effect: u32,
    pub outages: u32,
    pub spreads: u32,
    pub escalated_warnings: u32,
    pub recovery_total: f32,
    pub max_open: u32,
    pub critical_secs: f32,
    /// action key -> times it fixed something
    pub fixes_by: HashMap<String, u32>,
    pub points_fix: u32,
    pub points_prevent: u32,
    pub points_combo: u32,
    pub called_in: u32,
}

/// The next consequence: what the big Time to Loss timer shows.
#[derive(Clone, Debug)]
pub struct Deadline {
    pub node: usize,
    pub secs: f32,
    pub what: String,
}

#[derive(Clone, Debug)]
pub struct SimNode {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub pos: [f32; 2],
}

/// A brief "what happened and why" card after a fix or a failed attempt.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub node: usize,
    pub ok: bool,
    pub title: String,
    pub cause: String,
    pub action: String,
    pub learned: Option<String>,
    pub t: f32,
}

pub struct Sim {
    pub rules: Rules,
    pub nodes: Vec<SimNode>,
    pub links: Vec<(usize, usize)>,
    pub neighbours: Vec<Vec<usize>>,
    /// The core router (first) plus any redundant core nodes (the Guardian).
    pub core: usize,
    pub cores: Vec<usize>,
    pub watchtower: Option<usize>,
    pub health: Vec<Health>,
    pub faults: Vec<Option<Fault>>,
    pub workers: Vec<Worker>,
    pub phase: Phase,
    pub level: usize,
    pub clock: f32,
    rng: u64,
    pub log: Vec<Event>,
    pub stats: Stats,
    pub learned: Vec<(String, String)>,
    pub outcome: Option<Outcome>,
    /// What went right / wrong this shift, for the summary.
    pub notes_good: Vec<String>,
    pub notes_bad: Vec<String>,
    pub bananas: u32,
    pub score: u32,
    pub combo: u32,
    /// Network health now (0-100) and the shift SLA (time-averaged health).
    pub health_now: f32,
    pub sla: f32,
    sla_sum: f32,
    outage_for: f32,
    next_root: f32,
    grace_until: f32,
    next_notice: f32,
    tutorial: usize,
    tutorial_open: Vec<u32>,
    next_id: u32,
    pub version: u32,
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

impl Sim {
    pub fn new(rules: Rules, nodes: Vec<SimNode>, links: Vec<(usize, usize)>, seed: u64) -> Sim {
        let n = nodes.len();
        let mut neighbours = vec![Vec::new(); n];
        for &(a, b) in &links {
            neighbours[a].push(b);
            neighbours[b].push(a);
        }
        let core = nodes.iter().position(|x| x.kind == "router").or_else(|| nodes.iter().position(|x| x.kind == "monkey")).unwrap_or(0);
        // One core router. (Listing more here makes the core redundant; with this
        // meshed a graph that leaves single failures almost no dependency impact.)
        let cores = vec![core];
        let watchtower = nodes.iter().position(|x| x.kind == "monitor");
        let mut sim = Sim {
            rules,
            nodes,
            links,
            neighbours,
            core,
            cores,
            watchtower,
            health: vec![Health::Online; n],
            faults: vec![None; n],
            workers: Vec::new(),
            phase: Phase::Idle,
            level: 0,
            clock: 0.0,
            rng: seed.max(1),
            log: Vec::new(),
            stats: Stats::default(),
            learned: Vec::new(),
            outcome: None,
            notes_good: Vec::new(),
            notes_bad: Vec::new(),
            bananas: 0,
            score: 0,
            combo: 0,
            health_now: 100.0,
            sla: 100.0,
            sla_sum: 0.0,
            outage_for: 0.0,
            next_root: 0.0,
            grace_until: 0.0,
            next_notice: 0.0,
            tutorial: 0,
            tutorial_open: Vec::new(),
            next_id: 1,
            version: 0,
        };
        sim.spawn_workers();
        sim
    }

    fn spawn_workers(&mut self) {
        self.workers = self
            .rules
            .workers
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let at = self.nodes.iter().position(|n| n.id == d.home).unwrap_or(self.core);
                Worker {
                    def: i,
                    active: true,
                    name: d.name.clone(),
                    emoji: d.emoji.clone(),
                    at,
                    path: VecDeque::new(),
                    leg: 0.0,
                    leg_len: 1.0,
                    origin: None,
                    overland: false,
                    swing: false,
                    queue: VecDeque::new(),
                    stamina: d.stamina,
                    rest: 0.0,
                    pos: self.nodes[at].pos,
                    busy_secs: 0.0,
                    idle_secs: 0.0,
                }
            })
            .collect();
    }

    // ------------------------------------------------------------ rng

    pub fn rand(&mut self) -> f32 {
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        ((self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32) / (1u64 << 24) as f32
    }
    fn below(&mut self, n: usize) -> usize {
        ((self.rand() * n as f32) as usize).min(n.saturating_sub(1))
    }

    // ------------------------------------------------------------ queries

    pub fn progress(&self) -> f32 {
        (self.clock / self.rules.shift_secs).clamp(0.0, 1.0)
    }
    pub fn level_def(&self) -> &Level {
        let l = &self.rules.director.levels;
        &l[self.level.min(l.len() - 1)]
    }
    /// Forecasts (warnings) are visible while the Watchtower is reachable.
    pub fn sighted(&self) -> bool {
        self.watchtower.is_none_or(|w| self.health[w] == Health::Online || self.health[w] == Health::Degraded)
    }
    pub fn fault_def(&self, node: usize) -> Option<&FaultDef> {
        self.faults[node].as_ref().and_then(|f| self.rules.fault(&f.kind))
    }
    #[allow(dead_code)]
    pub fn open_roots(&self) -> usize {
        self.faults.iter().flatten().filter(|f| f.id == f.root).count()
    }
    /// Root causes still in play (warning or incident): what the director budgets.
    /// Outages left down don't hold the budget, so ignoring trouble has a cost.
    pub fn active_roots(&self) -> usize {
        self.faults.iter().flatten().filter(|f| f.id == f.root && (f.stage != Stage::Down || f.mitigated > 0.0)).count()
    }
    pub fn open_faults(&self) -> usize {
        self.faults.iter().flatten().count()
    }
    pub fn unreachable(&self) -> usize {
        self.health.iter().filter(|h| **h == Health::Unreachable).count()
    }
    /// Which nodes' faults the player can see (warnings need the Watchtower).
    pub fn visible(&self, node: usize) -> bool {
        self.faults[node].as_ref().is_some_and(|f| !f.is_warning() || self.sighted())
    }
    /// What happens if `action` is used on this node's fault.
    pub fn response(&self, node: usize, action: &str) -> Option<&Response> {
        self.fault_def(node)?.responses.get(action)
    }
    /// Right actions for the current stage (prevent while a warning).
    pub fn fixes_for(&self, node: usize) -> Vec<String> {
        let Some(f) = &self.faults[node] else { return Vec::new() };
        let Some(d) = self.rules.fault(&f.kind) else { return Vec::new() };
        if f.is_warning() {
            d.warning.as_ref().map(|w| w.prevent.clone()).unwrap_or_default()
        } else {
            d.responses.iter().filter(|(_, r)| r.effect == "fix").map(|(k, _)| k.clone()).collect()
        }
    }

    /// The earliest consequence: warning becoming an incident, an incident
    /// taking its node down, a mitigation wearing off, or a spread.
    pub fn deadline(&self) -> Option<Deadline> {
        let mut best: Option<Deadline> = None;
        for (i, f) in self.faults.iter().enumerate() {
            let Some(f) = f else { continue };
            if !self.visible(i) {
                continue;
            }
            let name = self.rules.fault(&f.kind).map_or(f.kind.as_str(), |d| d.name.as_str());
            let (secs, what) = if f.mitigated > 0.0 {
                (f.mitigated, format!("{name} returns (mitigation wears off)"))
            } else {
                match f.stage {
                    Stage::Warning => (f.ttl, format!("{name} hits")),
                    Stage::Incident => (f.ttl, format!("goes down: {name}")),
                    Stage::Down => {
                        let spreads = self.rules.fault(&f.kind).is_some_and(|d| d.spreads) && !f.spread_done;
                        if !spreads {
                            continue;
                        }
                        ((self.rules.director.spread_after - f.down_for).max(0.0), format!("{name} spreads to a neighbour"))
                    }
                }
            };
            if best.as_ref().is_none_or(|b| secs < b.secs) {
                best = Some(Deadline { node: i, secs, what });
            }
        }
        best
    }

    /// Coach line for the tutorial step in progress.
    pub fn coach(&self) -> Option<&str> {
        if !self.level_def().tutorial || self.tutorial == 0 || self.tutorial_open.is_empty() {
            return None;
        }
        self.rules.tutorial.get(self.tutorial - 1).map(|s| s.coach.as_str())
    }

    // ------------------------------------------------------------ events

    fn say(&mut self, cat: Cat, icon: &str, text: String, node: Option<usize>, worker: Option<usize>, group: Option<String>) {
        let e = Event { t: self.clock, cat, icon: icon.into(), text, node, worker, group, count: 1 };
        push_event(&mut self.log, e);
        self.version = self.version.wrapping_add(1);
    }

    fn learn(&mut self, concept: &str, example: String) -> Option<String> {
        let c = self.rules.concepts.get(concept)?.clone();
        if !self.learned.iter().any(|(k, _)| k == concept) {
            self.learned.push((concept.to_string(), example));
            self.say(Cat::Learn, "📖", format!("{}: {}", c.name, c.text), None, None, None);
        }
        Some(format!("{}: {}", c.name, c.text))
    }

    fn note(&mut self, good: bool, s: String) {
        let list = if good { &mut self.notes_good } else { &mut self.notes_bad };
        if !list.contains(&s) && list.len() < 12 {
            list.push(s);
        }
    }

    /// How many services would lose their path to the core if `node` went down.
    pub fn impact_if_down(&self, node: usize) -> usize {
        let n = self.nodes.len();
        let blocked = |i: usize| i == node || self.health[i] == Health::Down;
        let mut reach = vec![false; n];
        let mut stack: Vec<usize> = self.cores.iter().copied().filter(|&c| !blocked(c)).collect();
        for &c in &stack {
            reach[c] = true;
        }
        while let Some(u) = stack.pop() {
            for &v in &self.neighbours[u] {
                if !reach[v] && !blocked(v) {
                    reach[v] = true;
                    stack.push(v);
                }
            }
        }
        let before = (0..n).filter(|&i| self.health[i] != Health::Unreachable && self.health[i] != Health::Down).count();
        let after = (0..n).filter(|&i| reach[i]).count();
        before.saturating_sub(after).saturating_sub(1)
    }

    /// Spend bananas to bring an off-shift worker in.
    pub fn call_in(&mut self, w: usize) -> Result<(), String> {
        if self.workers[w].active {
            return Err(format!("{} is already on shift", self.workers[w].name));
        }
        if self.bananas < self.rules.call_cost {
            return Err(format!("Calling {} in costs {} 🍌", self.workers[w].name, self.rules.call_cost));
        }
        self.bananas -= self.rules.call_cost;
        self.workers[w].active = true;
        self.stats.called_in += 1;
        let (name, emoji) = (self.workers[w].name.clone(), self.workers[w].emoji.clone());
        self.say(Cat::Dispatch, &emoji, format!("{name} called in to help (🍌 -{})", self.rules.call_cost), None, Some(w), None);
        Ok(())
    }

    // ------------------------------------------------------------ shift

    pub fn start(&mut self, level: usize) {
        self.level = level.min(self.rules.director.levels.len() - 1);
        self.phase = Phase::Running;
        self.clock = 0.0;
        self.faults.iter_mut().for_each(|f| *f = None);
        self.stats = Stats::default();
        self.bananas = self.rules.bananas_start;
        self.score = 0;
        self.combo = 0;
        self.sla_sum = 0.0;
        self.sla = 100.0;
        self.outage_for = 0.0;
        self.outcome = None;
        self.grace_until = 0.0;
        self.tutorial = 0;
        self.tutorial_open.clear();
        self.next_root = self.level_def().first;
        self.next_notice = 12.0;
        self.log.clear();
        self.notes_good.clear();
        self.notes_bad.clear();
        self.spawn_workers();
        let crew = self.level_def().crew.clone();
        for w in &mut self.workers {
            w.active = crew.is_empty() || crew.contains(&w.name);
        }
        self.recompute();
        let name = self.level_def().name.clone();
        self.say(Cat::Shift, "🕘", format!("09:00 shift started ({name}). Keep the jungle network up until 17:00."), None, None, None);
    }

    /// Raise a fault on a node. `root`: the fault it came from (spreads).
    pub fn raise(&mut self, node: usize, kind: &str, warning: bool, root: Option<u32>) -> bool {
        if self.faults[node].is_some() {
            return false;
        }
        let Some(def) = self.rules.fault(kind).cloned() else { return false };
        let warning = warning && def.warning.is_some();
        let ttl_mult = self.level_def().ttl_mult;
        let ttl = if warning { def.warning.as_ref().unwrap().ttl } else { def.ttl } * ttl_mult;
        let id = self.next_id;
        self.next_id += 1;
        let options = self.options_for(&def, warning);
        self.faults[node] = Some(Fault {
            id,
            root: root.unwrap_or(id),
            kind: kind.to_string(),
            stage: if warning { Stage::Warning } else { Stage::Incident },
            ttl,
            mitigated: 0.0,
            mitigation: None,
            diagnosed: false,
            options,
            tried: 0,
            since: self.clock,
            down_for: 0.0,
            spread_done: root.is_some(),
        });
        let name = self.nodes[node].name.clone();
        if root.is_none() {
            self.stats.roots += 1;
        }
        if warning {
            self.stats.warnings += 1;
            let w = def.warning.as_ref().unwrap();
            if self.sighted() {
                self.say(Cat::Warning, &w.emoji, format!("{name}: {}", w.text), Some(node), None, None);
            }
        } else if let Some(r) = root {
            self.say(Cat::Cascade, &def.emoji, format!("{} spread to {name}", def.name), Some(node), None, Some(format!("spread{r}")));
        } else {
            self.say(Cat::Incident, &def.emoji, format!("{name}: {}. {}", def.name, def.symptom), Some(node), None, None);
        }
        self.recompute();
        true
    }

    /// Four options: the fix(es), a mitigation if any, plus plausible decoys.
    fn options_for(&mut self, def: &FaultDef, warning: bool) -> Vec<String> {
        let mut opts: Vec<String> = Vec::new();
        if warning {
            if let Some(w) = &def.warning {
                opts.extend(w.prevent.iter().take(1).cloned());
            }
        } else {
            let mut fixes: Vec<&String> = def.responses.iter().filter(|(_, r)| r.effect == "fix").map(|(k, _)| k).collect();
            fixes.sort();
            opts.extend(fixes.into_iter().take(2).cloned());
            let mut mits: Vec<&String> = def.responses.iter().filter(|(_, r)| r.effect == "mitigate").map(|(k, _)| k).collect();
            mits.sort();
            opts.extend(mits.into_iter().take(1).cloned());
        }
        for d in def.decoys.iter().chain(def.responses.iter().filter(|(_, r)| r.effect == "none").map(|(k, _)| k)) {
            if opts.len() >= 4 {
                break;
            }
            if !opts.contains(d) && self.rules.actions.contains_key(d) {
                opts.push(d.clone());
            }
        }
        let mut pool: Vec<String> = self.rules.actions.keys().filter(|k| !opts.contains(k)).cloned().collect();
        pool.sort();
        while opts.len() < 4 && !pool.is_empty() {
            let k = self.below(pool.len());
            opts.push(pool.remove(k));
        }
        for i in (1..opts.len()).rev() {
            let j = self.below(i + 1);
            opts.swap(i, j);
        }
        opts
    }

    /// Health from faults, then reachability from the core.
    fn recompute(&mut self) {
        let n = self.nodes.len();
        let own: Vec<Health> = (0..n)
            .map(|i| match &self.faults[i] {
                None => Health::Online,
                Some(f) if f.mitigated > 0.0 => Health::Online,
                Some(f) => match f.stage {
                    Stage::Warning => Health::Online,
                    Stage::Incident => Health::Degraded,
                    Stage::Down => Health::Down,
                },
            })
            .collect();
        let mut reach = vec![false; n];
        let mut stack: Vec<usize> = self.cores.iter().copied().filter(|&c| own[c] != Health::Down).collect();
        for &c in &stack {
            reach[c] = true;
        }
        {
            while let Some(u) = stack.pop() {
                for &v in &self.neighbours[u] {
                    if !reach[v] && own[v] != Health::Down {
                        reach[v] = true;
                        stack.push(v);
                    }
                }
            }
        }
        let before = self.health.clone();
        for i in 0..n {
            self.health[i] = if own[i] == Health::Down {
                Health::Down
            } else if !reach[i] && !self.cores.contains(&i) {
                Health::Unreachable
            } else {
                own[i]
            };
        }
        // Impacts and automatic recoveries, one grouped line each.
        let cut: Vec<usize> = (0..n).filter(|&i| self.health[i] == Health::Unreachable && before[i] != Health::Unreachable).collect();
        let back: Vec<usize> = (0..n).filter(|&i| before[i] == Health::Unreachable && self.health[i] != Health::Unreachable).collect();
        if !cut.is_empty() {
            let names: Vec<&str> = cut.iter().take(3).map(|&i| self.nodes[i].name.as_str()).collect();
            let more = if cut.len() > 3 { format!(" +{}", cut.len() - 3) } else { String::new() };
            let text = format!("{} service(s) unreachable: {}{more}. They depend on a path through a down node.", cut.len(), names.join(", "));
            self.say(Cat::Cascade, "⛔", text, Some(cut[0]), None, Some("unreach".into()));
        }
        if !back.is_empty() {
            let text = format!("{} service(s) reachable again on their own: the path is back", back.len());
            if back.len() >= 3 {
                self.note(true, format!("One repair brought {} cut-off services back at once", back.len()));
            }
            self.say(Cat::Recover, "↺", text, Some(back[0]), None, Some("reach".into()));
        }
        let mut total = 0.0;
        let mut up = 0.0;
        for i in 0..n {
            let w = *self.rules.criticality.get(&self.nodes[i].kind).unwrap_or(&1.0);
            total += w;
            up += w * match self.health[i] {
                Health::Online => 1.0,
                Health::Degraded => 0.5,
                // Still running locally, but nobody can reach it.
                Health::Unreachable => 0.3,
                Health::Down => 0.0,
            };
        }
        self.health_now = if total > 0.0 { up / total * 100.0 } else { 100.0 };
        self.version = self.version.wrapping_add(1);
    }

    // ------------------------------------------------------------ orders

    /// Can this worker do this action at all?
    pub fn can_do(&self, worker: usize, action: &str) -> bool {
        let d = &self.rules.workers[self.workers[worker].def];
        action == "diagnose" || self.rules.action(action).is_some_and(|a| d.can_heavy || !a.heavy)
    }

    /// Seconds of work for this worker.
    pub fn work_secs(&self, worker: usize, action: &str, node: usize) -> f32 {
        let d = &self.rules.workers[self.workers[worker].def];
        let (base, mult) = if action == "diagnose" {
            (self.rules.diagnose_secs, d.diagnose_work.unwrap_or(d.work))
        } else {
            let (secs, maint, hw) = self.rules.action(action).map_or((3.0, false, false), |a| (a.secs, a.maintenance, a.hardware));
            let mult = if hw && d.hardware_work.is_some() {
                d.hardware_work.unwrap()
            } else if maint {
                d.maintenance_work.unwrap_or(d.work)
            } else {
                d.work
            };
            (secs, mult)
        };
        let cold = if self.health[node] == Health::Down { 1.5 } else { 1.0 };
        base * mult * cold
    }

    /// Seconds until this worker could start on `node` (current queue + travel).
    pub fn eta(&self, worker: usize, node: usize) -> f32 {
        let w = &self.workers[worker];
        let d = &self.rules.workers[w.def];
        let mut t = w.rest.max(0.0);
        let mut at = w.at;
        let mut pos = w.pos;
        for task in &w.queue {
            t += self.travel(w.def, pos, at, task.node) + task.left;
            at = task.node;
            pos = self.nodes[at].pos;
        }
        if w.queue.is_empty() && w.walking() {
            if let Some(&last) = w.path.back() {
                t += dist(pos, self.nodes[last].pos) / d.speed;
                at = last;
                pos = self.nodes[last].pos;
            }
        }
        t + self.travel(w.def, pos, at, node)
    }

    fn travel(&self, def: usize, pos: [f32; 2], from: usize, to: usize) -> f32 {
        let d = &self.rules.workers[def];
        if d.flies {
            return dist(pos, self.nodes[to].pos) / d.speed;
        }
        match self.route(from, to) {
            Some(p) => p.windows(2).map(|w| dist(self.nodes[w[0]].pos, self.nodes[w[1]].pos)).sum::<f32>() / d.speed,
            None => dist(pos, self.nodes[to].pos) / self.rules.hike_speed,
        }
    }

    /// Best worker for a job: can do it, soonest start.
    pub fn pick_worker(&self, node: usize, action: Option<&str>) -> Option<usize> {
        (0..self.workers.len())
            .filter(|&w| self.workers[w].active)
            .filter(|&w| action.is_none_or(|a| self.can_do(w, a)))
            .filter(|&w| self.workers[w].queue.len() < self.rules.queue_max)
            .min_by(|&a, &b| self.eta(a, node).total_cmp(&self.eta(b, node)))
    }

    /// Order a worker (or the best one) to do `action` at `node` (None: just go).
    /// Returns the worker used.
    pub fn order(&mut self, worker: Option<usize>, node: usize, action: Option<&str>) -> Result<usize, String> {
        if self.phase != Phase::Running {
            return Err("No shift running".into());
        }
        let w = match worker {
            Some(w) => w,
            None => self.pick_worker(node, action).ok_or("Everyone's queue is full")?,
        };
        if !self.workers[w].active {
            return Err(format!("{} is off shift: call them in (K) for {} 🍌", self.workers[w].name, self.rules.call_cost));
        }
        if let Some(a) = action {
            if !self.can_do(w, a) {
                let d = &self.rules.workers[self.workers[w].def];
                return Err(format!("{} can't do that: too heavy for a {}", d.name, d.species));
            }
        }
        if self.workers[w].queue.len() >= self.rules.queue_max {
            return Err(format!("{}'s queue is full", self.workers[w].name));
        }
        let fault = self.faults[node].as_ref().map(|f| f.id);
        let total = action.map_or(0.0, |a| self.work_secs(w, a, node));
        let eta = self.eta(w, node);
        let task = Task { node, action: action.map(String::from), fault, left: total, total, started: false };
        let first = self.workers[w].queue.is_empty();
        self.workers[w].queue.push_back(task);
        if first {
            self.head_for(w, node);
        }
        let (name, emoji) = (self.workers[w].name.clone(), self.workers[w].emoji.clone());
        let what = match action {
            Some("diagnose") => "diagnose".to_string(),
            Some(a) => self.rules.action(a).map_or(a.to_string(), |d| d.label.to_lowercase()),
            None => "check in".to_string(),
        };
        let queued = if first { String::new() } else { format!(" (queued #{})", self.workers[w].queue.len()) };
        self.say(
            Cat::Dispatch,
            &emoji,
            format!("{name} → {} to {what}. ETA {:.0}s{queued}", self.nodes[node].name, eta),
            Some(node),
            Some(w),
            None,
        );
        Ok(w)
    }

    /// Cancel a worker's last queued (not started) task.
    pub fn cancel_last(&mut self, w: usize) -> bool {
        let wk = &mut self.workers[w];
        let can = wk.queue.len() > 1 || wk.queue.front().is_some_and(|t| !t.started);
        if !can {
            return false;
        }
        let t = wk.queue.pop_back().unwrap();
        if wk.queue.is_empty() {
            // Stop after the leg in progress.
            let next = wk.path.front().copied();
            wk.path.clear();
            if let Some(n) = next {
                wk.path.push_back(n);
            }
        }
        let name = wk.name.clone();
        let emoji = wk.emoji.clone();
        self.say(Cat::Dispatch, &emoji, format!("{name}: cancelled the job at {}", self.nodes[t.node].name), Some(t.node), Some(w), None);
        true
    }

    /// Spend bananas: the monkey swings on a vine straight to `node`.
    pub fn swing(&mut self, w: usize, node: usize) -> Result<(), String> {
        if self.bananas < self.rules.swing_cost {
            return Err("Not enough bananas".into());
        }
        if self.rules.workers[self.workers[w].def].flies {
            return Err("Already flying".into());
        }
        self.bananas -= self.rules.swing_cost;
        let wk = &mut self.workers[w];
        wk.queue.retain(|t| t.node == node);
        wk.path.clear();
        wk.path.push_back(node);
        wk.origin = Some(wk.pos);
        wk.leg = 0.0;
        wk.leg_len = dist(wk.pos, self.nodes[node].pos).max(1.0);
        wk.swing = true;
        wk.overland = false;
        let (name, emoji) = (wk.name.clone(), wk.emoji.clone());
        self.say(Cat::Dispatch, &emoji, format!("{name} swings on a vine to {} (🍌 -{})", self.nodes[node].name, self.rules.swing_cost), Some(node), Some(w), None);
        Ok(())
    }

    /// Spend bananas: end a worker's rest now.
    pub fn rally(&mut self, w: usize) -> Result<(), String> {
        if !self.workers[w].resting() {
            return Err("Not resting".into());
        }
        if self.bananas < self.rules.rally_cost {
            return Err("Not enough bananas".into());
        }
        self.bananas -= self.rules.rally_cost;
        let d = &self.rules.workers[self.workers[w].def];
        let stamina = d.stamina;
        let wk = &mut self.workers[w];
        wk.rest = 0.0;
        wk.stamina = stamina;
        let (name, emoji) = (wk.name.clone(), wk.emoji.clone());
        self.say(Cat::Rest, &emoji, format!("{name} eats a banana and gets back to work (🍌 -{})", self.rules.rally_cost), None, Some(w), None);
        Ok(())
    }

    fn head_for(&mut self, w: usize, to: usize) {
        let flies = self.rules.workers[self.workers[w].def].flies;
        let route = if flies { None } else { self.route(self.workers[w].path.front().copied().unwrap_or(self.workers[w].at), to) };
        let wk = &mut self.workers[w];
        let walking = wk.path.front().copied();
        let from = walking.unwrap_or(wk.at);
        wk.path.clear();
        if let Some(n) = walking {
            wk.path.push_back(n);
        }
        if from != to {
            match route {
                Some(p) => wk.path.extend(p.into_iter().skip(1)),
                None => wk.path.push_back(to),
            }
        }
        if walking.is_none() {
            self.begin_leg(w);
        }
    }

    fn begin_leg(&mut self, w: usize) {
        let flies = self.rules.workers[self.workers[w].def].flies;
        let at = self.workers[w].at;
        let next = self.workers[w].path.front().copied();
        let linked = next.is_some_and(|n| self.neighbours[at].contains(&n));
        let wk = &mut self.workers[w];
        wk.leg = 0.0;
        wk.swing = false;
        wk.origin = None;
        if let Some(n) = next {
            wk.leg_len = dist(self.nodes[at].pos, self.nodes[n].pos).max(1.0);
            wk.overland = !flies && !linked;
        }
    }

    /// Shortest walk over links; down nodes can't be crossed (only reached).
    pub fn route(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        let n = self.nodes.len();
        let mut d = vec![f32::INFINITY; n];
        let mut prev = vec![usize::MAX; n];
        let mut done = vec![false; n];
        d[from] = 0.0;
        while let Some(u) = (0..n).filter(|&i| !done[i] && d[i].is_finite()).min_by(|&a, &b| d[a].total_cmp(&d[b])) {
            if u == to {
                break;
            }
            done[u] = true;
            if u != from && self.health[u] == Health::Down {
                continue;
            }
            for &v in &self.neighbours[u] {
                let nd = d[u] + dist(self.nodes[u].pos, self.nodes[v].pos);
                if nd < d[v] {
                    d[v] = nd;
                    prev[v] = u;
                }
            }
        }
        if !d[to].is_finite() {
            return None;
        }
        let mut path = vec![to];
        while *path.last().unwrap() != from {
            path.push(prev[*path.last().unwrap()]);
        }
        path.reverse();
        Some(path)
    }

    // ------------------------------------------------------------ step

    pub fn step(&mut self, dt: f32) {
        if self.phase != Phase::Running {
            return;
        }
        self.clock += dt;
        for w in 0..self.workers.len() {
            if self.workers[w].active {
                self.step_worker(w, dt);
            }
        }
        self.step_faults(dt);
        self.director();
        // SLA: time-averaged network health.
        self.sla_sum += self.health_now * dt;
        self.sla = if self.clock > 0.0 { self.sla_sum / self.clock } else { 100.0 };
        let open = self.open_faults() as u32;
        self.stats.max_open = self.stats.max_open.max(open);
        if self.health_now < 50.0 {
            self.stats.critical_secs += dt;
        }
        if self.health_now < self.rules.outage_floor {
            self.outage_for += dt;
        } else {
            self.outage_for = 0.0;
        }
        if self.outage_for >= self.rules.outage_secs && !self.level_def().practice {
            self.phase = Phase::Collapsed;
            self.note(false, format!("Network health stayed under {:.0}% for {:.0}s: the shift ended early", self.rules.outage_floor, self.rules.outage_secs));
            self.say(Cat::Critical, "💥", "Major outage: the network stayed below the outage floor too long. Shift over.".into(), None, None, None);
        } else if self.clock >= self.rules.shift_secs {
            self.phase = Phase::Done;
            let met = self.sla >= self.rules.sla_target;
            self.say(
                Cat::Shift,
                "🌅",
                format!("17:00 shift complete. SLA {:.1}% ({} the {:.0}% target).", self.sla, if met { "met" } else { "missed" }, self.rules.sla_target),
                None,
                None,
                None,
            );
        }
    }

    fn step_worker(&mut self, w: usize, dt: f32) {
        let def = self.rules.workers[self.workers[w].def].clone();
        {
            let wk = &mut self.workers[w];
            if wk.rest > 0.0 {
                wk.rest -= dt;
                if wk.rest <= 0.0 {
                    wk.rest = 0.0;
                    wk.stamina = def.stamina;
                    let (name, emoji) = (wk.name.clone(), wk.emoji.clone());
                    self.say(Cat::Rest, &emoji, format!("{name} is rested and available"), None, Some(w), Some(format!("rest{w}")));
                }
                self.workers[w].idle_secs += dt;
                return;
            }
        }
        // Walk / fly.
        let mut left = dt;
        while left > 0.0 {
            let Some(&next) = self.workers[w].path.front() else { break };
            let at = self.workers[w].at;
            let wk = &self.workers[w];
            let speed = if wk.swing {
                self.rules.swing_speed
            } else if wk.overland {
                self.rules.hike_speed
            } else {
                let slow = !def.flies && (self.health[next] == Health::Degraded || self.health[at] == Health::Degraded);
                def.speed * if slow { 0.6 } else { 1.0 }
            };
            let step = speed * left / wk.leg_len;
            if wk.leg + step < 1.0 {
                self.workers[w].leg += step;
                left = 0.0;
            } else {
                left -= (1.0 - wk.leg) * wk.leg_len / speed;
                let wk = &mut self.workers[w];
                wk.at = next;
                wk.path.pop_front();
                self.begin_leg(w);
                if self.workers[w].path.is_empty() {
                    let (name, emoji) = (self.workers[w].name.clone(), self.workers[w].emoji.clone());
                    let text = format!("{name} arrived at {}", self.nodes[next].name);
                    self.say(Cat::Arrive, &emoji, text, Some(next), Some(w), None);
                }
            }
        }
        let wk = &mut self.workers[w];
        wk.pos = match wk.path.front() {
            Some(&n) => lerp(wk.origin.unwrap_or(self.nodes[wk.at].pos), self.nodes[n].pos, wk.leg),
            None => self.nodes[wk.at].pos,
        };
        if wk.walking() {
            wk.busy_secs += dt;
            return;
        }
        // Work.
        let Some(task) = wk.queue.front().cloned() else {
            wk.idle_secs += dt;
            return;
        };
        if task.node != wk.at {
            self.head_for(w, task.node);
            return;
        }
        let wk = &mut self.workers[w];
        wk.busy_secs += dt;
        if task.action.is_none() {
            wk.queue.pop_front();
            self.next_task(w);
            return;
        }
        if !task.started {
            wk.queue.front_mut().unwrap().started = true;
            let (name, emoji) = (wk.name.clone(), wk.emoji.clone());
            let a = task.action.clone().unwrap();
            let label = if a == "diagnose" { "Diagnosing".to_string() } else { self.rules.action(&a).map_or(a.clone(), |d| d.label.clone()) };
            let text = format!("{name}: {label} at {} · {:.0}s", self.nodes[task.node].name, task.total);
            self.say(Cat::Work, &emoji, text, Some(task.node), Some(w), None);
        }
        let wk = &mut self.workers[w];
        let t = wk.queue.front_mut().unwrap();
        t.left -= dt;
        if t.left > 0.0 {
            return;
        }
        let task = wk.queue.pop_front().unwrap();
        wk.stamina = wk.stamina.saturating_sub(1);
        self.finish(w, &task);
        if self.workers[w].stamina == 0 {
            self.workers[w].rest = def.rest;
            let (name, emoji) = (self.workers[w].name.clone(), self.workers[w].emoji.clone());
            self.say(Cat::Rest, &emoji, format!("{name} needs a break: resting {:.0}s (🍌 rally ends it)", def.rest), None, Some(w), None);
        }
        self.next_task(w);
    }

    fn next_task(&mut self, w: usize) {
        if let Some(n) = self.workers[w].queue.front().map(|t| t.node) {
            self.head_for(w, n);
        }
    }

    /// Apply a finished action.
    fn finish(&mut self, w: usize, task: &Task) {
        let node = task.node;
        let action = task.action.clone().unwrap_or_default();
        let (wname, wemoji) = (self.workers[w].name.clone(), self.workers[w].emoji.clone());
        let nname = self.nodes[node].name.clone();
        let Some(f) = self.faults[node].clone().filter(|f| Some(f.id) == task.fault || task.fault.is_none()) else {
            self.say(Cat::Work, &wemoji, format!("{wname}: nothing left to fix at {nname}"), Some(node), Some(w), None);
            return;
        };
        let Some(def) = self.rules.fault(&f.kind).cloned() else { return };
        let alabel = self.rules.action(&action).map_or(action.clone(), |a| a.label.clone());
        if action == "diagnose" {
            if let Some(slot) = self.faults[node].as_mut() {
                slot.diagnosed = true;
            }
            let learned = self.learn(&def.concept, format!("{} at {nname}", def.name));
            let fixes = self.fixes_for(node);
            let fix_labels: Vec<String> = fixes.iter().map(|k| self.rules.action(k).map_or(k.clone(), |a| a.label.clone())).collect();
            self.say(
                Cat::Learn,
                "🔍",
                format!("{wname} diagnosed {nname}: {}. Root-cause fix: {}", def.name, fix_labels.join(" or ")),
                Some(node),
                Some(w),
                None,
            );
            self.outcome = Some(Outcome {
                node,
                ok: true,
                title: format!("DIAGNOSED: {nname}"),
                cause: format!("{}: {}", def.name, def.symptom),
                action: format!("Fix: {}", fix_labels.join(" or ")),
                learned,
                t: self.clock,
            });
            return;
        }
        let opt = f.options.iter().position(|o| *o == action);
        // Warnings: prevention.
        if f.is_warning() {
            let prevent = def.warning.as_ref().is_some_and(|wd| wd.prevent.contains(&action));
            if prevent {
                self.faults[node] = None;
                self.stats.prevented += 1;
                self.reward(true, 0.0);
                let learned = self.learn(&def.concept, format!("prevented {} at {nname}", def.name));
                self.say(Cat::Resolved, "✅", format!("{wname} prevented {} at {nname}", def.name), Some(node), Some(w), None);
                self.note(true, format!("Prevented {} at {nname} before it hit ({alabel})", def.name));
                self.outcome = Some(Outcome { node, ok: true, title: format!("PREVENTED: {nname}"), cause: def.warning.as_ref().unwrap().text.clone(), action: alabel, learned, t: self.clock });
                self.after_resolve(f.id);
                self.recompute();
            } else {
                self.no_effect(w, node, opt, &alabel, "That doesn't address what the warning is about.".into());
            }
            return;
        }
        match def.responses.get(&action).cloned() {
            Some(r) if r.effect == "fix" => {
                self.faults[node] = None;
                self.stats.resolved += 1;
                self.stats.recovery_total += self.clock - f.since;
                *self.stats.fixes_by.entry(action.clone()).or_default() += 1;
                self.reward(false, f.ttl.max(0.0));
                let learned = self.learn(&def.concept, format!("{} at {nname}", def.name));
                self.say(Cat::Resolved, "✅", format!("{nname} restored: {}. {}", alabel, r.note), Some(node), Some(w), None);
                let full = def.ttl * self.level_def().ttl_mult;
                if f.diagnosed {
                    self.note(true, format!("Diagnosed {} at {nname} first, then fixed the cause", def.name));
                } else if f.stage != Stage::Down && f.ttl > full * 0.5 {
                    self.note(true, format!("Fixed {} at {nname} fast, with {:.0}s to spare", def.name, f.ttl));
                } else if f.stage == Stage::Down {
                    self.note(true, format!("Brought {nname} back from an outage ({})", def.name));
                }
                self.outcome = Some(Outcome { node, ok: true, title: format!("RESOLVED: {nname}"), cause: def.name.clone(), action: format!("{alabel}. {}", r.note), learned, t: self.clock });
                self.after_resolve(f.id);
                self.recompute();
            }
            Some(r) if r.effect == "mitigate" => {
                if let Some(slot) = self.faults[node].as_mut() {
                    slot.mitigated = r.secs;
                    slot.mitigation = Some(action.clone());
                    if slot.stage == Stage::Down {
                        slot.stage = Stage::Incident;
                        slot.ttl = slot.ttl.max(20.0);
                    }
                }
                self.stats.mitigations += 1;
                self.note(true, format!("Bought time at {nname} with {alabel}"));
                self.say(Cat::Mitigated, "⏱", format!("{nname}: {alabel} bought {:.0}s. {}", r.secs, r.note), Some(node), Some(w), None);
                self.outcome = Some(Outcome {
                    node,
                    ok: true,
                    title: format!("MITIGATED: {nname} (temporary)"),
                    cause: def.name.clone(),
                    action: format!("{alabel}: {}", r.note),
                    learned: None,
                    t: self.clock,
                });
                self.recompute();
            }
            Some(r) => self.no_effect(w, node, opt, &alabel, r.note),
            None => {
                let hint = format!("It doesn't touch the cause: {}", def.symptom);
                self.no_effect(w, node, opt, &alabel, hint)
            }
        }
    }

    fn no_effect(&mut self, w: usize, node: usize, opt: Option<usize>, alabel: &str, why: String) {
        if let (Some(i), Some(slot)) = (opt, self.faults[node].as_mut()) {
            slot.tried |= 1 << i;
        }
        self.combo = 0;
        self.stats.no_effect += 1;
        let nname = self.nodes[node].name.clone();
        let (wname, wemoji) = (self.workers[w].name.clone(), self.workers[w].emoji.clone());
        self.say(Cat::NoEffect, &wemoji, format!("{wname}: {alabel} done, {nname} unchanged. {why}"), Some(node), Some(w), None);
        let cause = self.fault_def(node).map_or(String::new(), |d| d.name.clone());
        self.note(false, format!("{alabel} on {cause} at {nname} didn't help: {why}"));
        self.outcome = Some(Outcome { node, ok: false, title: format!("NO EFFECT: {nname}"), cause, action: format!("{alabel}: {why}"), learned: None, t: self.clock });
    }

    fn reward(&mut self, prevent: bool, ttl_left: f32) {
        self.combo += 1;
        let mult = (1.0 + 0.25 * (self.combo - 1) as f32).min(2.0);
        let base = if prevent { self.rules.points_prevent } else { self.rules.points_fix + ttl_left * self.rules.points_per_ttl_sec };
        let total = (base * mult) as u32;
        self.score += total;
        if prevent {
            self.stats.points_prevent += base as u32;
        } else {
            self.stats.points_fix += base as u32;
        }
        self.stats.points_combo += total.saturating_sub(base as u32);
        self.bananas += if prevent { self.rules.banana_prevent } else { self.rules.banana_fix };
    }

    fn after_resolve(&mut self, id: u32) {
        self.grace_until = self.clock + self.rules.director.grace;
        self.tutorial_open.retain(|&x| x != id);
    }

    fn step_faults(&mut self, dt: f32) {
        let n = self.nodes.len();
        let mut changed = false;
        let mut spread = Vec::new();
        for i in 0..n {
            let Some(mut f) = self.faults[i].clone() else { continue };
            let def = self.rules.fault(&f.kind).cloned();
            let name = self.nodes[i].name.clone();
            if f.mitigated > 0.0 {
                f.mitigated -= dt;
                if f.mitigated <= 0.0 {
                    f.mitigated = 0.0;
                    f.ttl = f.ttl.max(15.0);
                    changed = true;
                    let fname = def.as_ref().map_or(f.kind.clone(), |d| d.name.clone());
                    self.say(Cat::Incident, "⏱", format!("{name}: the mitigation wore off, {fname} is back"), Some(i), None, None);
                    self.note(false, format!("The mitigation at {name} wore off before anyone came back with the real fix for {fname}"));
                }
                self.faults[i] = Some(f);
                continue;
            }
            match f.stage {
                Stage::Warning | Stage::Incident => {
                    f.ttl -= dt;
                    if f.ttl <= 0.0 {
                        changed = true;
                        if f.stage == Stage::Warning {
                            // It happens. A scheduled deploy may turn out to carry a migration.
                            let mut kind = f.kind.clone();
                            if kind == "deploy" && self.rand() < 0.4 && self.rules.faults.contains_key("migration") {
                                kind = "migration".into();
                            }
                            let d = self.rules.fault(&kind).cloned().unwrap();
                            f.kind = kind;
                            f.stage = Stage::Incident;
                            f.ttl = d.ttl * self.level_def().ttl_mult;
                            f.options = self.options_for(&d, false);
                            f.tried = 0;
                            f.diagnosed = false;
                            self.stats.escalated_warnings += 1;
                            self.note(false, format!("The warning at {name} wasn't acted on in time and became {}", d.name));
                            self.say(Cat::Incident, &d.emoji, format!("{name}: the warning became an incident: {}. {}", d.name, d.symptom), Some(i), None, None);
                        } else {
                            f.stage = Stage::Down;
                            f.down_for = 0.0;
                            self.stats.outages += 1;
                            let cut = self.impact_if_down(i);
                            let fname0 = def.as_ref().map_or(f.kind.clone(), |d| d.name.clone());
                            let extra = if cut > 0 { format!(", cutting off {cut} other service(s)") } else { String::new() };
                            self.note(false, format!("{name} went down ({fname0}) when its Time to Loss ran out{extra}"));
                            let crit = i == self.core || self.rules.criticality.get(&self.nodes[i].kind).is_some_and(|w| *w >= 3.0);
                            let fname = def.as_ref().map_or(f.kind.clone(), |d| d.name.clone());
                            self.say(
                                if crit { Cat::Critical } else { Cat::Incident },
                                "⛔",
                                format!("{name} is DOWN ({fname}). It still needs the same fix, now 1.5× slower."),
                                Some(i),
                                None,
                                None,
                            );
                        }
                    }
                }
                Stage::Down => {
                    f.down_for += dt;
                    if def.as_ref().is_some_and(|d| d.spreads) && !f.spread_done && f.down_for >= self.rules.director.spread_after && !self.level_def().practice {
                        f.spread_done = true;
                        spread.push((i, f.kind.clone(), f.root));
                    }
                }
            }
            self.faults[i] = Some(f);
        }
        for (from, kind, root) in spread {
            let healthy: Vec<usize> = self.neighbours[from].iter().copied().filter(|&j| self.faults[j].is_none() && self.health[j] == Health::Online).collect();
            if healthy.is_empty() {
                continue;
            }
            let j = healthy[self.below(healthy.len())];
            if self.raise(j, &kind, false, Some(root)) {
                self.stats.spreads += 1;
                let (a, b) = (self.nodes[from].name.clone(), self.nodes[j].name.clone());
                self.note(false, format!("{kind} spread from {a} to {b} after {a} stayed down"));
            }
        }
        if changed {
            self.recompute();
        }
    }

    /// Decide whether anything new happens.
    fn director(&mut self) {
        let lvl = self.level_def().clone();
        let p = self.progress();
        // Calm-period notices: atmosphere and observation, no action needed.
        if self.clock >= self.next_notice {
            self.next_notice = self.clock + self.rules.director.notice_every * (0.7 + self.rand() * 0.6);
            if self.open_faults() == 0 && !self.rules.notices.is_empty() {
                let k = self.below(self.rules.notices.len());
                let notice = self.rules.notices[k].clone();
                let nodes: Vec<usize> = (0..self.nodes.len()).filter(|&i| notice.kinds.is_empty() || notice.kinds.contains(&self.nodes[i].kind)).collect();
                if !nodes.is_empty() {
                    let i = nodes[self.below(nodes.len())];
                    let text = notice.text.replace("{node}", &self.nodes[i].name);
                    self.say(Cat::Notice, "🔹", text, Some(i), None, None);
                }
            }
        }
        // The scripted opening.
        if lvl.tutorial && self.tutorial < self.rules.tutorial.len() {
            // A step left down for a while doesn't hold the lesson plan hostage.
            let faults = &self.faults;
            self.tutorial_open.retain(|&id| faults.iter().flatten().any(|f| f.id == id && !(f.stage == Stage::Down && f.down_for > 40.0)));
            if self.tutorial_open.is_empty() && self.clock >= self.grace_until.max(self.next_root) {
                loop {
                    let step = self.rules.tutorial[self.tutorial].clone();
                    self.tutorial += 1;
                    if let Some(node) = self.nodes.iter().position(|x| x.id == step.node) {
                        if self.raise(node, &step.fault, step.warning, None) {
                            let id = self.faults[node].as_ref().unwrap().id;
                            self.tutorial_open.push(id);
                        }
                    }
                    let together = self.rules.tutorial.get(self.tutorial).is_some_and(|s| s.with_previous);
                    if !together {
                        break;
                    }
                }
                self.next_root = self.clock + lvl.interval[0];
            }
            return;
        }
        if self.clock < self.next_root || self.clock < self.grace_until {
            return;
        }
        let max_roots = (lvl.max_roots[0] + (lvl.max_roots[1] - lvl.max_roots[0]) * p).floor().max(1.0) as usize;
        let interval = lvl.interval[0] + (lvl.interval[1] - lvl.interval[0]) * p;
        if self.active_roots() >= max_roots || self.health_now < self.rules.director.health_floor * 100.0 {
            // Not now: look again shortly.
            self.next_root = self.clock + 5.0;
            return;
        }
        self.next_root = self.clock + interval * (0.75 + self.rand() * 0.5);
        let warning = self.rand() < lvl.warning_share;
        self.spawn_root(warning, p);
    }

    fn spawn_root(&mut self, warning: bool, p: f32) -> bool {
        let free: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.faults[i].is_none() && self.health[i] == Health::Online).collect();
        let mut cands: Vec<(usize, String, f32)> = Vec::new();
        for &i in &free {
            for (k, d) in &self.rules.faults {
                if d.weight <= 0.0 || d.min_level > self.level || d.min_progress > p || (warning && d.warning.is_none()) {
                    continue;
                }
                if !d.kinds.is_empty() && !d.kinds.contains(&self.nodes[i].kind) {
                    continue;
                }
                // Critical infrastructure fails rarely, and never in training.
                let crit = *self.rules.criticality.get(&self.nodes[i].kind).unwrap_or(&1.0);
                if self.level == 0 && crit >= 4.0 {
                    continue;
                }
                cands.push((i, k.clone(), d.weight / crit));
            }
        }
        if cands.is_empty() {
            return false;
        }
        cands.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
        let total: f32 = cands.iter().map(|c| c.2).sum();
        let mut r = self.rand() * total;
        let pick = cands.iter().find(|c| {
            r -= c.2;
            r <= 0.0
        });
        let (node, kind, _) = pick.unwrap_or(&cands[0]).clone();
        self.raise(node, &kind, warning, None)
    }

    /// A storm cloud's lightning near a node: the director may allow a power fault.
    pub fn lightning(&mut self, node: usize) -> bool {
        if self.phase != Phase::Running || self.clock < self.grace_until || self.level_def().tutorial && self.tutorial < self.rules.tutorial.len() {
            return false;
        }
        let lvl = self.level_def();
        let max_roots = (lvl.max_roots[0] + (lvl.max_roots[1] - lvl.max_roots[0]) * self.progress()).floor().max(1.0) as usize;
        if self.active_roots() >= max_roots || !self.rules.faults.contains_key("power") {
            return false;
        }
        self.raise(node, "power", false, None)
    }

    /// End-of-shift summary lines.
    pub fn summary(&self) -> Vec<String> {
        let s = &self.stats;
        let avg = if s.resolved > 0 { s.recovery_total / s.resolved as f32 } else { 0.0 };
        let mut best: Vec<(&String, &u32)> = s.fixes_by.iter().collect();
        best.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        let best = best.first().map_or("—".to_string(), |(k, n)| format!("{} (×{n})", self.rules.action(k).map_or(k.as_str(), |a| a.label.as_str())));
        let concepts: Vec<&str> = self.learned.iter().filter_map(|(k, _)| self.rules.concepts.get(k).map(|c| c.name.as_str())).collect();
        let still_open = self.open_faults();
        let tip = if s.resolved + s.prevented == 0 {
            "Nothing got fixed this shift: press Space to jump to an alert, then pick a response (or I to diagnose)."
        } else if still_open > 2 {
            "Several problems were still open at the end: send the crew to the oldest outages first."
        } else if s.escalated_warnings > 0 {
            "Act on warnings before they escalate: prevention is cheaper than repair."
        } else if s.no_effect > s.resolved / 2 + 1 {
            "Diagnose (🔍) before acting when unsure: it shows which option fixes the cause."
        } else if s.outages > 2 {
            "Fix nodes that others route through first: a down hub takes its dependents with it."
        } else if s.mitigations > s.resolved {
            "Mitigations buy time; come back with the permanent fix before they wear off."
        } else {
            "Nice work. Next shift: more advanced faults."
        };
        vec![
            format!("Network availability (SLA): {:.1}% (target {:.0}%)", self.sla, self.rules.sla_target),
            format!("Incidents resolved: {}  ·  prevented: {}  ·  mitigations: {}", s.resolved, s.prevented, s.mitigations),
            format!("Outages: {}  ·  spread events: {}  ·  avg recovery: {:.0}s", s.outages, s.spreads, avg),
            format!("Most effective action: {best}"),
            format!("Still open at the end: {still_open}"),
            format!("Concepts learned: {}", if concepts.is_empty() { "—".into() } else { concepts.join(", ") }),
            format!("Tip: {tip}"),
        ]
    }
}
