---
title: Lucky Monkey - failure types
---

# Failure types

> Eighteen ways a jungle network breaks, each a real production failure in
> costume, each with a real first move.

**Generated from [`game/data/tuning.json`](build-deploy.md#live_data) by `tools/gen-docs.py`**, so every number and sentence here is exactly what the game uses. Edit the JSON, run `tools/deploy.sh data`, and the game changes in seconds; the next full deploy regenerates this page.

How to read a fault:
- **Symptom** is what the panel says. It always contains the clue.
- A **warning stage** (📅 🧬 🔐 💾 …) means you can see it coming and *prevent* it: nothing is broken yet.
- **Fix** removes the root cause. **Mitigate** buys the stated seconds, then the fault comes back. **No effect** costs you the time and your combo, and the game explains why it didn't help.
- 🔍 **Diagnose** shows what every option would do before you commit, and teaches the concept.
- **Time to Loss** is the game's own countdown to the next consequence ([not IP TTL or DNS TTL](glossary.md)). Levels scale it (training ×1.3, shift 3+ ×0.85, practice ×3).

Every section has a ▶ **live** link that starts a shift with that fault staged on a real node.

## At a glance

| Fault | Group | Time to Loss | Fix | Mitigate | Can be prevented? | Spreads? |
|---|---|---|---|---|---|---|
| 💥 [Service crashed](#service_crashed) | systems | 60 s | 🔄 Restart the service | — | no | no |
| 📛 [DNS resolution failing](#dns_resolution_failing) | network | 55 s | 📛 Restore the DNS resolver config | 🔄 Restart the service (25 s) | no | no |
| 🏷️ [DHCP pool exhausted](#dhcp_pool_exhausted) | network | 60 s | 🏷️ Expand the DHCP pool | — | no | no |
| 🐊 [Damaged cable](#damaged_cable) | systems | 60 s | 🔌 Replace the cable | — | no | no |
| 🧭 [Missing route](#missing_route) | network | 50 s | 🧭 Correct the routing table | 🔀 Fail over to standby (30 s) | no | no |
| 🐘 [DDoS flood](#ddos_flood) | security | 40 s | 🚫 Block attackers with an ACL | 🛡️ Rate-limit traffic (40 s) | no | yes, once |
| 🐛 [Worm infection](#worm_infection) | security | 50 s | 🩹 Install the patch | 🚧 Quarantine on an isolated VLAN (45 s) | yes: 🧬 → 🩹 Install the patch | yes, once |
| 🔐 [Expired TLS certificate](#expired_tls_certificate) | security | 50 s | 🔑 Renew the TLS certificate | — | yes: 🔐 → 🔑 Renew the TLS certificate | no |
| 📈 [Memory leak](#memory_leak) | systems | 55 s | 🩹 Install the patch | 🔄 Restart the service (45 s), 🔀 Fail over to standby (60 s) | no | no |
| 💾 [Disk full](#disk_full) | systems | 50 s | 🧹 Clear old snapshots | — | yes: 💾 → 🧹 Clear old snapshots | no |
| 🌡️ [Thermal throttling](#thermal_throttling) | systems | 55 s | 🍃 Clear the vents | 🔄 Restart the service (20 s) | no | no |
| ⚡ [Power supply failed](#power_supply_failed) | systems | 35 s | 🔀 Fail over to standby | — | no (lightning) | no |
| 📦 [Bad deploy](#bad_deploy) | operations | 55 s | ⏪ Roll back the deploy | — | yes: 📅 → 🐤 Canary the deploy | no |
| 📦 [Bad deploy with migration](#bad_deploy_with_migration) | operations | 55 s | ⏩ Fall forward: ship a hotfix | — | no | no |
| ⚖️ [Replica disparity](#replica_disparity) | operations | 60 s | 🔁 Resync replicas from primary | — | no | no |
| 🔁 [Routing loop](#routing_loop) | advanced | 45 s | 🧭 Correct the routing table | 🔄 Restart the service (20 s) | no | no |
| 📢 [Broadcast storm](#broadcast_storm) | advanced | 40 s | 🌲 Enable Spanning Tree | — | no | no |
| 🏝️ [VLAN misconfiguration](#vlan_misconfiguration) | advanced | 55 s | 🏝️ Move ports to the right VLAN | — | no | no |

## Service crashed

💥 `crash` · systems · **60 s** to loss · hits: any node · ▶ [live on Lucky Monkey Guardian](https://go.fridge.run/luckymonkey#node=monkey-statue&incident=crash&zoom=1.2)

**Symptom:** The service process exited. The machine still answers ping, but its port is closed.

| Response | Effect | Why |
|---|---|---|
| 🔄 Restart the service (3 s of work) | ✅ fix | The process is back; the logs show a one-off crash. |
| 🧹 Clear old snapshots | ✖ no effect | A real tool for a different problem. |
| 🔑 Renew the TLS certificate | ✖ no effect | A real tool for a different problem. |

**Concept, Processes and ports:** A service is a running process listening on a port. The machine can be up (it answers ping) while the service is dead.

## DNS resolution failing

📛 `dns` · network · **55 s** to loss · hits: `dns` · ▶ [live on DNS Shrine](https://go.fridge.run/luckymonkey#node=dns-shrine&incident=dns&zoom=1.2)

**Symptom:** Clients can't resolve hostnames. Connecting by IP address still works, but nobody remembers the numbers.

| Response | Effect | Why |
|---|---|---|
| 📛 Restore the DNS resolver config (4 s of work) | ✅ fix | Resolver configuration restored; names resolve again. |
| 🔄 Restart the service (3 s of work) | 🩹 mitigate, 25 s | The restart reloaded the cache, but the bad config returns as records expire. |
| 🔌 Replace the cable | ✖ no effect | A real tool for a different problem. |
| 🏷️ Expand the DHCP pool | ✖ no effect | A real tool for a different problem. |

**Concept, DNS:** The Domain Name System turns names into IP addresses. If it fails, connections by IP still work, but names don't.

## DHCP pool exhausted

🏷️ `dhcp` · network · **60 s** to loss · hits: `river`, `lake`, `switch`, `lizard`, `terminal` · ▶ [live on Logical Lagoon API](https://go.fridge.run/luckymonkey#node=logical-lagoon&incident=dhcp&zoom=1.2)

**Symptom:** No addresses left in the pool. New devices get no lease and fall back to 169.254.x.x, which can't reach anything.

| Response | Effect | Why |
|---|---|---|
| 🏷️ Expand the DHCP pool (4 s of work) | ✅ fix | The pool is widened from /26 to /24; new devices get leases. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | Restarting doesn't create addresses: the pool is still full. |
| 📛 Restore the DNS resolver config | ✖ no effect | A real tool for a different problem. |

**Concept, DHCP:** DHCP hands out IP addresses from a pool, as time-limited leases. With an empty pool, new devices get no address.

## Damaged cable

🐊 `cable` · systems · **60 s** to loss · hits: `switch`, `router`, `balancer`, `cdn`, `server` · ▶ [live on Crashed Sector 404](https://go.fridge.run/luckymonkey#node=crashed-sector&incident=cable&zoom=1.2)

**Symptom:** 20% packet loss and CRC errors on one port. A crocodile chewed the patch cable.

| Response | Effect | Why |
|---|---|---|
| 🔌 Replace the cable (5 s of work) | ✅ fix | New cable fitted; the CRC errors are gone. It was the physical layer. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | Software restarts don't fix a damaged cable (layer 1). |
| 🧭 Correct the routing table | ✖ no effect | A real tool for a different problem. |

**Concept, Physical layer:** Layer 1: cables, ports and signals. CRC errors and packet loss on one port often mean a bad cable, which no software fixes.

## Missing route

🧭 `route` · network · **50 s** to loss · hits: `router`, `firewall` · ▶ [live on メインルーター Router](https://go.fridge.run/luckymonkey#node=main-router&incident=route&zoom=1.2)

**Symptom:** The router lost its route to the southern islands; packets come back as ICMP 'destination unreachable'.

| Response | Effect | Why |
|---|---|---|
| 🧭 Correct the routing table (5 s of work) | ✅ fix | The static route is restored; traffic flows again. |
| 🔀 Fail over to standby (4 s of work) | 🩹 mitigate, 30 s | The standby router works for now, but its routing table is stale. |
| 📛 Restore the DNS resolver config | ✖ no effect | A real tool for a different problem. |
| 🛡️ Rate-limit traffic | ✖ no effect | A real tool for a different problem. |

**Concept, Routing:** Routers forward packets using a routing table. Without a route, the packet is dropped and ICMP reports 'destination unreachable'.

## DDoS flood

🐘 `ddos` · security · **40 s** to loss · hits: `cdn`, `balancer`, `auth`, `firewall`, `banana-cache`, `river`, `lake` · ▶ [live on Logical Lagoon API](https://go.fridge.run/luckymonkey#node=logical-lagoon&incident=ddos&zoom=1.2)

**Symptom:** Requests ×40 from thousands of sources. A stampede is hammering one endpoint.

| Response | Effect | Why |
|---|---|---|
| 🚫 Block attackers with an ACL (5 s of work) | ✅ fix | Firewall ACL drops the attacking sources; real users get through. |
| 🛡️ Rate-limit traffic (3 s of work) | 🩹 mitigate, 40 s | Rate limiting buys time but also turns away some real users. The attack continues. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | Restarting doesn't stop traffic that is still arriving. |
| ⏪ Roll back the deploy | ✖ no effect | A real tool for a different problem. |

**Spreads:** 30 s after its node goes down, it jumps once to a healthy neighbour.

**Concept, DDoS:** A distributed denial of service floods a target from many sources. Rate limits buy time; filtering the sources (ACLs) stops it.

## Worm infection

🐛 `worm` · security · **50 s** to loss · hits: `server`, `terminal`, `ai-core`, `temple`, `vault`, `queue` · ▶ [live on Node Temple Alpha](https://go.fridge.run/luckymonkey#node=node-temple-1&incident=worm&zoom=1.2)

**Symptom:** An unknown process copies itself over every connection it can reach.

**Warning stage** 🧬 (75 s): A worm signature is circulating and this node is unpatched. Prevent with 🩹 Install the patch. ▶ [live](https://go.fridge.run/luckymonkey#node=node-temple-1&forecast=worm&zoom=1.2)

| Response | Effect | Why |
|---|---|---|
| 🩹 Install the patch (5 s of work) | ✅ fix | Patched and cleaned; the hole it used is closed. |
| 🚧 Quarantine on an isolated VLAN (3 s of work) | 🩹 mitigate, 45 s | Isolated on its own VLAN: it can't spread, but it's still infected. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | The worm survives reboots: it's in the startup scripts. |

**Spreads:** 30 s after its node goes down, it jumps once to a healthy neighbour.

**Concept, Patching and segmentation:** Worms exploit unpatched holes and spread across the network. Isolating a node (a separate VLAN) stops the spread; patching removes the hole.

## Expired TLS certificate

🔐 `cert` · security · **50 s** to loss · hits: `auth`, `cdn`, `queue`, `firewall`, `balancer` · ▶ [live on 北ファイアウォール Firewall](https://go.fridge.run/luckymonkey#node=firewall-north&incident=cert&zoom=1.2)

**Symptom:** Clients reject the TLS handshake: the certificate's validity date has passed.

**Warning stage** 🔐 (80 s): The TLS certificate expires soon. Renew it now and nobody notices. Prevent with 🔑 Renew the TLS certificate. ▶ [live](https://go.fridge.run/luckymonkey#node=firewall-north&forecast=cert&zoom=1.2)

| Response | Effect | Why |
|---|---|---|
| 🔑 Renew the TLS certificate (3 s of work) | ✅ fix | New certificate installed; handshakes succeed. |
| 🔓 Disable TLS (1 s of work) | ✖ no effect | Refused: turning off TLS would send passwords in clear text. Renew the certificate instead. |
| 🔄 Restart the service | ✖ no effect | A real tool for a different problem. |

**Concept, TLS certificates:** TLS encrypts connections and proves identity with a certificate that expires. Renew it before it does; never just turn encryption off.

## Memory leak

📈 `leak` · systems · **55 s** to loss · hits: `server`, `ai-core`, `terminal`, `queue`, `monkey` · ▶ [live on Lucky Monkey Guardian](https://go.fridge.run/luckymonkey#node=monkey-statue&incident=leak&zoom=1.2)

**Symptom:** RAM use grows 2 GB a minute and the system is swapping. Something never frees its memory.

| Response | Effect | Why |
|---|---|---|
| 🩹 Install the patch (5 s of work) | ✅ fix | The code that never freed memory is fixed. |
| 🔄 Restart the service (3 s of work) | 🩹 mitigate, 45 s | Memory is freed for now; the leak will fill RAM again. |
| 🔀 Fail over to standby (4 s of work) | 🩹 mitigate, 60 s | The standby takes over with clean memory; the same bug leaks there too. |

**Concept, Memory and swap:** A leak keeps claiming RAM until the system swaps to disk and crawls. A restart frees it for a while; only a code fix stops it.

## Disk full

💾 `disk` · systems · **50 s** to loss · hits: `database`, `vault`, `lake`, `banana-cache`, `queue` · ▶ [live on Banana Sector Cache](https://go.fridge.run/luckymonkey#node=banana-sector&incident=disk&zoom=1.2)

**Symptom:** Disk at 100%. Writes fail and logs can't rotate.

**Warning stage** 💾 (75 s): Disk at 85% and climbing: old snapshots are piling up. Prevent with 🧹 Clear old snapshots. ▶ [live](https://go.fridge.run/luckymonkey#node=banana-sector&forecast=disk&zoom=1.2)

| Response | Effect | Why |
|---|---|---|
| 🧹 Clear old snapshots (4 s of work) | ✅ fix | Old snapshots removed; writes work again. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | Rebooting doesn't create disk space. |

**Concept, Storage capacity:** A full disk makes every write fail. Rebooting doesn't free space: delete or move data, or add capacity.

## Thermal throttling

🌡️ `heat` · systems · **55 s** to loss · hits: `server`, `ai-core`, `volcano`, `vault` · ▶ [live on Crashed Sector 404](https://go.fridge.run/luckymonkey#node=crashed-sector&incident=heat&zoom=1.2)

**Symptom:** CPU at 98 °C and throttling its clock speed: the vents are clogged with leaves.

| Response | Effect | Why |
|---|---|---|
| 🍃 Clear the vents (4 s of work) | ✅ fix | Vents cleared; the temperature is back to 55 °C. |
| 🔄 Restart the service (3 s of work) | 🩹 mitigate, 20 s | It cooled while rebooting; it will heat up again. |

**Concept, Thermal throttling:** Hot CPUs slow themselves down to survive. Cooling (airflow, clean vents) is the fix.

## Power supply failed

⚡ `power` · systems · **35 s** to loss · hits: any node · ▶ [live on Lucky Monkey Guardian](https://go.fridge.run/luckymonkey#node=monkey-statue&incident=power&zoom=1.2)

**Symptom:** Lightning hit the rack and the primary power supply is dead. The redundant one is idle.

| Response | Effect | Why |
|---|---|---|
| 🔀 Fail over to standby (4 s of work) | ✅ fix | Switched to the redundant power supply. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | Nothing to restart: there's no power. |

**Concept, Redundancy and failover:** A spare component (a second power supply, a standby server) takes over when the primary fails, if one was there to begin with.

## Bad deploy

📦 `deploy` · operations · **55 s** to loss · hits: `server`, `auth`, `cdn`, `queue`, `banana-cache`, `ai-core` · ▶ [live on Banana Sector Cache](https://go.fridge.run/luckymonkey#node=banana-sector&incident=deploy&zoom=1.2)

**Symptom:** v2.4 went out and the error rate is 30%. There was no schema change: v2.3 is still compatible.

**Warning stage** 📅 (80 s): A deploy of v2.4 is scheduled. Canary it first: a bad build then reaches only a few users. Prevent with 🐤 Canary the deploy. ▶ [live](https://go.fridge.run/luckymonkey#node=banana-sector&forecast=deploy&zoom=1.2)

| Response | Effect | Why |
|---|---|---|
| ⏪ Roll back the deploy (5 s of work) | ✅ fix | Back on v2.3, which is compatible: the errors stop. |
| ⏩ Fall forward: ship a hotfix (8 s of work) | ✖ no effect | No hotfix is ready yet. v2.3 is compatible, so rolling back is faster and safe. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | Restarting runs the same broken v2.4. |

**Concept, Rollback:** Undo a bad change by going back to the previous version, as long as that version still works with today's data.

## Bad deploy with migration

📦 `migration` · operations · **55 s** to loss · hits: any node · ▶ [live on Lucky Monkey Guardian](https://go.fridge.run/luckymonkey#node=monkey-statue&incident=migration&zoom=1.2)

**Symptom:** v2.4 migrated the database schema and the error rate is 30%. v2.3 can't read the new tables.

| Response | Effect | Why |
|---|---|---|
| ⏩ Fall forward: ship a hotfix (8 s of work) | ✅ fix | Hotfix v2.4.1 shipped; it works with the new schema. |
| ⏪ Roll back the deploy (5 s of work) | ✖ no effect | v2.3 can't read the migrated schema: rolling back would break it further. |
| 🔄 Restart the service | ✖ no effect | A real tool for a different problem. |

**Concept, Schema migrations:** A deploy that changes the database schema can't simply be rolled back: the old version can't read the new tables. Fix forward.

## Replica disparity

⚖️ `disparity` · operations · **60 s** to loss · hits: `database`, `vault`, `lake`, `banana-cache` · ▶ [live on Banana Sector Cache](https://go.fridge.run/luckymonkey#node=banana-sector&incident=disparity&zoom=1.2)

**Symptom:** The replicas' checksums disagree: two copies report two different banana counts.

| Response | Effect | Why |
|---|---|---|
| 🔁 Resync replicas from primary (7 s of work) | ✅ fix | Replicas re-copied from the primary; the checksums match. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | Restarting doesn't reconcile the data. |

**Concept, Replication:** Replicas are copies of a database. When they disagree, re-copy them from the authoritative primary.

## Routing loop

🔁 `loop` · advanced · **45 s** to loss · hits: `router`, `switch` · ▶ [live on メインルーター Router](https://go.fridge.run/luckymonkey#node=main-router&incident=loop&zoom=1.2)

**Symptom:** Packets circle between two routers until their IP TTL (a hop count) reaches 0 and they're dropped.

| Response | Effect | Why |
|---|---|---|
| 🧭 Correct the routing table (5 s of work) | ✅ fix | The route that pointed back is fixed; packets reach their destination. |
| 🔄 Restart the service (3 s of work) | 🩹 mitigate, 20 s | Routes were relearned, but the bad static route returns. |

**Concept, IP TTL:** Each IP packet carries a TTL hop count; every router subtracts 1, and at 0 the packet is dropped. That stops a routing loop from circling forever.

## Broadcast storm

📢 `broadcast` · advanced · **40 s** to loss · hits: `switch`, `balancer` · ▶ [live on Junction Switch](https://go.fridge.run/luckymonkey#node=switch-01&incident=broadcast&zoom=1.2)

**Symptom:** A looped cable floods the switch with broadcasts; Spanning Tree was turned off on this switch.

| Response | Effect | Why |
|---|---|---|
| 🌲 Enable Spanning Tree (4 s of work) | ✅ fix | STP blocked the redundant port; the storm stops. |
| 🔌 Replace the cable (5 s of work) | ✖ no effect | The cables are fine; it's the loop they form. |

**Concept, Spanning Tree:** STP finds loops between switches and blocks a redundant port, so broadcasts can't circle and multiply into a storm.

## VLAN misconfiguration

🏝️ `vlan` · advanced · **55 s** to loss · hits: `switch`, `firewall` · ▶ [live on 北ファイアウォール Firewall](https://go.fridge.run/luckymonkey#node=firewall-north&incident=vlan&zoom=1.2)

**Symptom:** Ports were put on the wrong VLAN: the tree frogs can't reach their default gateway.

| Response | Effect | Why |
|---|---|---|
| 🏝️ Move ports to the right VLAN (4 s of work) | ✅ fix | Ports are back on VLAN 20; the gateway answers. |
| 🔄 Restart the service (3 s of work) | ✖ no effect | The port configuration survives a restart. |

**Concept, VLANs:** VLANs split one switch into separate networks. A port on the wrong VLAN can't reach its own subnet's default gateway.

## Traps

- 🔓 Disable TLS: the game refuses it and explains why turning off encryption is never a fix.

## Lightning

Storm clouds flash and strike a node within range: a **Power supply failed** fault (above). It goes through the same director budget as every other fault, so a storm never piles on top of a bad moment.

Next: [Nodes and lore](nodes.md) · Back to the [documentation](index.md)
