//! Headless harness for the Lucky Monkey simulation: compiles game/src/sim.rs
//! on its own (no Bevy), loads the real data files, and offers bot players.

#[path = "../../src/sim.rs"]
pub mod sim;

use serde_json::Value;
use sim::{Phase, Rules, Sim, SimNode, Stage};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../data");

pub fn rules() -> Rules {
    let s = std::fs::read_to_string(format!("{DATA}/tuning.json")).unwrap();
    serde_json::from_str(&s).expect("tuning.json parses as Rules")
}

/// Nodes and links exactly as the game builds them (screen positions via iso).
pub fn network() -> (Vec<SimNode>, Vec<(usize, usize)>) {
    let w: Value = serde_json::from_str(&std::fs::read_to_string(format!("{DATA}/world.json")).unwrap()).unwrap();
    let nodes = w["nodes"].as_array().unwrap();
    let out: Vec<SimNode> = nodes
        .iter()
        .map(|n| {
            let p = &n["position"];
            let (x, y, h) = (p["x"].as_f64().unwrap() as f32, p["y"].as_f64().unwrap() as f32, p["height"].as_f64().unwrap_or(0.0) as f32);
            SimNode {
                id: n["id"].as_str().unwrap().into(),
                name: n["name"].as_str().unwrap().into(),
                kind: n["type"].as_str().unwrap().into(),
                pos: [(x - y) * 32.0, -(x + y) * 16.0 + h * 16.0],
            }
        })
        .collect();
    let mut links = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        for c in n["connections"].as_array().into_iter().flatten() {
            if let Some(j) = out.iter().position(|o| o.id == c.as_str().unwrap()) {
                let l = (i.min(j), i.max(j));
                if i != j && !links.contains(&l) {
                    links.push(l);
                }
            }
        }
    }
    (out, links)
}

pub fn new_sim(seed: u64) -> Sim {
    let (n, l) = network();
    Sim::new(rules(), n, l, seed)
}

pub fn node(sim: &Sim, id: &str) -> usize {
    sim.nodes.iter().position(|n| n.id == id).unwrap_or_else(|| panic!("no node {id}"))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Policy {
    /// Never acts.
    Idle,
    /// Slow to notice, guesses among the offered options, one job at a time.
    Novice,
    /// Slow to notice, diagnoses first, one job at a time.
    Beginner,
    /// Fast, knows the fixes, uses every worker, prevents warnings.
    Expert,
}

#[derive(Debug, Default, Clone)]
pub struct Run {
    pub completed: bool,
    pub sla: f32,
    pub roots: u32,
    pub max_open: u32,
    pub critical_secs: f32,
    pub resolved: u32,
    pub prevented: u32,
    pub avg_recovery: f32,
    pub spreads: u32,
    pub outages: u32,
    pub busy: f32,
    pub idle: f32,
}

/// Play one shift with a bot. Steps at 10 Hz.
pub fn play(seed: u64, level: usize, policy: Policy) -> (Run, Sim) {
    let mut sim = new_sim(seed);
    sim.start(level);
    let dt = 0.1;
    let mut seen: std::collections::HashMap<u32, f32> = Default::default();
    let delay = match policy {
        Policy::Beginner | Policy::Novice => 8.0,
        _ => 1.0,
    };
    while sim.phase == Phase::Running {
        sim.step(dt);
        if policy == Policy::Idle {
            continue;
        }
        // Faults by urgency.
        let mut open: Vec<(usize, u32, f32, bool, bool)> = (0..sim.nodes.len())
            .filter(|&i| sim.visible(i))
            .map(|i| {
                let f = sim.faults[i].as_ref().unwrap();
                (i, f.id, if f.stage == Stage::Down { 0.0 } else { f.ttl }, f.diagnosed, f.mitigated > 0.0)
            })
            .collect();
        open.sort_by(|a, b| a.2.total_cmp(&b.2));
        for &(i, id, _, diagnosed, _) in &open {
            let first = *seen.entry(id).or_insert(sim.clock);
            if sim.clock - first < delay {
                continue;
            }
            let assigned = sim.workers.iter().any(|w| w.queue.iter().any(|t| t.node == i && t.fault == Some(id)));
            if assigned {
                continue;
            }
            match policy {
                Policy::Beginner => {
                    // One job at a time across the whole team.
                    if sim.workers.iter().any(|w| !w.queue.is_empty()) {
                        break;
                    }
                    let action = if diagnosed { sim.fixes_for(i).into_iter().next() } else { Some("diagnose".to_string()) };
                    if let Some(a) = action {
                        let _ = sim.order(None, i, Some(&a));
                    }
                    break;
                }
                Policy::Novice => {
                    if sim.workers.iter().any(|w| !w.queue.is_empty()) {
                        break;
                    }
                    let f = sim.faults[i].as_ref().unwrap();
                    let untried: Vec<String> = f.options.iter().enumerate().filter(|(k, _)| f.tried & (1 << k) == 0).map(|(_, o)| o.clone()).collect();
                    if !untried.is_empty() {
                        let k = (sim.rand() * untried.len() as f32) as usize % untried.len();
                        let _ = sim.order(None, i, Some(&untried[k]));
                    }
                    break;
                }
                Policy::Expert => {
                    let Some(a) = sim.fixes_for(i).into_iter().find(|a| (0..sim.workers.len()).any(|w| sim.can_do(w, a))) else { continue };
                    let free: Vec<usize> = (0..sim.workers.len()).filter(|&w| sim.workers[w].active && sim.workers[w].queue.len() < 2 && sim.can_do(w, &a)).collect();
                    if let Some(&w) = free.iter().min_by(|&&a2, &&b2| sim.eta(a2, i).total_cmp(&sim.eta(b2, i))) {
                        let _ = sim.order(Some(w), i, Some(&a));
                    }
                }
                Policy::Idle => {}
            }
        }
        if policy == Policy::Expert {
            for w in 0..sim.workers.len() {
                if sim.workers[w].resting() && sim.bananas >= sim.rules.rally_cost + 2 {
                    let _ = sim.rally(w);
                }
            }
        }
    }
    let s = &sim.stats;
    let run = Run {
        completed: sim.phase == Phase::Done,
        sla: sim.sla,
        roots: s.roots,
        max_open: s.max_open,
        critical_secs: s.critical_secs,
        resolved: s.resolved,
        prevented: s.prevented,
        avg_recovery: if s.resolved > 0 { s.recovery_total / s.resolved as f32 } else { 0.0 },
        spreads: s.spreads,
        outages: s.outages,
        busy: sim.workers.iter().map(|w| w.busy_secs).sum(),
        idle: sim.workers.iter().map(|w| w.idle_secs).sum(),
    };
    (run, sim)
}
