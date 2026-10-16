---
title: Lucky Monkey - the network simulation
---

# The network simulation

> The map is the stage, and this is the play. Under the jungle there's a graph, a
> clock, a crew with to-do lists, and a director deciding what breaks next.

The whole game model is [`game/src/sim.rs`](source/sim.md), and it **knows nothing
about Bevy**. It's plain Rust over plain data, so:
- the browser game ([`ops.rs`](source/ops.md)) just calls
  [`Sim::step`](source/sim.md#impl_sim) every frame and draws the result;
- a native test crate, `game/simtest/`, compiles the *same file* and runs it
  thousands of times without a screen: 12 rule tests and a balancing harness with
  bot players.

This split is the [humble object](https://martinfowler.com/bliki/HumbleObject.html)
pattern: keep the logic where it's easy to test, and keep the hard-to-test part
(rendering, input) thin.

## Data in, simulation out

| Layer | File | What |
|---|---|---|
| Rules | [`tuning.json`](build-deploy.md#live_data) | every fault, response, action, worker, level, tutorial step, concept, notice and number; deserialised into [`Rules`](source/sim.md#struct_rules) |
| Map | `world.json` (from `islands.json` + `nodes.json`) | nodes, links, positions |
| Model | [`Sim`](source/sim.md#struct_sim) | health, faults, workers, the event log, stats |
| Presentation | [`ops.rs`](source/ops.md), [`ui.rs`](source/ui.md) | HUD, panels, map overlays, keys and clicks → orders |

The tuning file is fetched by the page at startup and deep-merged over a built-in
copy ([`data.rs`](source/data.md)), so a partial or broken file never stops the game.

## Health: root causes vs impacts

Each node is [`Online`, `Degraded`, `Down` or `Unreachable`](source/sim.md#enum_health).
After every change, [`recompute`](source/sim.md#impl_sim) walks the graph from the
core router through nodes that aren't down
([breadth-first search](https://en.wikipedia.org/wiki/Breadth-first_search)).
Anything not reached is **Unreachable**. That's an *impact*, not a fault: it gets
no incident, no repair, and recovers on its own when the path returns.

Network health is the criticality-weighted mean (online 1, degraded 0.5,
unreachable 0.3, down 0). Shift SLA is its time average.
[`impact_if_down`](source/sim.md#impl_sim) runs the same walk with one node
removed. The tour, the panel and the end-of-shift notes use it ("cutting off 27
other services").

## Faults

A [`Fault`](source/sim.md#struct_fault) sits on one node with a
[`Stage`](source/sim.md#enum_stage): **Warning → Incident → Down**. Each stage
has a Time to Loss. A **mitigation** pauses it for N seconds, then the fault is
back. Spreading faults (worm, DDoS) jump once to a healthy neighbour 30 s after
going down. The new fault keeps the original's `root` id, so the log and the end
screen can say "spread from".

The four options offered are the fault's own responses plus decoys, shuffled
([`options_for`](source/sim.md#impl_sim)). The panel never says "wrong", only
what happened and why.

## Workers

A [`Worker`](source/sim.md#struct_worker) has a position, stamina, a rest timer
and a queue of [`Task`](source/sim.md#struct_task)s. Walkers route over the
links with [Dijkstra](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm)
([`route`](source/sim.md#impl_sim)), and can't pass through a down node.
Flyers go straight. [`eta`](source/sim.md#impl_sim) +
[`work_secs`](source/sim.md#impl_sim) let the game pick whoever finishes first
when you don't choose ([`pick_worker`](source/sim.md#impl_sim)).

## The director

[`director`](source/sim.md#impl_sim) deals new root causes only after a 15 s
grace period since the last fix, while active root causes are under the level's
budget (it grows through the day) and health is above 60%. Weights are divided
by node criticality. Shift 1 follows a script (`tutorial` in tuning.json) and
advances past a step left down for 40 s. In quiet periods it posts flavour
notices ("Nightly backup completed on the Vault") that teach what normal looks
like.

## The event log

Everything that happens goes through [`push_event`](source/sim.md#fn_push_event):
dispatch, arrival, work, results, rests, spreads and recoveries. Events of the
same group within 6 s merge into one line with a count, so a router outage is
*one* line, "27 services unreachable", not 27. [`feed`](source/sim.md#fn_feed)
keeps urgent lines visible longer than chatter.

## Tests and balancing

    ssh fridge@fridge
    cd build/luckymonkey/game/simtest
    cargo test --release             # 12 rule tests
    cargo run --release --bin balance   # 4 levels × 4 bot policies × 20 seeds

The bots: **Idle** never acts, **Novice** guesses, **Beginner** diagnoses first,
**Expert** knows the fix and uses every worker.

| Level | Idle SLA (avg/min) | Novice | Beginner | Expert | Expert recovery |
|---|---|---|---|---|---|
| Practice | 97.9 / 94.2 | 99.6 | 99.7 | 99.8 | 8 s |
| Shift 1 | 91.6 / 91.6 | 99.6 | 99.6 | 99.8 | 8 s |
| Shift 2 | 92.2 / 85.9 | 99.5 | 99.6 | 99.8 | 9 s |
| Shift 3+ | 77.5 / 39.9 (2 of 20 collapse) | 99.1 | 99.4 | 99.7 | 8 s |

Reading it: ignoring the network misses the 95% target at every real level, and
any engaged player clears it. Skill shows in score, recovery time and
preventions. The bots react within seconds, so humans will score lower. A
human playtest is still the missing measurement.

Next: [Failure types](failures.md) · Back to the [documentation](index.md)
