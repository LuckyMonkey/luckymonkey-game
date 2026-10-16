//! Behaviour tests for the simulation (spec tests A-J).
use lm_simtest::sim::{feed, push_event, Cat, Event, Health, Phase, Stage};
use lm_simtest::{new_sim, node, play, Policy};

fn run_until(sim: &mut lm_simtest::sim::Sim, max: f32, mut done: impl FnMut(&lm_simtest::sim::Sim) -> bool) -> f32 {
    let t0 = sim.clock;
    while sim.clock - t0 < max && !done(sim) {
        sim.step(0.05);
    }
    sim.clock - t0
}

/// A quiet sim: running, but the director held off (we stage faults by hand).
fn staged(seed: u64) -> lm_simtest::sim::Sim {
    let mut sim = new_sim(seed);
    sim.rules.director.levels[2].first = 1e9;
    sim.rules.director.notice_every = 1e9;
    sim.start(2);
    sim
}

#[test]
fn a_director_is_gentle_on_a_healthy_network() {
    for seed in 1..=10 {
        // Training shift: the first minute brings exactly one scripted problem.
        let mut sim = new_sim(seed);
        sim.start(1);
        let mut max_open = 0;
        while sim.clock < 60.0 {
            sim.step(0.1);
            max_open = max_open.max(sim.open_faults());
        }
        assert_eq!(max_open, 1, "seed {seed}: training opened {max_open} faults in the first minute");
        // Shift 2, nobody acting: root causes stay within the budget and no critical event comes early.
        let mut sim = new_sim(seed);
        sim.start(2);
        while sim.clock < 120.0 {
            sim.step(0.1);
            assert!(sim.open_roots() <= 3, "seed {seed}: {} roots open", sim.open_roots());
        }
        assert!(!sim.log.iter().any(|e| e.cat == Cat::Critical && e.t < 45.0), "seed {seed}: critical in the first 45s");
    }
}

#[test]
fn b_router_failure_is_one_root_cause_and_dependents_recover() {
    // The hub whose failure cuts off the most services.
    let probe = staged(1);
    let core = (0..probe.nodes.len())
        .max_by_key(|&i| {
            let mut s = staged(1);
            s.raise(i, "crash", false, None);
            if let Some(f) = s.faults[i].as_mut() {
                f.ttl = 0.01;
            }
            s.step(0.05);
            s.unreachable()
        })
        .unwrap();
    let mut sim = staged(1);
    assert!(sim.raise(core, "crash", false, None));
    run_until(&mut sim, 200.0, |s| s.faults[core].as_ref().is_some_and(|f| f.stage == Stage::Down));
    assert_eq!(sim.health[core], Health::Down);
    let cut = sim.unreachable();
    eprintln!("hub {} down: {cut} services unreachable", sim.nodes[core].name);
    assert!(cut >= 2, "only {cut} dependents unreachable");
    assert_eq!(sim.open_faults(), 1, "dependents must not become separate incidents");
    sim.order(None, core, Some("restart")).unwrap();
    run_until(&mut sim, 120.0, |s| s.faults[core].is_none());
    assert!(sim.faults[core].is_none(), "router not fixed");
    assert_eq!(sim.unreachable(), 0);
    assert!(sim.health.iter().all(|h| *h == Health::Online));
    assert!(sim.log.iter().any(|e| e.cat == Cat::Recover), "no automatic recovery event");
}

#[test]
fn c_restart_mitigates_a_leak_without_fixing_it() {
    let mut sim = staged(2);
    let n = node(&sim, "server-hub");
    assert!(sim.raise(n, "leak", false, None));
    sim.order(None, n, Some("restart")).unwrap();
    run_until(&mut sim, 60.0, |s| s.faults[n].as_ref().is_some_and(|f| f.mitigated > 0.0));
    let f = sim.faults[n].as_ref().expect("leak must remain after a restart");
    assert!(f.mitigated > 0.0);
    assert_eq!(sim.health[n], Health::Online, "service restored temporarily");
    run_until(&mut sim, 90.0, |s| s.faults[n].as_ref().is_some_and(|f| f.mitigated <= 0.0));
    assert_eq!(sim.health[n], Health::Degraded, "the leak is back once the mitigation wears off");
    sim.order(None, n, Some("patch")).unwrap();
    run_until(&mut sim, 90.0, |s| s.faults[n].is_none());
    assert!(sim.faults[n].is_none(), "patch fixes the root cause");
}

#[test]
fn d_only_relevant_actions_fix_a_fault() {
    let mut sim = staged(3);
    let n = node(&sim, "logical-lagoon");
    assert!(sim.raise(n, "dhcp", false, None));
    sim.order(None, n, Some("restart")).unwrap();
    run_until(&mut sim, 60.0, |s| s.log.iter().any(|e| e.cat == Cat::NoEffect));
    assert!(sim.faults[n].is_some(), "restart must not fix DHCP exhaustion");
    let note = sim.log.iter().rev().find(|e| e.cat == Cat::NoEffect).unwrap();
    assert!(note.text.contains("addresses"), "explains why: {}", note.text);
    sim.order(None, n, Some("expandpool")).unwrap();
    run_until(&mut sim, 60.0, |s| s.faults[n].is_none());
    assert!(sim.faults[n].is_none());
}

#[test]
fn e_travel_takes_the_expected_time() {
    let mut sim = staged(4);
    let to = node(&sim, "cdn-canopy");
    let mango = 0;
    let eta = sim.eta(mango, to);
    sim.order(Some(mango), to, None).unwrap();
    let took = run_until(&mut sim, 300.0, |s| s.workers[mango].at == to && !s.workers[mango].walking());
    assert!((took - eta).abs() < 0.5, "eta {eta:.1}s, took {took:.1}s");
    assert!(sim.log.iter().any(|e| e.cat == Cat::Arrive && e.node == Some(to)));
}

