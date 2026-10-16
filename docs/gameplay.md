---
title: Lucky Monkey - how to play
---

# How to play

> You're on call for a jungle. Eight hours, a crew of animals, and a network that
> breaks the way real ones do.

**▶ [Start the training shift](https://go.fridge.run/luckymonkey#shift)** ·
**▶ [Try a DDoS right now](https://go.fridge.run/luckymonkey#node=banana-sector&incident=ddos&zoom=1.2)**

The rules live in [`game/src/sim.rs`](source/sim.md) (no graphics, fully tested),
and every number lives in [`game/data/tuning.json`](build-deploy.md#live_data). You can
change the number in the JSON and see the result in seconds, with no rebuild.
[Failure types](failures.md) and [Nodes and lore](nodes.md) are generated from
that data.

![An incident: the red Time to Loss timer, red rope links, the node panel with fixes](img/oncall-red-links.png)

## Learn the network before you play

Nothing breaks until you press **START**. Before that you can:

| Key | Mode | What it is |
|---|---|---|
| **T** | Tour | Walks island by island through all 28 nodes: what each one *is* in plain words, its lore, what it links to, and how many services would lose their path if it went down. → / Space next, ← back, Esc ends. |
| **P** | Practice | One problem at a time, timers ×3, nothing spreads, no early game over, full crew. Doesn't count toward levels. |
| **B** | Field guide | Every concept the game has taught you so far, with the example where you met it. |

Click any node at any time: its panel shows its **role**, **lore**, **connections**
and **impact if down**.

## The shift

- 09:00 to 17:00 on the clock, **480 real seconds** (`shift_secs`).
- **Network health** is right now: each node counts 1 online, 0.5 degraded,
  0.3 unreachable, 0 down, weighted by how critical it is.
- **Shift SLA** is the *average* health over the shift so far. The target is
  **95%** (`sla_target`). That's the score that decides whether you pass.
- Health under **25%** for **45 s** (`outage_floor`, `outage_secs`) ends the shift
  early: **GAME OVER · major outage**.

### Levels

| Level | Crew | Pressure |
|---|---|---|
| Practice | 🐵 🦜 🦥 🐸 | one problem at a time, timers ×3 |
| Shift 1: training | 🐵 Mango, 🦜 Pip | a scripted first hour that teaches as it goes (crash → DNS → certificate warning → two at once), then gentle random trouble |
| Shift 2 | + 🦥 Moss | up to 3 root causes at once by the afternoon |
| Shift 3+ | + 🐸 Fern | 2 → 4 at once, timers ×0.85 |

You only move up a level by **meeting the SLA target**.

## Your crew

| | Moves | Good at | Can't |
|---|---|---|---|
| 🐵 **Mango**, monkey | walks the links | anything; swings on a vine for 🍌 1 (J) | — |
| 🦜 **Pip**, parrot | flies straight over the void, fastest | diagnosis at 2× | heavy work (deploys, databases) |
| 🦥 **Moss**, sloth | slow | preventive maintenance at 2× (certificates, patches, clean-ups, canaries) | — |
| 🐸 **Fern**, tree frog | hops fast | hardware at 2× (cables, vents, power) | — |

- Each worker has **stamina** (●●●○○) and rests when it runs out. **R** rallies a
  resting worker for 🍌 2.
- Each has a queue of up to 3 jobs. **Backspace** cancels the last.
- **Z / X / C / V** choose who takes the next order. With nobody chosen, the
  game sends whoever can arrive and finish first.
- Someone **off shift** can be called in with **K** for 🍌 3.

Bananas come from fixes and preventions. You start with 2.

## Trouble

Every problem is **a root cause on one node**. What it does to everything else is
an **impact**: when a router goes down, the services behind it go
**unreachable**. They don't get their own incident, and they recover by
themselves when the path comes back. Fix the cause and not the symptoms.

- **Warning** (amber, e.g. 📅 a deploy is scheduled, 🔐 a certificate expires
  soon): nothing is broken yet. Prevent it in time for bonus points and bananas.
- **Incident** (red): the node is degraded. Its **Time to Loss** counts down; at 0
  it goes **down**.
- **Down**: a worm or a flood can now **spread** once, 30 s later.

The **big Time to Loss timer** at the top always shows the *next* consequence on
the whole network, whether a warning turning into an incident, a node going
down, a mitigation wearing off, or a spread. Click it to fly there. It's a game
term: [not IP TTL and not DNS TTL](glossary.md).

### Responding

Open a node (click, Space for the most urgent, Tab to cycle) and pick one of four
responses (**1-4**):

- ✅ **Fix**: removes the root cause.
- 🩹 **Mitigate**: buys time (e.g. rate-limiting a DDoS for 40 s). The fault comes back.
- ✖ **No effect**: a real tool for a different problem. It costs the time and
  your combo, and the outcome card explains *why*, so you learn something.
- 🔍 **Diagnose (I)**: a worker inspects first. Every option is then labelled with
  what it would do, and the concept goes into your field guide.

Every outcome pops a card: **RESOLVED / MITIGATED / NO EFFECT / DIAGNOSED**, with
the cause, what was done, and what to learn.

## The director: why it never piles on unfairly

New problems are dealt by an incident director
([`director`](source/sim.md#impl_sim)) that waits until all of these are true:
- it's been 15 s since your last fix;
- open root causes are under the level's budget (outages left down still count, so
  ignoring trouble has a cost);
- network health is above 60%.

Critical infrastructure (router, DNS, auth, firewall, databases) is chosen less
often, and never in practice. Lightning from storm clouds goes through the same
budget.

## Score and the end of the shift

![The shift summary: score breakdown, what went right and wrong, your shift history](img/oncall-gameover.png)

Points come from fixes, preventions (worth more), the Time to Loss left when you
acted, and a combo for consecutive right calls. The end screen shows:

- a title: **SHIFT COMPLETE · SLA met**, **SHIFT OVER · SLA target missed**, or **GAME OVER · major outage**;
- the score broken down;
- ✅ **what went right** and ⚠ **what went wrong**, as specific events, e.g.
  "メインルーター Router went down when its Time to Loss ran out, cutting off 27 other service(s)";
- a tip;
- 🏆 your last 20 shifts in this browser.

## All the keys

| Key | Does |
|---|---|
| Enter | start the shift / go |
| Space | jump to the most urgent problem |
| Tab | cycle open problems |
| 1-4 | pick a response |
| I | diagnose |
| G | send the chosen worker here, no job |
| Z X C V | choose a worker |
| K | call in an off-shift worker (🍌 3) |
| R | rally a resting worker (🍌 2) |
| J | vine swing: Mango jumps to the selected node (🍌 1) |
| Backspace | cancel the chosen worker's last job |
| F | follow the chosen worker |
| arrows | hop to the nearest node in that direction |
| Q / E | previous / next island |
| WASD, drag | pan · wheel, pinch, +/− zoom |
| T · P · B | tour · practice · field guide |

Nodes off screen become **pins on the screen edge** in their direction, coloured
by health. Click a pin to fly there.

**On a phone:** drag to pan, pinch to zoom, tap a node, tap a response. The Time to
Loss timer sits across the top, and the node panel takes the lower half.

Next: [Failure types](failures.md) · [The simulation](simulation.md) · Back to the [documentation](index.md)
