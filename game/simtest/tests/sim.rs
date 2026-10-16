//! Behaviour tests for the simulation.
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
            max_open = max_open.max(sim.open_roots());
        }
        assert_eq!(max_open, 1, "seed {seed}: training opened {max_open} root causes in the first minute");
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
    assert_eq!(sim.open_roots(), 1, "dependents must not become separate incidents");
    for (i, f) in sim.faults.iter().enumerate() {
        if let Some(f) = f {
            assert!(i == core || f.cause.is_some_and(|c| c.1 == core), "{} has its own fault", sim.nodes[i].name);
        }
    }
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
fn f_responses_run_without_waiting_for_travel() {
    let mut sim = staged(5);
    // The node farthest from Mango.
    let far = (0..sim.nodes.len()).max_by(|&a, &b| sim.eta(0, a).total_cmp(&sim.eta(0, b))).unwrap();
    assert!(sim.eta(0, far) > 5.0, "test needs a distant node");
    sim.raise(far, "crash", false, None);
    sim.order(Some(0), far, Some("restart")).unwrap();
    let run = sim.work_secs(0, "restart", far);
    let took = run_until(&mut sim, 60.0, |s| s.faults[far].is_none());
    assert!(took <= run + 0.1, "took {took:.1}s, run is {run:.1}s");
    assert!(sim.outcome_run.is_some(), "the terminal can find the result");
    let cats: Vec<Cat> = sim.log.iter().map(|e| e.cat).collect();
    let d = cats.iter().position(|c| *c == Cat::Dispatch).unwrap();
    let r = cats.iter().position(|c| *c == Cat::Resolved).unwrap();
    assert!(d < r);
    assert!(sim.order(Some(0), far, Some("restart")).is_err() || sim.faults[far].is_none());
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
fn k_whole_crew_and_banana_hints() {
    let mut sim = staged(9);
    let on: Vec<&str> = sim.workers.iter().filter(|w| w.active).map(|w| w.name.as_str()).collect();
    assert_eq!(on, ["Mango", "Pip", "Moss", "Fern", "Tusk"]);
    let n = node(&sim, "logical-lagoon");
    sim.raise(n, "dhcp", false, None);
    sim.bananas = 1;
    sim.hint(n).unwrap();
    assert_eq!(sim.bananas, 0);
    assert!(sim.faults[n].as_ref().unwrap().hinted);
    assert!(sim.hint(n).is_err(), "no second hint for the same problem");
    let m = node(&sim, "terminal-01");
    sim.raise(m, "crash", false, None);
    assert!(sim.hint(m).is_err(), "no bananas, no hint");
    // Tusk does heavy work twice as fast as Mango.
    let tusk = sim.workers.iter().position(|w| w.name == "Tusk").unwrap();
    assert!(sim.work_secs(tusk, "resync", n) < sim.work_secs(0, "resync", n));
}

#[test]
fn m_trouble_spreads_from_its_origin_and_clears_with_it() {
    let mut sim = staged(12);
    let dns = node(&sim, "dns-shrine");
    assert!(sim.raise(dns, "dns", false, None));
    assert!(!sim.faults[dns].as_ref().unwrap().story.is_empty(), "every incident says how it started");
    let id = sim.faults[dns].as_ref().unwrap().id;
    run_until(&mut sim, 200.0, |s| s.faults.iter().flatten().filter(|f| f.is_symptom()).count() >= 2);
    let symptoms: Vec<usize> = (0..sim.nodes.len()).filter(|&i| sim.is_symptom(i)).collect();
    assert!(symptoms.len() >= 2, "an untouched incident spreads");
    for &j in &symptoms {
        assert!(sim.neighbours[dns].contains(&j), "symptoms appear next to the origin");
        assert_eq!(sim.faults[j].as_ref().unwrap().cause, Some((id, dns)));
        assert!(sim.order(None, j, Some("restart")).is_err(), "nothing to fix at a symptom");
    }
    assert_eq!(sim.open_roots(), 1);
    sim.order(None, dns, Some("fixdns")).unwrap();
    run_until(&mut sim, 30.0, |s| s.faults[dns].is_none());
    assert!(sim.faults.iter().all(|f| f.is_none()), "fixing the origin clears its symptoms");
    assert!(sim.log.iter().any(|e| e.cat == Cat::Recover && e.text.contains("once")));
}

#[test]
fn n_worms_are_contagious() {
    let mut sim = staged(13);
    let n = node(&sim, "server-hub");
    sim.raise(n, "worm", false, None);
    run_until(&mut sim, 200.0, |s| s.faults.iter().flatten().any(|f| f.kind == "worm" && f.id != f.root));
    let spread: Vec<_> = sim.faults.iter().flatten().filter(|f| f.kind == "worm" && f.id != f.root).collect();
    assert!(!spread.is_empty(), "a worm left alone infects a neighbour");
    assert!(spread[0].cause.is_none() && spread[0].story.contains("spread"), "it's a real infection with its origin named");
}

#[test]
fn l_summary_explains_right_and_wrong() {
    let mut sim = staged(11);
    let n = node(&sim, "logical-lagoon");
    sim.raise(n, "dhcp", false, None);
    sim.order(None, n, Some("restart")).unwrap();
    run_until(&mut sim, 30.0, |s| s.runs.is_empty());
    sim.order(None, n, Some("expandpool")).unwrap();
    run_until(&mut sim, 30.0, |s| s.faults[n].is_none());
    assert!(sim.notes_bad.iter().any(|x| x.contains("didn't help")), "{:?}", sim.notes_bad);
    assert!(!sim.notes_good.is_empty(), "{:?}", sim.notes_good);
    assert_eq!(sim.score, sim.stats.points_fix + sim.stats.points_prevent + sim.stats.points_combo);
}