#[test]
fn f_worker_activity_reaches_the_event_stream() {
    let mut sim = staged(5);
    let n = node(&sim, "terminal-01");
    sim.raise(n, "crash", false, None);
    sim.order(Some(0), n, Some("restart")).unwrap();
    run_until(&mut sim, 120.0, |s| s.faults[n].is_none());
    let cats: Vec<Cat> = sim.log.iter().filter(|e| e.worker == Some(0) || e.cat == Cat::Resolved).map(|e| e.cat).collect();
    let order = [Cat::Dispatch, Cat::Arrive, Cat::Work, Cat::Resolved];
    let mut k = 0;
    for c in cats {
        if k < order.len() && c == order[k] {
            k += 1;
        }
    }
    assert_eq!(k, order.len(), "missing dispatch/arrive/work/resolved in order");
}

#[test]
fn g_time_to_loss_tracks_the_next_consequence() {
    let mut sim = staged(6);
    let a = node(&sim, "terminal-01");
    let b = node(&sim, "server-hub");
    sim.raise(a, "crash", false, None);
    sim.raise(b, "heat", false, None);
    let d = sim.deadline().unwrap();
    let want = if sim.faults[a].as_ref().unwrap().ttl < sim.faults[b].as_ref().unwrap().ttl { a } else { b };
    assert_eq!(d.node, want);
    let clock = sim.clock;
    for _ in 0..50 {
        sim.step(0.1);
    }
    let d2 = sim.deadline().unwrap();
    assert!((d.secs - d2.secs - 5.0).abs() < 0.05, "TTL moves with time");
    assert!((sim.clock - clock - 5.0).abs() < 0.01, "shift clock is independent");
    // Calm: no deadline at all.
    let mut calm = staged(7);
    calm.step(1.0);
    assert!(calm.deadline().is_none());
}

#[test]
fn h_event_flood_is_grouped_and_urgent_stays_visible() {
    let mut log = Vec::new();
    for i in 0..30 {
        push_event(&mut log, Event { t: i as f32 * 0.1, cat: Cat::Cascade, icon: "🐘".into(), text: format!("hit {i}"), node: None, worker: None, group: Some("ddos".into()), count: 1 });
    }
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].count, 30);
    push_event(&mut log, Event { t: 4.0, cat: Cat::Incident, icon: "🚨".into(), text: "DNS down".into(), node: None, worker: None, group: None, count: 1 });
    for i in 0..40 {
        push_event(&mut log, Event { t: 4.0 + i as f32 * 0.05, cat: Cat::Arrive, icon: "🐵".into(), text: format!("chatter {i}"), node: None, worker: Some(0), group: None, count: 1 });
    }
    let rows = feed(&log, 6.0, 8, 30.0);
    assert!(rows.len() <= 8);
    assert!(rows.iter().any(|e| e.text == "DNS down"), "urgent incident buried by chatter");
}

#[test]
fn i_same_seed_same_shift() {
    let (a, sa) = play(42, 3, Policy::Expert);
    let (b, sb) = play(42, 3, Policy::Expert);
    assert_eq!(a.roots, b.roots);
    assert_eq!(sa.score, sb.score);
    let la: Vec<&String> = sa.log.iter().map(|e| &e.text).collect();
    let lb: Vec<&String> = sb.log.iter().map(|e| &e.text).collect();
    assert_eq!(la, lb);
}

#[test]
fn j_shift_ends_cleanly_with_real_stats() {
    let (run, sim) = play(7, 1, Policy::Expert);
    assert!(run.completed, "an attentive player finishes the training shift");
    assert_eq!(sim.phase, Phase::Done);
    let summary = sim.summary().join("\n");
    assert!(summary.contains(&format!("Incidents resolved: {}", sim.stats.resolved)));
    assert!(sim.log.iter().any(|e| e.cat == Cat::Shift && e.text.contains("17:00")));
}

#[test]
fn k_scenario_crew_and_call_in() {
    let mut sim = new_sim(9);
    sim.start(1); // training: Mango and Pip only
    let on: Vec<&str> = sim.workers.iter().filter(|w| w.active).map(|w| w.name.as_str()).collect();
    assert_eq!(on, ["Mango", "Pip"]);
    let moss = sim.workers.iter().position(|w| w.name == "Moss").unwrap();
    assert!(sim.order(Some(moss), 0, None).is_err(), "off-shift workers can't be ordered");
    sim.bananas = 5;
    sim.call_in(moss).unwrap();
    assert!(sim.workers[moss].active);
    assert_eq!(sim.bananas, 5 - sim.rules.call_cost);
    assert!(sim.order(Some(moss), 0, None).is_ok());
}

#[test]
fn l_summary_explains_right_and_wrong() {
    let mut sim = staged(11);
    let n = node(&sim, "logical-lagoon");
    sim.raise(n, "dhcp", false, None);
    sim.order(None, n, Some("restart")).unwrap();
    sim.order(None, n, Some("expandpool")).unwrap();
    run_until(&mut sim, 120.0, |s| s.faults[n].is_none());
    assert!(sim.notes_bad.iter().any(|x| x.contains("didn't help")), "{:?}", sim.notes_bad);
    assert!(!sim.notes_good.is_empty(), "{:?}", sim.notes_good);
    assert_eq!(sim.score, sim.stats.points_fix + sim.stats.points_prevent + sim.stats.points_combo);
}
