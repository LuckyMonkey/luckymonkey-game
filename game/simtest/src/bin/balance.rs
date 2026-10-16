//! Balancing report: bots play many seeded shifts per difficulty level.
use lm_simtest::{play, Policy};

fn main() {
    let seeds: Vec<u64> = (1..=20).collect();
    println!("level policy    done  SLA(avg/min)  roots  maxOpen(avg/worst)  crit-s  resolved  prevented  avgRec  spreads  outages  busy%");
    for level in 0..4 {
        for policy in [Policy::Idle, Policy::Novice, Policy::Beginner, Policy::Expert] {
            let runs: Vec<_> = seeds.iter().map(|&s| play(s, level, policy).0).collect();
            let n = runs.len() as f32;
            let avg = |f: &dyn Fn(&lm_simtest::Run) -> f32| runs.iter().map(f).sum::<f32>() / n;
            let done = runs.iter().filter(|r| r.completed).count();
            let min_sla = runs.iter().map(|r| r.sla).fold(f32::MAX, f32::min);
            let worst = runs.iter().map(|r| r.max_open).max().unwrap();
            println!(
                "{level:>5} {:<9} {done:>2}/{:<2} {:>5.1}/{:<5.1}  {:>5.1}  {:>5.1}/{:<3}          {:>6.0}  {:>8.1}  {:>9.1}  {:>6.1}  {:>7.1}  {:>7.1}  {:>5.0}",
                format!("{policy:?}"),
                runs.len(),
                avg(&|r| r.sla),
                min_sla,
                avg(&|r| r.roots as f32),
                avg(&|r| r.max_open as f32),
                worst,
                avg(&|r| r.critical_secs),
                avg(&|r| r.resolved as f32),
                avg(&|r| r.prevented as f32),
                avg(&|r| r.avg_recovery),
                avg(&|r| r.spreads as f32),
                avg(&|r| r.outages as f32),
                avg(&|r| r.busy / (r.busy + r.idle).max(1.0) * 100.0),
            );
        }
    }
}
