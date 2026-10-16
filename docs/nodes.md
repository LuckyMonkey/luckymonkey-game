---
title: Lucky Monkey - nodes and lore
---

# Nodes and lore

> Twenty-eight places in the jungle, each a real kind of infrastructure, each with a story.

**Generated from [`game/data/nodes.json`](world.md) and [`islands.json`](world.md#the_archipelago) by `tools/gen-docs.py`.** In the game, press **T** (outside a shift) for the same tour, island by island, with what each node links to and how many services would lose their path if it went down.

## 🫧 Lagoon Heights

Height 9 · jungle · on the river

### 🌊 Logical Lagoon API

`logical-lagoon` · river · 192.168.1.42 · ▶ [see it](https://go.fridge.run/luckymonkey#node=logical-lagoon&zoom=1.4)

**Role:** A network segment where devices join, get addresses (DHCP) and talk to each other.

*The river slows here into a warm pool where APIs float on lotus pads. The frogs say the water remembers every request ever made of it. They also hand out addresses to anything new that splashes in.*

**Links to:** Lucky Monkey Guardian, Digital River Stream

### 📨 Message Queue Falls

`mq-falls` · queue · 10.2.0.5 · ▶ [see it](https://go.fridge.run/luckymonkey#node=mq-falls&zoom=1.4)

**Role:** A message queue holds messages in order until consumers are ready, smoothing out bursts.

*Messages queue in the pool at the top of the falls and drop one at a time, in order, so nothing downstream gets swamped.*

**Links to:** Logical Lagoon API, 北ファイアウォール Firewall, Node Temple Beta

### 🦎 Reptile Protocol

`lizard-node` · lizard · 10.0.2.42 · ▶ [see it](https://go.fridge.run/luckymonkey#node=lizard-node&zoom=1.4)

**Role:** A monitoring probe watching a segment.

*A cold-blooded monitor that lies very still on a warm rock and notices everything. It reports nothing until it is sure, which is either wisdom or laziness.*

**Links to:** Lucky Monkey Guardian, Digital River Stream

## 🌳 Canopy Ridge

Height 7 · jungle · on the river, vine bridge to Digital Shore

### 🔥 北ファイアウォール Firewall

`firewall-north` · firewall · 192.168.0.10 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=firewall-north&zoom=1.4)

**Role:** A firewall allows or blocks traffic by rules (ACLs). The first line against attacks.

*A wall of thorny vines that lets the right traffic through and stings everything else. It runs rules written so long ago that nobody dares delete one.*

**Links to:** メインルーター Router, データベース Database

### 🏯 Node Temple Beta

`node-temple-2` · temple · 10.0.0.2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=node-temple-2&zoom=1.4)

**Role:** An application server: runs services that other nodes call.

*Its door opens only for a hash nobody remembers choosing. The priests keep a backup of the hash on a stone tablet in the vault, which is locked with the same hash.*

**Links to:** Banana Sector Cache, Server Hub Cluster

### 💻 Server Hub Cluster

`server-hub` · server · 192.168.1.200 · ▶ [see it](https://go.fridge.run/luckymonkey#node=server-hub&zoom=1.4)

**Role:** A general-purpose server running services. Can crash, leak memory or overheat.

*Rows of humming racks under the canopy, warm enough that the parrots nest on top of them in the rainy season. Most of the jungle's work actually happens here.*

**Links to:** Node Temple Beta

### 🧱 Backup Vault Node

`vault-01` · vault · 10.0.0.3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=vault-01&zoom=1.4)

**Role:** Backup storage. Backups only count if they can be restored.

*The backups live here, behind three doors and one very suspicious toucan. A backup nobody has ever restored is just a hope, so the toucan restores one every Friday.*

**Links to:** Neural Processor Alpha

## 🛕 Temple Isle

Height 5 · temple · on the river, vine bridge to Banana Grove

### 🐵 Lucky Monkey Guardian

`monkey-statue` · monkey · 192.168.1.1 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=monkey-statue&zoom=1.4)

**Role:** The Guardian: a core hub. Many paths run through it.

*Carved by the first sysadmins from a single block of jade, the Guardian sits at the top of the pyramid and watches every packet with half-closed eyes. Monkeys leave a banana at its feet before every deploy. Nobody admits to believing it helps.*

**Links to:** Logical Lagoon API, Banana Sector Cache, Node Temple Alpha, Volcano Core Processor

### 📡 メインルーター Router

`main-router` · router · 192.168.0.1 · criticality 5 · ▶ [see it](https://go.fridge.run/luckymonkey#node=main-router&zoom=1.4)

**Role:** A router forwards packets between networks using its routing table. If it fails, everything that reaches the rest of the network through it is cut off.

*Perched on the highest ledge of Temple Isle, the router knows a path to every island. If it goes quiet, the islands can still see each other across the void but have no way to talk.*

**Links to:** Lucky Monkey Guardian, 北ファイアウォール Firewall, 南ファイアウォール Firewall

### ⛩️ DNS Shrine

`dns-shrine` · dns · 10.0.0.53 · criticality 4 · ▶ [see it](https://go.fridge.run/luckymonkey#node=dns-shrine&zoom=1.4)

**Role:** A DNS server turns names into IP addresses. Without it, names stop working even though the network is fine.

*Monks chant the names of every island and the numbers they stand for. Visitors ask for a name and leave with a number. When the monks forget, nobody can find anything.*

**Links to:** Lucky Monkey Guardian, Node Temple Alpha, Reptile Protocol, メインルーター Router

### 🧠 Neural Processor Alpha

`ai-core-01` · ai-core · 10.0.1.42 · ▶ [see it](https://go.fridge.run/luckymonkey#node=ai-core-01&zoom=1.4)

**Role:** A compute node running heavy workloads.

*A thinking stone that sorts the jungle's problems by how weird they are. It once predicted a disk would fill on a Tuesday. It was a Wednesday, but everyone was impressed.*

**Links to:** Lucky Monkey Guardian, Backup Vault Node

## 💾 Digital Shore

Height 3 · jungle · on the river, vine bridge to Canopy Ridge

### 🌊 Digital River Stream

`digital-river` · river · 192.168.1.101 · ▶ [see it](https://go.fridge.run/luckymonkey#node=digital-river&zoom=1.4)

**Role:** A network segment where devices join, get addresses (DHCP) and talk to each other.

*A fast, shallow channel that carries raw signal from the lagoon to the shore. On calm days you can see the frames glinting under the surface like minnows.*

**Links to:** Logical Lagoon API, Crashed Sector 404

### 🏯 Node Temple Alpha

`node-temple-1` · temple · 10.0.0.1 · ▶ [see it](https://go.fridge.run/luckymonkey#node=node-temple-1&zoom=1.4)

**Role:** An application server: runs services that other nodes call.

*The older of the two temples, built before anyone wrote things down. Ghostpackets still sweep its halls at night, delivering messages addressed to people who left long ago.*

**Links to:** Lucky Monkey Guardian, Crashed Sector 404

### 🔥 南ファイアウォール Firewall

`firewall-south` · firewall · 192.168.0.11 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=firewall-south&zoom=1.4)

**Role:** A firewall allows or blocks traffic by rules (ACLs). The first line against attacks.

*The southern wall is younger and stricter. It checks every packet twice, which the southern islanders call security and everyone else calls slow.*

**Links to:** メインルーター Router

## 🍌 Banana Grove

Height 5 · grove · on the river, vine bridge to Temple Isle

### 🍌 Banana Sector Cache

`banana-sector` · banana-cache · 192.168.1.99 · ▶ [see it](https://go.fridge.run/luckymonkey#node=banana-sector&zoom=1.4)

**Role:** A cache keeps copies of frequently used data close by, so requests don't have to go all the way to the database.

*Every banana the jungle has ever needed twice is kept here, ripe and ready, so nobody has to climb to the archive again. When it forgets one, the whole jungle notices the walk.*

**Links to:** Lucky Monkey Guardian, Node Temple Beta

### 🍌 Humor Cache Alpha

`banana-cache-01` · banana-cache · 10.0.0.4 · ▶ [see it](https://go.fridge.run/luckymonkey#node=banana-cache-01&zoom=1.4)

**Role:** A cache keeps copies of frequently used data close by, so requests don't have to go all the way to the database.

*A small cache of jokes the monkeys tell during long outages. It is surprisingly critical infrastructure.*

**Links to:** Junction Switch, Legacy Terminal

### 🛂 Auth Gate

`auth-gate` · auth · 10.3.0.1 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=auth-gate&zoom=1.4)

**Role:** Authentication: checks identities and holds TLS certificates others trust.

*A carved gate where every visitor shows their papers. It also keeps the certificates the rest of the jungle trusts, which makes it a very popular target.*

**Links to:** Backup Vault Node, Neural Processor Alpha, Humor Cache Alpha, メインルーター Router

### 📚 Ancient Data Archive

`data-archive` · database · 10.0.1.42 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=data-archive&zoom=1.4)

**Role:** A database stores records, often as several replicas that must agree.

*The deepest storage in the jungle: every log, every snapshot, every version of every banana count. It fills up slowly and then all at once.*

**Links to:** Banana Sector Cache

## 🌊 Data Lake

Height 1 · jungle · on the river

### 🏞️ Data Lake

`data-lake` · lake · 10.4.0.1 · ▶ [see it](https://go.fridge.run/luckymonkey#node=data-lake&zoom=1.4)

**Role:** A data lake: large, cheap storage for raw data and analytics.

*Where the river finally stops. Every frame that ever flowed downstream settles here, and the analysts fish for patterns from the shore.*

**Links to:** Digital River Stream, Legacy Terminal, 南ファイアウォール Firewall

### 💻 Legacy Terminal

`terminal-01` · terminal · 10.0.0.5 · ▶ [see it](https://go.fridge.run/luckymonkey#node=terminal-01&zoom=1.4)

**Role:** A client machine: a user's view of the network.

*An old green-screen terminal at the end of the lake path. Its commands still work, mostly, as long as you type them in capital letters.*

**Links to:** Humor Cache Alpha

### 🔀 Junction Switch

`switch-01` · switch · 10.0.0.2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=switch-01&zoom=1.4)

**Role:** A switch connects devices on the same network and forwards frames by MAC address. Loops between switches cause broadcast storms unless Spanning Tree blocks them.

*Every cable on the shore meets at this junction, where a switch decides which port each frame goes out of. Plug two of its ports into each other and the frames chase themselves forever.*

**Links to:** Humor Cache Alpha

### 🪷 Lilypad Load Balancer

`lb-lilypads` · balancer · 10.1.0.10 · criticality 2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=lb-lilypads&zoom=1.4)

**Role:** A load balancer spreads requests across several servers so none is overloaded.

*Frames cross the river by hopping pad to pad. The pads take turns carrying them, so no single pad sinks under the weight.*

**Links to:** メインルーター Router, Junction Switch, Banana Sector Cache, CDN Canopy

## 🌋 Ember Isle

Height 6 · volcano · free-floating

### 🌋 Volcano Core Processor

`volcano-core` · volcano · 192.168.1.666 · ▶ [see it](https://go.fridge.run/luckymonkey#node=volcano-core&zoom=1.4)

**Role:** A high-performance compute node: fast, hot, and sensitive to cooling.

*The fastest processor in the jungle runs inside a live volcano, cooled by the river and good intentions. It throttles itself whenever the leaves pile up on the vents.*

**Links to:** Lucky Monkey Guardian

### 💻 Crashed Sector 404

`crashed-sector` · server · 192.168.1.404 · ▶ [see it](https://go.fridge.run/luckymonkey#node=crashed-sector&zoom=1.4)

**Role:** A general-purpose server running services. Can crash, leak memory or overheat.

*Something fell out of the sky here once and never booted again. The monkeys keep it around as a reminder to take backups.*

**Links to:** Node Temple Alpha, Digital River Stream

### ⚛️ Quantum Nexus Core

`quantum-nexus` · server · 192.168.q.∞ · ▶ [see it](https://go.fridge.run/luckymonkey#node=quantum-nexus&zoom=1.4)

**Role:** A general-purpose server running services. Can crash, leak memory or overheat.

*An experimental machine on Ember Isle that is both running and not running until someone checks. The on-call monkey would rather it just picked one.*

**Links to:** Node Temple Alpha

## 🔭 Watch Isle

Height 8 · jungle · free-floating

### 🔭 Canopy Watchtower

`watchtower` · monitor · 10.9.0.1 · criticality 2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=watchtower&zoom=1.4)

**Role:** Monitoring: collects metrics and warns before things break. If it's unreachable you lose your warnings.

*A lonely tower on a floating rock, its telescope pointed at every island at once. It sees trouble coming: expiring certificates, scheduled deploys, filling disks.*

**Links to:** メインルーター Router, Banana Sector Cache, データベース Database

### 🗄️ データベース Database

`database-cluster` · database · 192.168.0.50 · criticality 3 · ▶ [see it](https://go.fridge.run/luckymonkey#node=database-cluster&zoom=1.4)

**Role:** A database stores records, often as several replicas that must agree.

*Three stone towers that hold the same records, so losing one never loses a record. Keeping the three in agreement is a full-time job for a very patient sloth.*

**Links to:** 北ファイアウォール Firewall

### 🌐 CDN Canopy

`cdn-canopy` · cdn · 10.5.0.1 · criticality 2 · ▶ [see it](https://go.fridge.run/luckymonkey#node=cdn-canopy&zoom=1.4)

**Role:** A content delivery network caches content close to users.

*Copies of the most popular bananas, cached high in the treetops by the delta, so visitors never have to wait for the trip upstream.*

**Links to:** Lilypad Load Balancer, Junction Switch, Legacy Terminal

Next: [Failure types](failures.md) · Back to the [documentation](index.md)
