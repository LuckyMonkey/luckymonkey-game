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
- 🔍 **Diagnose** shows what every option would do before you commit, and teaches the concept. A 🍌 **hint** just marks the right one.
- **How it started:** every incident opens with a plausible origin story (one is picked from the list below).
- **Spread:** left alone, an incident hurts its neighbours as its Time to Loss runs down (at ⅔ and ⅓ left, and again when it goes down). They show a 📉 **symptom** with nothing to fix locally, and clear the moment the origin is fixed. Worms are contagious instead: they infect neighbours for real.
- **Time to Loss** is the game's own countdown to the next consequence ([not IP TTL or DNS TTL](glossary.md)). Levels scale it (training ×1.3, shift 3+ ×0.85, practice ×3).

Every section has a ▶ **live** link that starts a shift with that fault staged on a real node.

## At a glance

| Fault | Group | Time to Loss | Fix | Mitigate | Can be prevented? | Spreads? |
|---|---|---|---|---|---|---|
| 💥 [Service crashed](#service_crashed) | systems | 60 s | 🔄 Restart the service | — | no | symptoms |
| 📛 [DNS resolution failing](#dns_resolution_failing) | network | 55 s | 📛 Restore the DNS resolver config | 🔄 Restart the service (25 s) | no | symptoms |
| 🏷️ [DHCP pool exhausted](#dhcp_pool_exhausted) | network | 60 s | 🏷️ Expand the DHCP pool | — | no | symptoms |
| 🐊 [Damaged cable](#damaged_cable) | systems | 60 s | 🔌 Replace the cable | — | no | symptoms |
| 🧭 [Missing route](#missing_route) | network | 50 s | 🧭 Correct the routing table | 🔀 Fail over to standby (30 s) | no | symptoms |
| 🦬 [DDoS flood](#ddos_flood) | security | 40 s | 🚫 Block attackers with an ACL | 🛡️ Rate-limit traffic (40 s) | no | symptoms |
| 🐛 [Worm infection](#worm_infection) | security | 50 s | 🩹 Install the patch | 🚧 Quarantine on an isolated VLAN (45 s) | yes: 🧬 → 🩹 Install the patch | contagious |
| 🔐 [Expired TLS certificate](#expired_tls_certificate) | security | 50 s | 🔑 Renew the TLS certificate | — | yes: 🔐 → 🔑 Renew the TLS certificate | symptoms |
| 📈 [Memory leak](#memory_leak) | systems | 55 s | 🩹 Install the patch | 🔄 Restart the service (45 s), 🔀 Fail over to standby (60 s) | no | symptoms |
| 💾 [Disk full](#disk_full) | systems | 50 s | 🧹 Clear old snapshots | — | yes: 💾 → 🧹 Clear old snapshots | symptoms |
| 🌡️ [Thermal throttling](#thermal_throttling) | systems | 55 s | 🍃 Clear the vents | 🔄 Restart the service (20 s) | no | symptoms |
| ⚡ [Power supply failed](#power_supply_failed) | systems | 35 s | 🔀 Fail over to standby | — | no (lightning) | symptoms |
| 📦 [Bad deploy](#bad_deploy) | operations | 55 s | ⏪ Roll back the deploy | — | yes: 📅 → 🐤 Canary the deploy | symptoms |
| 📦 [Bad deploy with migration](#bad_deploy_with_migration) | operations | 55 s | ⏩ Fall forward: ship a hotfix | — | no | symptoms |
| ⚖️ [Replica disparity](#replica_disparity) | operations | 60 s | 🔁 Resync replicas from primary | — | no | symptoms |
| 🔁 [Routing loop](#routing_loop) | advanced | 45 s | 🧭 Correct the routing table | 🔄 Restart the service (20 s) | no | symptoms |
| 📢 [Broadcast storm](#broadcast_storm) | advanced | 40 s | 🌲 Enable Spanning Tree | — | no | symptoms |
| 🏝️ [VLAN misconfiguration](#vlan_misconfiguration) | advanced | 55 s | 🏝️ Move ports to the right VLAN | — | no | symptoms |

## Service crashed

💥 `crash` · systems · **60 s** to loss · hits: any node · ▶ [live on Lucky Monkey Guardian](https://go.fridge.run/luckymonkey#node=monkey-statue&incident=crash&zoom=1.2)

**Symptom:** The service process exited. The machine still answers ping, but its port is closed.

**How it starts:** *A nightly job on the node ran out of file handles and the service exited.* · *the node hit an unhandled error after the morning traffic spike.* · *A config reload on the node went wrong and the process died.*

**What its neighbours suffer:** every call it makes to the origin is refused.

| Response | Effect | Why |
|---|---|---|
| 🔄 Restart the service<br>`systemctl restart {node}` | ✅ fix | The process is back; the logs show a one-off crash. |
| 🧹 Clear old snapshots | ✖ no effect | A real tool for a different problem. |
| 🔑 Renew the TLS certificate | ✖ no effect | A real tool for a different problem. |

**Concept, Processes and ports:** A service is a running process listening on a port. The machine can be up (it answers ping) while the service is dead.

## DNS resolution failing

📛 `dns` · network · **55 s** to loss · hits: `dns` · ▶ [live on DNS Shrine](https://go.fridge.run/luckymonkey#node=dns-shrine&incident=dns&zoom=1.2)

**Symptom:** Clients can't resolve hostnames. Connecting by IP address still works, but nobody remembers the numbers.

**How it starts:** *A typo slipped into the node's resolver config during a 'quick' change.* · *the node's upstream resolver was swapped for one that doesn't answer.*

**What its neighbours suffer:** it can't look up the origin's names, so its requests fail.

| Response | Effect | Why |
|---|---|---|
| 📛 Restore the DNS resolver config<br>`cp /etc/resolv.conf.known-good /etc/resolv.conf && dig +short api.jungle` | ✅ fix | Resolver configuration restored; names resolve again. |
| 🔄 Restart the service<br>`systemctl restart {node}` | 🩹 mitigate, 25 s | The restart reloaded the cache, but the bad config returns as records expire. |
| 🔌 Replace the cable | ✖ no effect | A real tool for a different problem. |
| 🏷️ Expand the DHCP pool | ✖ no effect | A real tool for a different problem. |

**Concept, DNS:** The Domain Name System turns names into IP addresses. If it fails, connections by IP still work, but names don't.

## DHCP pool exhausted

🏷️ `dhcp` · network · **60 s** to loss · hits: `river`, `lake`, `switch`, `lizard`, `terminal` · ▶ [live on Logical Lagoon API](https://go.fridge.run/luckymonkey#node=logical-lagoon&incident=dhcp&zoom=1.2)

**Symptom:** No addresses left in the pool. New devices get no lease and fall back to 169.254.x.x, which can't reach anything.

**How it starts:** *A batch of new sensors joined the node and took every free address.* · *Someone shrank the node's DHCP range while tidying up.*

**What its neighbours suffer:** new devices that reach it through the origin get no address.

| Response | Effect | Why |
|---|---|---|
| 🏷️ Expand the DHCP pool<br>`dhcpctl pool {node} --range /23` | ✅ fix | The pool is widened from /26 to /24; new devices get leases. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | Restarting doesn't create addresses: the pool is still full. |
| 📛 Restore the DNS resolver config | ✖ no effect | A real tool for a different problem. |

**Concept, DHCP:** DHCP hands out IP addresses from a pool, as time-limited leases. With an empty pool, new devices get no address.

## Damaged cable

🐊 `cable` · systems · **60 s** to loss · hits: `switch`, `router`, `balancer`, `cdn`, `server` · ▶ [live on Crashed Sector 404](https://go.fridge.run/luckymonkey#node=crashed-sector&incident=cable&zoom=1.2)

**Symptom:** 20% packet loss and CRC errors on one port. A crocodile chewed the patch cable.

**How it starts:** *A crocodile chewed through the node's uplink cable.* · *A falling branch crushed the cable into the node.*

**What its neighbours suffer:** its traffic through the origin keeps getting dropped.

| Response | Effect | Why |
|---|---|---|
| 🔌 Replace the cable<br>`ethtool {node}:eth0 && swap-cable --port eth0` | ✅ fix | New cable fitted; the CRC errors are gone. It was the physical layer. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | Software restarts don't fix a damaged cable (layer 1). |
| 🧭 Correct the routing table | ✖ no effect | A real tool for a different problem. |

**Concept, Physical layer:** Layer 1: cables, ports and signals. CRC errors and packet loss on one port often mean a bad cable, which no software fixes.

## Missing route

🧭 `route` · network · **50 s** to loss · hits: `router`, `firewall` · ▶ [live on メインルーター Router](https://go.fridge.run/luckymonkey#node=main-router&incident=route&zoom=1.2)

**Symptom:** The router lost its route to the southern islands; packets come back as ICMP 'destination unreachable'.

**How it starts:** *A route to the node was deleted during maintenance.* · *the node learned a bad route from a neighbour.*

**What its neighbours suffer:** packets it sends via the origin never arrive.

| Response | Effect | Why |
|---|---|---|
| 🧭 Correct the routing table<br>`ip route replace default via 10.0.0.1 dev eth0` | ✅ fix | The static route is restored; traffic flows again. |
| 🔀 Fail over to standby<br>`haproxyctl failover {node} --to standby` | 🩹 mitigate, 30 s | The standby router works for now, but its routing table is stale. |
| 📛 Restore the DNS resolver config | ✖ no effect | A real tool for a different problem. |
| 🛡️ Rate-limit traffic | ✖ no effect | A real tool for a different problem. |

**Concept, Routing:** Routers forward packets using a routing table. Without a route, the packet is dropped and ICMP reports 'destination unreachable'.

## DDoS flood

🦬 `ddos` · security · **40 s** to loss · hits: `cdn`, `balancer`, `auth`, `firewall`, `banana-cache`, `river`, `lake` · ▶ [live on Logical Lagoon API](https://go.fridge.run/luckymonkey#node=logical-lagoon&incident=ddos&zoom=1.2)

**Symptom:** Requests ×40 from thousands of sources. A bison stampede is hammering one endpoint.

**How it starts:** *A botnet of a million bison found the node on the open internet.* · *Someone posted the node's address on a stampede forum.*

**What its neighbours suffer:** the flood through the origin is crushing it too.

| Response | Effect | Why |
|---|---|---|
| 🚫 Block attackers with an ACL<br>`iptables -A INPUT -m set --match-set stampede src -j DROP` | ✅ fix | Firewall ACL drops the attacking sources; real users get through. |
| 🛡️ Rate-limit traffic<br>`tc qdisc add dev eth0 root tbf rate 200mbit burst 32k` | 🩹 mitigate, 40 s | Rate limiting buys time but also turns away some real users. The attack continues. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | Restarting doesn't stop traffic that is still arriving. |
| ⏪ Roll back the deploy | ✖ no effect | A real tool for a different problem. |

**Concept, DDoS:** A distributed denial of service floods a target from many sources. Rate limits buy time; filtering the sources (ACLs) stops it.

## Worm infection

🐛 `worm` · security · **50 s** to loss · hits: `server`, `terminal`, `ai-core`, `temple`, `vault`, `queue` · ▶ [live on Node Temple Alpha](https://go.fridge.run/luckymonkey#node=node-temple-1&incident=worm&zoom=1.2)

**Symptom:** An unknown process copies itself over every connection it can reach.

**How it starts:** *Someone on the node opened a banana-themed email attachment.* · *An unpatched service on the node was found by a scanning worm.*

**Warning stage** 🧬 (75 s): A worm signature is circulating and this node is unpatched. Prevent with 🩹 Install the patch. ▶ [live](https://go.fridge.run/luckymonkey#node=node-temple-1&forecast=worm&zoom=1.2)

| Response | Effect | Why |
|---|---|---|
| 🩹 Install the patch<br>`apt-get install --only-upgrade jungle-runtime` | ✅ fix | Patched and cleaned; the hole it used is closed. |
| 🚧 Quarantine on an isolated VLAN<br>`switchctl move {node} --vlan 666-quarantine` | 🩹 mitigate, 45 s | Isolated on its own VLAN: it can't spread, but it's still infected. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | The worm survives reboots: it's in the startup scripts. |

**Contagious:** as its timer runs down it infects healthy neighbours with the same fault, each needing its own fix.

**Concept, Patching and segmentation:** Worms exploit unpatched holes and spread across the network. Isolating a node (a separate VLAN) stops the spread; patching removes the hole.

## Expired TLS certificate

🔐 `cert` · security · **50 s** to loss · hits: `auth`, `cdn`, `queue`, `firewall`, `balancer` · ▶ [live on 北ファイアウォール Firewall](https://go.fridge.run/luckymonkey#node=firewall-north&incident=cert&zoom=1.2)

**Symptom:** Clients reject the TLS handshake: the certificate's validity date has passed.

**How it starts:** *the node's certificate auto-renewal has been failing quietly for weeks.* · *the node's certificate was issued by hand a year ago, and nobody set a reminder.*

**What its neighbours suffer:** TLS handshakes with the origin fail.

**Warning stage** 🔐 (80 s): The TLS certificate expires soon. Renew it now and nobody notices. Prevent with 🔑 Renew the TLS certificate. ▶ [live](https://go.fridge.run/luckymonkey#node=firewall-north&forecast=cert&zoom=1.2)

| Response | Effect | Why |
|---|---|---|
| 🔑 Renew the TLS certificate<br>`certbot renew --cert-name {node} && systemctl reload {node}` | ✅ fix | New certificate installed; handshakes succeed. |
| 🔓 Disable TLS<br>`sed -i 's/tls on/tls off/' /etc/{node}.conf` | ✖ no effect | Refused: turning off TLS would send passwords in clear text. Renew the certificate instead. |
| 🔄 Restart the service | ✖ no effect | A real tool for a different problem. |

**Concept, TLS certificates:** TLS encrypts connections and proves identity with a certificate that expires. Renew it before it does; never just turn encryption off.

## Memory leak

📈 `leak` · systems · **55 s** to loss · hits: `server`, `ai-core`, `terminal`, `queue`, `monkey` · ▶ [live on Lucky Monkey Guardian](https://go.fridge.run/luckymonkey#node=monkey-statue&incident=leak&zoom=1.2)

**Symptom:** RAM use grows 2 GB a minute and the system is swapping. Something never frees its memory.

**How it starts:** *The new release on the node forgets to free its cache.* · *A slow memory leak in the node has been growing since Monday.*

**What its neighbours suffer:** its calls to the origin time out.

| Response | Effect | Why |
|---|---|---|
| 🩹 Install the patch<br>`apt-get install --only-upgrade jungle-runtime` | ✅ fix | The code that never freed memory is fixed. |
| 🔄 Restart the service<br>`systemctl restart {node}` | 🩹 mitigate, 45 s | Memory is freed for now; the leak will fill RAM again. |
| 🔀 Fail over to standby<br>`haproxyctl failover {node} --to standby` | 🩹 mitigate, 60 s | The standby takes over with clean memory; the same bug leaks there too. |

**Concept, Memory and swap:** A leak keeps claiming RAM until the system swaps to disk and crawls. A restart frees it for a while; only a code fix stops it.

## Disk full

💾 `disk` · systems · **50 s** to loss · hits: `database`, `vault`, `lake`, `banana-cache`, `queue` · ▶ [live on Banana Sector Cache](https://go.fridge.run/luckymonkey#node=banana-sector&incident=disk&zoom=1.2)

**Symptom:** Disk at 100%. Writes fail and logs can't rotate.

**How it starts:** *the node's snapshots haven't been pruned since spring.* · *A debug log on the node was left on, writing gigabytes an hour.*

**What its neighbours suffer:** its writes to the origin are rejected.

**Warning stage** 💾 (75 s): Disk at 85% and climbing: old snapshots are piling up. Prevent with 🧹 Clear old snapshots. ▶ [live](https://go.fridge.run/luckymonkey#node=banana-sector&forecast=disk&zoom=1.2)

| Response | Effect | Why |
|---|---|---|
| 🧹 Clear old snapshots<br>`snapctl prune {node} --older-than 30d` | ✅ fix | Old snapshots removed; writes work again. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | Rebooting doesn't create disk space. |

**Concept, Storage capacity:** A full disk makes every write fail. Rebooting doesn't free space: delete or move data, or add capacity.

## Thermal throttling

🌡️ `heat` · systems · **55 s** to loss · hits: `server`, `ai-core`, `volcano`, `vault` · ▶ [live on Crashed Sector 404](https://go.fridge.run/luckymonkey#node=crashed-sector&incident=heat&zoom=1.2)

**Symptom:** CPU at 98 °C and throttling its clock speed: the vents are clogged with leaves.

**How it starts:** *The afternoon sun is on the node's rack and a fan is clogged with leaves.* · *the node's cooling vent is blocked by a fallen palm frond.*

**What its neighbours suffer:** the origin answers so slowly that it times out.

| Response | Effect | Why |
|---|---|---|
| 🍃 Clear the vents<br>`ipmitool sensor && blow-dust --rack {node}` | ✅ fix | Vents cleared; the temperature is back to 55 °C. |
| 🔄 Restart the service<br>`systemctl restart {node}` | 🩹 mitigate, 20 s | It cooled while rebooting; it will heat up again. |

**Concept, Thermal throttling:** Hot CPUs slow themselves down to survive. Cooling (airflow, clean vents) is the fix.

## Power supply failed

⚡ `power` · systems · **35 s** to loss · hits: any node · ▶ [live on Lucky Monkey Guardian](https://go.fridge.run/luckymonkey#node=monkey-statue&incident=power&zoom=1.2)

**Symptom:** Lightning hit the rack and the primary power supply is dead. The redundant one is idle.

**How it starts:** *A storm cloud's lightning struck the node's power supply.*

**What its neighbours suffer:** it lost its link through the origin.

| Response | Effect | Why |
|---|---|---|
| 🔀 Fail over to standby<br>`haproxyctl failover {node} --to standby` | ✅ fix | Switched to the redundant power supply. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | Nothing to restart: there's no power. |

**Concept, Redundancy and failover:** A spare component (a second power supply, a standby server) takes over when the primary fails, if one was there to begin with.

## Bad deploy

📦 `deploy` · operations · **55 s** to loss · hits: `server`, `auth`, `cdn`, `queue`, `banana-cache`, `ai-core` · ▶ [live on Banana Sector Cache](https://go.fridge.run/luckymonkey#node=banana-sector&incident=deploy&zoom=1.2)

**Symptom:** v2.4 went out and the error rate is 30%. There was no schema change: v2.3 is still compatible.

**How it starts:** *v2.4 went out to the node without a canary.* · *A Friday-afternoon deploy reached the node.*

**What its neighbours suffer:** one in three requests to the origin returns an error.

**Warning stage** 📅 (80 s): A deploy of v2.4 is scheduled. Canary it first: a bad build then reaches only a few users. Prevent with 🐤 Canary the deploy. ▶ [live](https://go.fridge.run/luckymonkey#node=banana-sector&forecast=deploy&zoom=1.2)

| Response | Effect | Why |
|---|---|---|
| ⏪ Roll back the deploy<br>`deployctl rollback {node} --to v2.3` | ✅ fix | Back on v2.3, which is compatible: the errors stop. |
| ⏩ Fall forward: ship a hotfix<br>`deployctl ship {node} v2.4.1-hotfix` | ✖ no effect | No hotfix is ready yet. v2.3 is compatible, so rolling back is faster and safe. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | Restarting runs the same broken v2.4. |

**Concept, Rollback:** Undo a bad change by going back to the previous version, as long as that version still works with today's data.

## Bad deploy with migration

📦 `migration` · operations · **55 s** to loss · hits: any node · ▶ [live on Lucky Monkey Guardian](https://go.fridge.run/luckymonkey#node=monkey-statue&incident=migration&zoom=1.2)

**Symptom:** v2.4 migrated the database schema and the error rate is 30%. v2.3 can't read the new tables.

**How it starts:** *v2.4 went out to the node with a database migration.*

**What its neighbours suffer:** one in three requests to the origin returns an error.

| Response | Effect | Why |
|---|---|---|
| ⏩ Fall forward: ship a hotfix<br>`deployctl ship {node} v2.4.1-hotfix` | ✅ fix | Hotfix v2.4.1 shipped; it works with the new schema. |
| ⏪ Roll back the deploy<br>`deployctl rollback {node} --to v2.3` | ✖ no effect | v2.3 can't read the migrated schema: rolling back would break it further. |
| 🔄 Restart the service | ✖ no effect | A real tool for a different problem. |

**Concept, Schema migrations:** A deploy that changes the database schema can't simply be rolled back: the old version can't read the new tables. Fix forward.

## Replica disparity

⚖️ `disparity` · operations · **60 s** to loss · hits: `database`, `vault`, `lake`, `banana-cache` · ▶ [live on Banana Sector Cache](https://go.fridge.run/luckymonkey#node=banana-sector&incident=disparity&zoom=1.2)

**Symptom:** The replicas' checksums disagree: two copies report two different banana counts.

**How it starts:** *A replica of the node fell behind during a network blip and nobody noticed.* · *Writes to the node went to the wrong replica during a failover.*

**What its neighbours suffer:** it reads stale or conflicting data from the origin.

| Response | Effect | Why |
|---|---|---|
| 🔁 Resync replicas from primary<br>`replctl resync {node}-replica --from primary` | ✅ fix | Replicas re-copied from the primary; the checksums match. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | Restarting doesn't reconcile the data. |

**Concept, Replication:** Replicas are copies of a database. When they disagree, re-copy them from the authoritative primary.

## Routing loop

🔁 `loop` · advanced · **45 s** to loss · hits: `router`, `switch` · ▶ [live on メインルーター Router](https://go.fridge.run/luckymonkey#node=main-router&incident=loop&zoom=1.2)

**Symptom:** Packets circle between two routers until their IP TTL (a hop count) reaches 0 and they're dropped.

**How it starts:** *Two routers near the node each think the other has the best path.* · *A static route on the node points back at itself.*

**What its neighbours suffer:** packets through the origin go round in circles.

| Response | Effect | Why |
|---|---|---|
| 🧭 Correct the routing table<br>`ip route replace default via 10.0.0.1 dev eth0` | ✅ fix | The route that pointed back is fixed; packets reach their destination. |
| 🔄 Restart the service<br>`systemctl restart {node}` | 🩹 mitigate, 20 s | Routes were relearned, but the bad static route returns. |

**Concept, IP TTL:** Each IP packet carries a TTL hop count; every router subtracts 1, and at 0 the packet is dropped. That stops a routing loop from circling forever.

## Broadcast storm

📢 `broadcast` · advanced · **40 s** to loss · hits: `switch`, `balancer` · ▶ [live on Junction Switch](https://go.fridge.run/luckymonkey#node=switch-01&incident=broadcast&zoom=1.2)

**Symptom:** A looped cable floods the switch with broadcasts; Spanning Tree was turned off on this switch.

**How it starts:** *Someone plugged both ends of a cable into the node.* · *A redundant link was added to the node without Spanning Tree.*

**What its neighbours suffer:** the broadcast flood from the origin swamps it.

| Response | Effect | Why |
|---|---|---|
| 🌲 Enable Spanning Tree<br>`switchctl stp enable --all-ports` | ✅ fix | STP blocked the redundant port; the storm stops. |
| 🔌 Replace the cable<br>`ethtool {node}:eth0 && swap-cable --port eth0` | ✖ no effect | The cables are fine; it's the loop they form. |

**Concept, Spanning Tree:** STP finds loops between switches and blocks a redundant port, so broadcasts can't circle and multiply into a storm.

## VLAN misconfiguration

🏝️ `vlan` · advanced · **55 s** to loss · hits: `switch`, `firewall` · ▶ [live on 北ファイアウォール Firewall](https://go.fridge.run/luckymonkey#node=firewall-north&incident=vlan&zoom=1.2)

**Symptom:** Ports were put on the wrong VLAN: the tree frogs can't reach their default gateway.

**How it starts:** *A port change on the node put servers in the wrong VLAN.* · *the node was re-cabled and the VLAN tags weren't updated.*

**What its neighbours suffer:** it can't reach anything behind the origin.

| Response | Effect | Why |
|---|---|---|
| 🏝️ Move ports to the right VLAN<br>`switchctl set-vlan {node} ports 1-12 --vlan 20` | ✅ fix | Ports are back on VLAN 20; the gateway answers. |
| 🔄 Restart the service<br>`systemctl restart {node}` | ✖ no effect | The port configuration survives a restart. |

**Concept, VLANs:** VLANs split one switch into separate networks. A port on the wrong VLAN can't reach its own subnet's default gateway.

## Traps

- 🔓 Disable TLS: the game refuses it and explains why turning off encryption is never a fix.

## Lightning

Storm clouds flash and strike a node within range: a **Power supply failed** fault (above). It goes through the same director budget as every other fault, so a storm never piles on top of a bad moment.

Next: [Nodes and lore](nodes.md) · Back to the [documentation](index.md)
