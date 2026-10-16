---
title: Lucky Monkey - nodes and lore
---

# Nodes and lore

> Thirty-three places on eleven islands, each a real kind of infrastructure, each with a story.

**Generated from [`game/data/nodes.json`](world.md) and [`islands.json`](world.md#the_archipelago) by `tools/gen-docs.py`.** In the game, press **T** (outside a shift) for the same tour, island by island, with what each node links to and how many services would lose their path if it went down.

## The legend

> Long before the cables, a river fell out of the sky. Nobody has seen where it comes from: it pours in from the void above the highest island and runs on, past the Data Lake and the southern reefs, until it leaves the map again into the dark. Islands drifted along it like leaves, and the animals who lived on them learned to talk across the water, first with shouts, then with vines, and finally with frames that ride the current. Every island has its job, and every job depends on the others. When one breaks, the river carries the trouble downstream. That's why the jungle keeps an on-call crew.

## 🫧 Lagoon Heights

Height 9 · jungle · on the river

*The highest island, where the river first touches land. The water is still warm from wherever it fell from, and the frogs who run the lagoon greet every newcomer with an address and a lotus pad. The Message Queue Falls tumble off its western edge.*

### 🌊 Logical Lagoon API

`logical-lagoon` · river · 192.168.1.42 · ▶ [see it](https://go.fridge.run/luckymonkey#node=logical-lagoon&zoom=1.4)

**Role:** A network segment where devices join, get addresses (DHCP) and talk to each other.

*The river slows here into a warm pool where APIs float on lotus pads. The frogs say the water remembers every request ever made of it. They also hand out addresses to anything new that splashes in.*

*Long before the cables, this was where the first animals met to trade. The lagoon still hands every newcomer an address, a name tag for the water, and lends it back when they leave. On-call note: when new devices can't join, look here first. Running out of addresses looks a lot like everything else breaking.*

**Links to:** Lucky Monkey Guardian, Digital River Stream

### 📨 Message Queue Falls

`mq-falls` · queue · 10.2.0.5 · ▶ [see it](https://go.fridge.run/luckymonkey#node=mq-falls&zoom=1.4)

**Role:** A message queue holds messages in order until consumers are ready, smoothing out bursts.

*Messages queue in the pool at the top of the falls and drop one at a time, in order, so nothing downstream gets swamped.*

*Messages wait their turn in the pool and drop over the edge one by one. When the falls jam, the pool rises, and the whole lagoon hears about it. On-call note: queues smooth out bursts, until they fill. Then everything upstream backs up.*

**Links to:** Logical Lagoon API, 北ファイアウォール Firewall, Node Temple Beta

### 🦎 Reptile Protocol

`lizard-node` · lizard · 10.0.2.42 · ▶ [see it](https://go.fridge.run/luckymonkey#node=lizard-node&zoom=1.4)

**Role:** A monitoring probe watching a segment.

*A cold-blooded monitor that lies very still on a warm rock and notices everything. It reports nothing until it is sure, which is either wisdom or laziness.*

*The lizard sees traffic as warmth: busy links glow, quiet ones go cold. It has noticed every outage in history about four seconds before everyone else, and told nobody. On-call note: the lagoon's segment runs through here; address trouble on the lagoon reaches it first.*

**Links to:** Lucky Monkey Guardian, Digital River Stream

## 🌳 Canopy Ridge

Height 7 · jungle · on the river, vine bridge to Digital Shore

*A ridge of old trees so tall that their tops hold the jungle's server racks, its northern firewall and its backup vault. Parrots nest on the warm racks; the toucan who guards the vault doesn't nest anywhere, out of principle.*

### 🔥 北ファイアウォール Firewall

`firewall-north` · firewall · 192.168.0.10 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=firewall-north&zoom=1.4)

**Role:** A firewall allows or blocks traffic by rules (ACLs). The first line against attacks.

*A wall of thorny vines that lets the right traffic through and stings everything else. It runs rules written so long ago that nobody dares delete one.*

*Vines grow over old vines; rules sit on top of older rules. Once a year someone proposes cleaning it up, and once a year the proposal is gently returned unread. On-call note: when a stampede hits, this is where an ACL turns away the attackers and lets the real visitors through.*

**Links to:** メインルーター Router, データベース Database

### 🏯 Node Temple Beta

`node-temple-2` · temple · 10.0.0.2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=node-temple-2&zoom=1.4)

**Role:** An application server: runs services that other nodes call.

*Its door opens only for a hash nobody remembers choosing. The priests keep a backup of the hash on a stone tablet in the vault, which is locked with the same hash.*

*Beta was built as a copy of Alpha so that one temple could always be awake. It drifted. Nobody remembers when the two stopped agreeing about which day it is. On-call note: if something here won't open, check its certificate before blaming the hash.*

**Links to:** Banana Sector Cache, Server Hub Cluster

### 💻 Server Hub Cluster

`server-hub` · server · 192.168.1.200 · ▶ [see it](https://go.fridge.run/luckymonkey#node=server-hub&zoom=1.4)

**Role:** A general-purpose server running services. Can crash, leak memory or overheat.

*Rows of humming racks under the canopy, warm enough that the parrots nest on top of them in the rainy season. Most of the jungle's work actually happens here.*

*The racks are named after fruit, and the fruit after the racks; after a while nobody could say which came first. The parrots have learned the fans' songs and sing along when one starts to fail. On-call note: most real work runs here, so a leak or a crash here is felt everywhere downstream.*

**Links to:** Node Temple Beta

### 🧱 Backup Vault Node

`vault-01` · vault · 10.0.0.3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=vault-01&zoom=1.4)

**Role:** Backup storage. Backups only count if they can be restored.

*The backups live here, behind three doors and one very suspicious toucan. A backup nobody has ever restored is just a hope, so the toucan restores one every Friday.*

*The toucan has never lost a backup and has never once been thanked, which is how backup work usually goes. The vault door is carved with the date of the last successful restore. On-call note: snapshots pile up. Prune them before the disk fills, not after.*

**Links to:** Neural Processor Alpha

## 🛕 Temple Isle

Height 5 · temple · on the river, vine bridge to Banana Grove

*The heart of the network: the Guardian's pyramid, the main router on the highest ledge, the DNS Shrine and the thinking stone. Almost every path in the jungle crosses this island, which is why the monks never stop chanting.*

### 🐵 Lucky Monkey Guardian

`monkey-statue` · monkey · 192.168.1.1 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=monkey-statue&zoom=1.4)

**Role:** The Guardian: a core hub. Many paths run through it.

*Carved by the first sysadmins from a single block of jade, the Guardian sits at the top of the pyramid and watches every packet with half-closed eyes. Monkeys leave a banana at its feet before every deploy. Nobody admits to believing it helps.*

*Under its base runs the oldest cable in the jungle, laid by hand and never replaced, because nobody can agree who would have to move the statue. Four paths meet at its feet, which is why the first thing anyone does in an outage is look up at the Guardian. On-call note: it's a hub. When it struggles, the lagoon, the bananas, the temple and the volcano all feel it.*

**Links to:** Logical Lagoon API, Banana Sector Cache, Node Temple Alpha, Volcano Core Processor

### 📡 メインルーター Router

`main-router` · router · 192.168.0.1 · criticality 5 · ▶ [see it](https://go.fridge.run/luckymonkey#node=main-router&zoom=1.4)

**Role:** A router forwards packets between networks using its routing table. If it fails, everything that reaches the rest of the network through it is cut off.

*Perched on the highest ledge of Temple Isle, the router knows a path to every island. If it goes quiet, the islands can still see each other across the void but have no way to talk.*

*Its routing table is written on a scroll so long it's kept on a spindle, and the monkeys turn it by hand at night. A single wrong line can send every packet on the shore in circles. On-call note: the core. Almost every path in the jungle crosses it, so when it goes down, whole islands go dark at once.*

**Links to:** Lucky Monkey Guardian, 北ファイアウォール Firewall, 南ファイアウォール Firewall

### ⛩️ DNS Shrine

`dns-shrine` · dns · 10.0.0.53 · criticality 4 · ▶ [see it](https://go.fridge.run/luckymonkey#node=dns-shrine&zoom=1.4)

**Role:** A DNS server turns names into IP addresses. Without it, names stop working even though the network is fine.

*Monks chant the names of every island and the numbers they stand for. Visitors ask for a name and leave with a number. When the monks forget, nobody can find anything.*

*The monks have chanted without a break for as long as the jungle has had names. Visitors who ask for a name that doesn't exist get a polite bow and a NXDOMAIN. On-call note: when names stop working but numbers still do, it's always the shrine. Its neighbours fall quiet one by one when it stumbles.*

**Links to:** Lucky Monkey Guardian, Node Temple Alpha, Reptile Protocol, メインルーター Router

### 🧠 Neural Processor Alpha

`ai-core-01` · ai-core · 10.0.1.42 · ▶ [see it](https://go.fridge.run/luckymonkey#node=ai-core-01&zoom=1.4)

**Role:** A compute node running heavy workloads.

*A thinking stone that sorts the jungle's problems by how weird they are. It once predicted a disk would fill on a Tuesday. It was a Wednesday, but everyone was impressed.*

*It sorts problems by smell, mostly. Its favourite word is 'anomaly', and it uses it about twice a minute. On-call note: memory leaks love long-running thinkers. A restart helps for a while; the patch helps for good.*

**Links to:** Lucky Monkey Guardian, Backup Vault Node

## 💾 Digital Shore

Height 3 · jungle · on the river, vine bridge to Canopy Ridge

*Where the river widens and slows. The Digital River runs along its banks, the southern firewall stands guard, and Node Temple Alpha has been sending packets to people who left long ago for longer than anyone can remember.*

### 🌊 Digital River Stream

`digital-river` · river · 192.168.1.101 · ▶ [see it](https://go.fridge.run/luckymonkey#node=digital-river&zoom=1.4)

**Role:** A network segment where devices join, get addresses (DHCP) and talk to each other.

*A fast, shallow channel that carries raw signal from the lagoon to the shore. On calm days you can see the frames glinting under the surface like minnows.*

*The river was here before the network, and the network learned to float on it. The channel is shallow enough that a careless deploy can stir up the mud for miles. On-call note: a segment, not a server. It carries traffic for everyone downstream, so its trouble travels.*

**Links to:** Logical Lagoon API, Crashed Sector 404

### 🏯 Node Temple Alpha

`node-temple-1` · temple · 10.0.0.1 · ▶ [see it](https://go.fridge.run/luckymonkey#node=node-temple-1&zoom=1.4)

**Role:** An application server: runs services that other nodes call.

*The older of the two temples, built before anyone wrote things down. Ghostpackets still sweep its halls at night, delivering messages addressed to people who left long ago.*

*The walls are carved with routing tables in a script that predates the shrine. Scholars argue about whether the carvings describe the network or caused it. On-call note: old, trusted and well connected, which is exactly how worms like their temples.*

**Links to:** Lucky Monkey Guardian, Crashed Sector 404

### 🔥 南ファイアウォール Firewall

`firewall-south` · firewall · 192.168.0.11 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=firewall-south&zoom=1.4)

**Role:** A firewall allows or blocks traffic by rules (ACLs). The first line against attacks.

*The southern wall is younger and stricter. It checks every packet twice, which the southern islanders call security and everyone else calls slow.*

*It was planted after a worm walked in through the shore one quiet Sunday. The southern islanders have never quite forgiven the north for leaving the gate open. On-call note: strict is good, but a strict wall on the wrong VLAN blocks your own traffic too.*

**Links to:** メインルーター Router

## 🍌 Banana Grove

Height 5 · grove · on the river, vine bridge to Temple Isle

*Rows of banana palms on terraces, a cache for every craving, and the Lilypad Load Balancer across the water. The grove sits on the open side of the jungle, so it's the first place a stampede arrives.*

### 🍌 Banana Sector Cache

`banana-sector` · banana-cache · 192.168.1.99 · ▶ [see it](https://go.fridge.run/luckymonkey#node=banana-sector&zoom=1.4)

**Role:** A cache keeps copies of frequently used data close by, so requests don't have to go all the way to the database.

*Every banana the jungle has ever needed twice is kept here, ripe and ready, so nobody has to climb to the archive again. When it forgets one, the whole jungle notices the walk.*

*The cache was built after the Great Walk, the week the archive was slow and every monkey had to climb for every banana. Nobody wants another Great Walk. On-call note: it sits on the open edge of the grove, so it's the first thing stampedes find.*

**Links to:** Lucky Monkey Guardian, Node Temple Beta

### 🍌 Humor Cache Alpha

`banana-cache-01` · banana-cache · 10.0.0.4 · ▶ [see it](https://go.fridge.run/luckymonkey#node=banana-cache-01&zoom=1.4)

**Role:** A cache keeps copies of frequently used data close by, so requests don't have to go all the way to the database.

*A small cache of jokes the monkeys tell during long outages. It is surprisingly critical infrastructure.*

*Its contents are classified, mostly because they're not very good. During the Long Night of the DNS Outage, it served eleven thousand jokes and kept morale exactly where it needed to be. On-call note: small things matter more than they look during a long incident.*

**Links to:** Junction Switch, Legacy Terminal

### 🛂 Auth Gate

`auth-gate` · auth · 10.3.0.1 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=auth-gate&zoom=1.4)

**Role:** Authentication: checks identities and holds TLS certificates others trust.

*A carved gate where every visitor shows their papers. It also keeps the certificates the rest of the jungle trusts, which makes it a very popular target.*

*The gate's carvings change every year, when the certificates are renewed in a small ceremony nobody likes to miss. The one year somebody did miss it is still talked about. On-call note: an expired certificate here breaks every login in the jungle at once. Renew it before it expires.*

**Links to:** Backup Vault Node, Neural Processor Alpha, Humor Cache Alpha, メインルーター Router

### 📚 Ancient Data Archive

`data-archive` · database · 10.0.1.42 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=data-archive&zoom=1.4)

**Role:** A database stores records, often as several replicas that must agree.

*The deepest storage in the jungle: every log, every snapshot, every version of every banana count. It fills up slowly and then all at once.*

*The archive is so deep that the oldest records are in a format nobody can read anymore, kept on the off chance someone learns. Its librarian is a tortoise who has never been in a hurry and never lost a page. On-call note: disks fill slowly and then all at once. A warning here is a gift.*

**Links to:** Banana Sector Cache

## 🌊 Data Lake

Height 1 · jungle · on the river

*Where the river spreads into a wide, still lake before running on south. Everything that floated downstream settles here; the analysts fish for patterns from the lotus beds.*

### 🏞️ Data Lake

`data-lake` · lake · 10.4.0.1 · ▶ [see it](https://go.fridge.run/luckymonkey#node=data-lake&zoom=1.4)

**Role:** A data lake: large, cheap storage for raw data and analytics.

*Where the river finally stops. Every frame that ever flowed downstream settles here, and the analysts fish for patterns from the shore.*

*Every frame that ever ran downstream ends here, which makes the lake the most honest place in the jungle: whatever the logs upstream say, the lake knows what actually happened. On-call note: the lake is where the river's trouble collects, and where the new islands to the south and west tie in.*

**Links to:** Digital River Stream, Legacy Terminal, 南ファイアウォール Firewall

### 💻 Legacy Terminal

`terminal-01` · terminal · 10.0.0.5 · ▶ [see it](https://go.fridge.run/luckymonkey#node=terminal-01&zoom=1.4)

**Role:** A client machine: a user's view of the network.

*An old green-screen terminal at the end of the lake path. Its commands still work, mostly, as long as you type them in capital letters.*

*Its keyboard is missing the letter Q, so every command involving a queue is typed with a sigh. It has outlived three replacements, each of which crashed first. On-call note: a classic first incident. The process died, the machine didn't. Restart it.*

**Links to:** Humor Cache Alpha

### 🔀 Junction Switch

`switch-01` · switch · 10.0.0.2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=switch-01&zoom=1.4)

**Role:** A switch connects devices on the same network and forwards frames by MAC address. Loops between switches cause broadcast storms unless Spanning Tree blocks them.

*Every cable on the shore meets at this junction, where a switch decides which port each frame goes out of. Plug two of its ports into each other and the frames chase themselves forever.*

*Every frame on the shore passes through here, and the switch remembers which port every animal lives behind. It forgets the moment the power blinks, and then it has to ask everyone again. On-call note: loops and broadcast storms start at switches; Spanning Tree is the cure.*

**Links to:** Humor Cache Alpha

### 🪷 Lilypad Load Balancer

`lb-lilypads` · balancer · 10.1.0.10 · criticality 2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=lb-lilypads&zoom=1.4)

**Role:** A load balancer spreads requests across several servers so none is overloaded.

*Frames cross the river by hopping pad to pad. The pads take turns carrying them, so no single pad sinks under the weight.*

*The pads are rotated every morning by a family of frogs, who take the job very seriously. One pad, Number Seven, is everyone's favourite and is never allowed to carry more than its share. On-call note: a load balancer in front of a stampede takes the hit first. Block the source, don't sink the pads.*

**Links to:** メインルーター Router, Junction Switch, Banana Sector Cache, CDN Canopy

## 🌋 Ember Isle

Height 6 · volcano · free-floating

*A volcano island that drifted off on its own. It runs the fastest processor in the jungle and the strangest machine, and it's connected to the rest only by long ropes across the void.*

### 🌋 Volcano Core Processor

`volcano-core` · volcano · 192.168.1.666 · ▶ [see it](https://go.fridge.run/luckymonkey#node=volcano-core&zoom=1.4)

**Role:** A high-performance compute node: fast, hot, and sensitive to cooling.

*The fastest processor in the jungle runs inside a live volcano, cooled by the river and good intentions. It throttles itself whenever the leaves pile up on the vents.*

*Nobody chose to put a processor inside a volcano; it just ran faster there, and nobody wanted to be the one to move it. The lava keeps it warm, and the river keeps it alive. On-call note: heat is a hardware problem. Restarting a hot machine buys minutes; clearing the vents fixes it.*

**Links to:** Lucky Monkey Guardian

### 💻 Crashed Sector 404

`crashed-sector` · server · 192.168.1.404 · ▶ [see it](https://go.fridge.run/luckymonkey#node=crashed-sector&zoom=1.4)

**Role:** A general-purpose server running services. Can crash, leak memory or overheat.

*Something fell out of the sky here once and never booted again. The monkeys keep it around as a reminder to take backups.*

*Some say it was a satellite. Some say it was a deploy on a Friday. Either way, the crater is now the jungle's favourite place to remember why rollbacks exist. On-call note: it still answers ping, it just never does anything else. A machine can be up while its service is dead.*

**Links to:** Node Temple Alpha, Digital River Stream

### ⚛️ Quantum Nexus Core

`quantum-nexus` · server · 192.168.q.∞ · ▶ [see it](https://go.fridge.run/luckymonkey#node=quantum-nexus&zoom=1.4)

**Role:** A general-purpose server running services. Can crash, leak memory or overheat.

*An experimental machine on Ember Isle that is both running and not running until someone checks. The on-call monkey would rather it just picked one.*

*It arrived in a crate labelled THIS WAY UP on every side. The research frogs say it's the future. The on-call monkeys say the future keeps paging them at three in the morning. On-call note: experimental machines run hot and leak memory. Watch them.*

**Links to:** Node Temple Alpha

## 🔭 Watch Isle

Height 8 · jungle · free-floating

*A rock with a tower on it, floating alone to the east, from which the heron can see every island at once and every storm forming over the void.*

### 🔭 Canopy Watchtower

`watchtower` · monitor · 10.9.0.1 · criticality 2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=watchtower&zoom=1.4)

**Role:** Monitoring: collects metrics and warns before things break. If it's unreachable you lose your warnings.

*A lonely tower on a floating rock, its telescope pointed at every island at once. It sees trouble coming: expiring certificates, scheduled deploys, filling disks.*

*The keeper is a heron who has not blinked since the last major outage. From the tower you can see storms forming over the void hours before they arrive. On-call note: while it's reachable, you see warnings before they turn into incidents. Lose it and you're flying blind.*

**Links to:** メインルーター Router, Banana Sector Cache, データベース Database

### 🗄️ データベース Database

`database-cluster` · database · 192.168.0.50 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=database-cluster&zoom=1.4)

**Role:** A database stores records, often as several replicas that must agree.

*Three stone towers that hold the same records, so losing one never loses a record. Keeping the three in agreement is a full-time job for a very patient sloth.*

*The three towers are called Yes, Also Yes and Probably, after how often they agree. The sloth keeps a ledger of every disagreement in his own neat handwriting. On-call note: when replicas drift, resync from the primary. Guessing which tower is right is how data gets lost.*

**Links to:** 北ファイアウォール Firewall

### 🌐 CDN Canopy

`cdn-canopy` · cdn · 10.5.0.1 · criticality 2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=cdn-canopy&zoom=1.4)

**Role:** A content delivery network caches content close to users.

*Copies of the most popular bananas, cached high in the treetops by the delta, so visitors never have to wait for the trip upstream.*

*The highest branches of the delta hold copies of everything the jungle's visitors ask for most. The monkeys who climb up to refresh them get the best view in the jungle and the worst commute. On-call note: the edge. Stampedes and bad deploys reach it first.*

**Links to:** Lilypad Load Balancer, Junction Switch, Legacy Terminal

## 🌿 Mangrove Hollow

Height 3 · jungle · free-floating

*A floating thicket of mangrove roots south-west of the lake, where the off-site backups are woven in and the crabs batch the night's work in tide pools.*

### 🌿 Mangrove Backup Roots

`mangrove-backup` · vault · 10.40.0.10 · ▶ [see it](https://go.fridge.run/luckymonkey#node=mangrove-backup&zoom=1.4)

**Role:** Backup storage. Backups only count if they can be restored.

*After the Long Night of the DNS Outage, the elders decided the jungle's backups shouldn't all live in the vault on Canopy Ridge. A copy was carried downriver and woven into the roots of a floating mangrove, where floods can't reach it and fire can't burn it.*

*The crabs who keep it are fond of saying that a backup in the same place as the original is just a second copy of the same bad day. On-call note: off-site backups only matter if they're tested and the disk isn't full. Prune the old ones.*

**Links to:** Data Lake, Tide Pool Queue

### 🦀 Tide Pool Queue

`tide-queue` · queue · 10.40.0.20 · ▶ [see it](https://go.fridge.run/luckymonkey#node=tide-queue&zoom=1.4)

**Role:** A message queue holds messages in order until consumers are ready, smoothing out bursts.

*Work that can wait is left in the tide pools, and the crabs carry it out to the reef when the water turns. The lake's analysts send their heaviest questions here at dusk and find the answers waiting at dawn.*

*On-call note: when the pool backs up, everything that writes to it backs up with it. A queue hides a slow consumer, until it doesn't.*

**Links to:** 南ファイアウォール Firewall, Firefly Log Reef

## ✨ Firefly Reef

Height 2 · jungle · free-floating

*A glowing coral reef drifting beside the river. Every log line in the jungle ends up here as a firefly, and the southern cache grows from the coral.*

### ✨ Firefly Log Reef

`firefly-logs` · database · 10.41.0.5 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=firefly-logs&zoom=1.4)

**Role:** A database stores records, often as several replicas that must agree.

*Each firefly carries one log line, and the reef glows brighter the busier the jungle has been. On bad nights the whole reef lights up like a festival, and the on-call monkeys watch it from the lake and wince.*

*On-call note: logs grow faster than anyone budgets for, and a full disk stops writes. The reef's replica falls behind when the tide is high; resync it from the primary, never the other way round.*

**Links to:** Coral Edge Cache

### 🪸 Coral Edge Cache

`reef-cache` · banana-cache · 10.41.0.9 · ▶ [see it](https://go.fridge.run/luckymonkey#node=reef-cache&zoom=1.4)

**Role:** A cache keeps copies of frequently used data close by, so requests don't have to go all the way to the database.

*The southern islands are far from the delta, so they grew their own cache, one coral polyp at a time. It holds the bananas the south asks for most and almost nothing else.*

*On-call note: being far from everything makes it a quiet target, until a stampede notices it.*

**Links to:** Heron Relay

## 🪶 Heron Rock

Height 4 · jungle · free-floating

*The last rock before the river leaves the map. A single relay, a single heron, and a long rope back to the lake.*

### 🪶 Heron Relay

`heron-relay` · switch · 10.42.0.1 · ▶ [see it](https://go.fridge.run/luckymonkey#node=heron-relay&zoom=1.4)

**Role:** A switch connects devices on the same network and forwards frames by MAC address. Loops between switches cause broadcast storms unless Spanning Tree blocks them.

*The heron stands on one leg on the last rock before the river leaves the known jungle, passing frames between the southern islands and the lake. She says she has seen where the river goes. She has never said where that is.*

*On-call note: a switch at the end of a long rope. Loops and wrong VLANs here cut off the whole southern chain.*

**Links to:** Data Lake

Next: [Failure types](failures.md) · Back to the [documentation](index.md)
