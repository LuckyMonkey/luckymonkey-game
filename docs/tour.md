---
title: Lucky Monkey - a tour in pictures
---

# A tour in pictures

Every picture here was taken from the live game by opening a
[deep link](interface.md#deep_links). Click **▶ live** under any picture to
open the game at exactly that spot, in your browser, right now. (The game
downloads about 6.7 MB the first time.)

## The on-call jungle (current)

The archipelago as it is now: separate floating islands at different heights,
one river from void to void, rope links between islands.

![Overview: islands, the river, rope links](img/oncall-overview.png)

**Tour mode (T)** walks you through every node with its role, links and lore:

![Tour mode: a node's role, what it links to, how much depends on it](img/oncall-tour.png)

An incident: the big red **Time to Loss**, red links to the troubled node, the crew
roster and the shared network log:

![An incident with red links and the Time to Loss timer](img/oncall-red-links.png)

When the main router goes down, everything behind it goes **unreachable**: an
impact, not a fault of its own.

![Router down: unreachable services greyed out](img/oncall-router-down.png)

Diagnose first and the panel tells you what each option would do; the field guide
keeps every concept you've met:

![Diagnosed: every option labelled](img/oncall-diagnosed.png)
![The field guide](img/oncall-field-guide.png)

Pixel vines hang from the island edges:

![Pixel vines](img/oncall-vines.png)

And the end of the shift tells you what went right, what went wrong, and your score:

![Game over screen](img/oncall-gameover.png)

## Earlier pictures

The pictures below are from before the archipelago (one landmass, single monkey,
Watchtower forecasts). The places still exist, and the ▶ live links still open them.

### The whole jungle

![The map zoomed all the way out: islands of terrain floating in a dark grid, rivers, node cards and link frames](img/overview.png)

The world is a set of terrain islands floating in a dark void. The void is a
[tiled grid pattern](source/world.md#fn_spawn_backdrop) with faint teal
[edge islands](source/world.md#fn_spawn_islands) hinting at more network beyond.
The coloured cards are the 28 [nodes](assets.md#what_is_whatthe_nodes). The pale
dashes between them are [data frames streaming along the links](world.md#the_river_and_the_links).
▶ [live](https://go.fridge.run/luckymonkey#zoom=0.5)

### The Guardian's pyramid

![The golden stepped pyramid with the Lucky Monkey Guardian card on top](img/pyramid.png)

The Lucky Monkey Guardian sits on a stepped golden temple at the centre of the
network. The map generator originally built this pyramid **upside down** (the
widest layer was the highest). The loader [rebuilds it](source/world.md#fn_plugin)
so that the more layers cover a cell, the higher it stands. See
[Decisions](decisions.md#bugs_found_and_fixed_in_the_port).
▶ [live](https://go.fridge.run/luckymonkey#node=monkey-statue&zoom=1.1)

### The waterfall and the Message Queue Falls

![The waterfall stepping down from height 5, white streams pouring, the Message Queue Falls card beside it](img/waterfall.png)

The main river starts at a waterfall that steps down five levels. White streams
pour over each step: three [streaks per tile](source/water.md#fn_spawn) restored from the
original canvas animation (see [water effects](world.md#water_effects)). Messages
wait at the top of the falls and drop one at a time, in order. Usually.
▶ [live](https://go.fridge.run/luckymonkey#node=mq-falls&zoom=1.6)

### The Logical Lagoon

![The lagoon with bubbles rising and frames floating past](img/lagoon.png)

Bubbles rise and pop in the lagoon (the original CSS `bubble` keyframes,
re-done in [`water.rs`](source/water.md#fn_animate)). The little boxes floating
on the water are **data frames**: the rivers carry data downstream.
▶ [live](https://go.fridge.run/luckymonkey#node=logical-lagoon&zoom=1.8)

### The Data Lake

![The Data Lake card standing on a round lake at the end of the Digital River](img/data-lake.png)

The Digital River runs down the west edge of the map and ends here. Every frame
that ever flowed downstream settles in the Data Lake, "replicated three ways and
rarely read". Data stores like this one suffer
[replica disparity](failures.md#replica_disparity) and [full disks](failures.md#disk_full).
▶ [live](https://go.fridge.run/luckymonkey#node=data-lake&zoom=1.4)

### The volcano

![The Volcano Core Processor on a raised lava field studded with crystals](img/volcano.png)

The Volcano Core Processor: "processes data at extreme speeds but occasionally
erupts". Its platform is lava tiles from the [atlas](world.md#tiles_and_the_alignment_bug_that_started_it_all);
the crystals are emoji decorations.
▶ [live](https://go.fridge.run/luckymonkey#node=volcano-core&zoom=1.2)

### The Lilypad Load Balancer

![The pink Lilypad Load Balancer card standing on the river](img/lilypads.png)

A load balancer standing in the river: "frames hop from pad to pad; the pads
take turns so no single one sinks". It is one of
[seven places added](overview.md#historythree_renderers_in_three_weeks) so the network reads like real
infrastructure.
▶ [live](https://go.fridge.run/luckymonkey#node=lb-lilypads&zoom=1.6)

### The DNS Shrine

![The red DNS Shrine card](img/dns.png)

"Monks chant names into numbers. When the shrine forgets, nobody in the jungle
can find anybody else." ([What DNS is](https://en.wikipedia.org/wiki/Domain_Name_System);
the fridge's [Network+ lesson on the services every network runs](https://go.fridge.run/certs/learn/netplus?n=4).)
▶ [live](https://go.fridge.run/luckymonkey#node=dns-shrine&zoom=1.4)

### The Canopy Watchtower

![The orange Canopy Watchtower card with its connections](img/watchtower.png)

Monitoring. While the Watchtower is up, you get **forecasts** of trouble before it
happens. If it goes down, the forecasts vanish and you fly blind. See
[forecasts](gameplay.md#trouble).
▶ [live](https://go.fridge.run/luckymonkey#node=watchtower&zoom=1.4)

### Trouble: a DDoS stampede

![A DDoS on the Banana Sector Cache: red frames racing in along every link, the incident panel open with four fixes](img/ddos.png)

Every link into the victim fills with fast red frames heading in: the flood made
visible ([the code](source/ops.md#fn_frames)). The panel shows the symptoms and
four possible fixes. One is right. Which one, and why, is in the
[failure guide](failures.md#ddos_flood).
▶ [live](https://go.fridge.run/luckymonkey#node=banana-sector&incident=ddos&zoom=1.2)

### Trouble you can see coming: a forecast

![The Auth Gate with an amber forecast: certificate about to expire, countdown, preventive fixes](img/forecast.png)

An amber **forecast** from the Watchtower: the Auth Gate's certificate is about to
expire. Nothing is broken yet. Rotate it before the countdown ends and the outage
never happens, and you score more than you would for fixing it afterwards.
▶ [live](https://go.fridge.run/luckymonkey#node=auth-gate&forecast=cert&zoom=1.3)

### Trouble that tests you: a deploy with a migration

![The main router with a "bad deploy + migration" incident; both Roll back and Fall forward are offered](img/migration.png)

Both *Roll back* and *Fall forward* are always offered for a bad deploy. Read the
symptom: this deploy **migrated the schema**, so the old version cannot read the
new tables, and rolling back would make things worse. This is the lesson the
owner asked for ("fall forward").
[Why](failures.md#bad_deploy).
▶ [live](https://go.fridge.run/luckymonkey#node=main-router&incident=migration)

### Trouble with data: replica disparity

![The database cluster with a replica disparity incident](img/disparity.png)

Two copies of the data, two different banana counts. Resync from the primary
before the countdown ends, or the data is lost for good (and the SLA takes an extra hit).
▶ [live](https://go.fridge.run/luckymonkey#node=database-cluster&incident=disparity&zoom=1.3)

Next: [Overview and history](overview.md) · Back to the [documentation](index.md)
