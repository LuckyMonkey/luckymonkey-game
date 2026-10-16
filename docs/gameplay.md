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
| **P** | Practice | One problem at a time, timers ×3, worms don't spread, no early game over. Doesn't count toward levels. |
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

| Level | Pressure |
|---|---|
| Practice | one problem at a time, timers ×3, worms don't spread |
| Shift 1: training | a scripted first hour that teaches as it goes (crash → DNS → certificate warning → two at once), then one problem at a time |
| Shift 2 | up to 2 independent problems by the afternoon |
| Shift 3+ | up to 3, timers ×0.85 |

The whole crew is on every shift. You only move up a level by **meeting the SLA target**.

## Your crew

| | | Faster at | Can't |
|---|---|---|---|
| 🐵 | **Mango** the monkey | nothing special, good at everything | |
| 🦜 | **Pip** the parrot | diagnosis (2×) | heavy work (deploys, databases) |
| 🦥 | **Moss** the sloth | preventive maintenance (2×): certificates, patches, clean-ups, canaries | |
| 🐸 | **Fern** the tree frog | hardware (2×): cables, vents, power | |
| 🐘 | **Tusk** the elephant | heavy work (2×): rollbacks, hotfixes, replicas, cables | |

- **Delegate, don't walk.** Pick a response and a crew member *runs* it straight
  away: about 2.5 s, or less for a specialist. They still head to the node on the
  map, but nothing waits for them to arrive.
- Each crew member runs one thing at a time, so five problems can be handled at once.
- **Z / X / C / V / N** choose who runs the next one. With nobody chosen, the game
  picks the quickest free one.

## Trouble starts somewhere and spreads

![DNS left alone: two neighbours already failing because of it, each pointing back to the origin](img/oncall-spread.png)

Every problem is **a root cause on one node**, and it starts for a reason. The
panel and the log say how: *"A botnet of a million bison found the Banana Cache on
the open internet."*, *"v2.4 went out to the API server without a canary."*

Left alone, it **spreads along its links** as its Time to Loss runs down: with
⅔ left, with ⅓ left, and when the node goes down. Neighbours show a 📉
**symptom**, for example *"the Lucky Monkey Guardian is failing because of the DNS
Shrine: it can't look up the shrine's names"*. A symptom has nothing to fix
locally: its panel names the origin and **O** takes you there. Fix the origin and
every symptom clears at once. Worms are the exception: they're contagious, and
each infected node needs its own patch.

When a node goes **down**, services whose only path ran through it go
**unreachable** (grey), and come back by themselves when the path does.

- **Warning** (amber, e.g. 📅 a deploy is scheduled, 🔐 a certificate expires
  soon): nothing is broken yet. Prevent it in time for bonus points and bananas.
- **Incident** (red): the node is degraded and its **Time to Loss** counts down.
- **Down**: still needs the same fix.

The **big Time to Loss timer** always shows the *next* consequence on the whole
network: a warning turning into an incident, a problem starting to spread, a node
going down, or a mitigation wearing off. Click it to fly there. It's a game term:
[not IP TTL and not DNS TTL](glossary.md).

### Responding: a command and its verdict

![Four responses to the same DDoS: two wrong, one that buys time, one that fixes it](img/oncall-terminal.png)

Open a node (click, Space for the most urgent, Tab to cycle) and pick one of four
responses (**1-4**). The terminal at the bottom types the real command, for example
`iptables -A INPUT -m set --match-set stampede src -j DROP`, streams its output,
runs a health check, and then gives the verdict:

- ✅ **CORRECT**: that fixed the root cause.
- 🩹 **PARTLY**: it bought time (e.g. rate-limiting a DDoS for 40 s); the cause is still there.
- ❌ **NOT THE FIX**: a real tool for a different problem. It costs the combo, and the
  verdict explains *why*, so you learn something.

Not sure?
- 🍌 **H: hint** marks the response that fixes it. Bananas are for hints only:
  you start with 3 and earn one per fix or prevention.
- 🔍 **I: diagnose** runs a check that labels *every* option with what it would
  do, and adds the concept to your field guide.

## The director: why it never piles on unfairly

New problems are dealt by an incident director
([`director`](source/sim.md#impl_sim)) that waits until all of these are true:
- it's been 15 s since your last fix;
- open root causes (symptoms don't count) are under the level's budget (outages left down still count, so
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
| Enter | start the shift |
| Space | jump to the most urgent problem |
| Tab | cycle open problems |
| 1-4 | run a response |
| H | 🍌 hint: mark the right response |
| I | diagnose |
| O | from a symptom, go to its origin |
| G | send the chosen crew member to the node (no job) |
| Z X C V N | choose Mango, Pip, Moss, Fern or Tusk |
| F | follow the chosen crew member |
| arrows | hop to the nearest node in that direction |
| Q / E | previous / next island |
| WASD, drag | pan · wheel, pinch, +/− zoom |
| Esc | hide the terminal |
| T · P · B | tour · practice · field guide |

Nodes off screen become **pins on the screen edge** in their direction, coloured
by health. Click a pin to fly there.

**On a phone:** drag to pan, pinch to zoom, tap a node, tap a response. The Time to
Loss timer sits across the top, and the node panel takes the lower half.

Next: [Failure types](failures.md) · [The simulation](simulation.md) · Back to the [documentation](index.md)
