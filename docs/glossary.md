---
title: Lucky Monkey - glossary
---

# Glossary

Terms used across these pages, each with where to read more.

## Game terms

| Term | Meaning | More |
|---|---|---|
| **Shift** | One game: 09:00 to 17:00, 8 real minutes | [How to play](gameplay.md#the_shift) |
| **Network health** | Right now: online 1, degraded 0.5, unreachable 0.3, down 0, weighted by criticality | [The shift](gameplay.md#the_shift) |
| **Shift SLA** | The average network health over the shift; target 95%. In real life, a [service level agreement](https://en.wikipedia.org/wiki/Service-level_agreement) | [The shift](gameplay.md#the_shift) |
| **Time to Loss** ⏳ | The game's own countdown to the next consequence. **Not** IP TTL (a hop count) and **not** DNS TTL (how long a record may be cached) | [Trouble](gameplay.md#trouble) |
| **Warning** | Trouble you can see coming and prevent; nothing broken yet | [Trouble](gameplay.md#trouble) |
| **Incident** 🚨 | A node degraded by a fault, counting down to down | [Trouble](gameplay.md#trouble) |
| **Root cause / impact** | The fault itself vs what it does to others (unreachable nodes have no fault of their own) | [Simulation](simulation.md#healthroot_causes_vs_impacts) |
| **Fix / mitigate** | Remove the cause vs buy time | [Responding](gameplay.md#responding) |
| **Diagnose** 🔍 | Inspect first: shows what each option would do | [Responding](gameplay.md#responding) |
| **Director** | Decides when new trouble arrives, and holds back when you're swamped | [The director](gameplay.md#the_directorwhy_it_never_piles_on_unfairly) |
| **Crew** | Mango 🐵, Pip 🦜, Moss 🦥, Fern 🐸 | [Your crew](gameplay.md#your_crew) |
| **Tour / Practice** | Learn the network with nothing breaking / one problem at a time | [Learn first](gameplay.md#learn_the_network_before_you_play) |
| **Edge pins** | Off-screen nodes shown as their icon on the screen edge | [All the keys](gameplay.md#all_the_keys) |
| **Data frames** | The little boxes streaming along links and the river | [The world](world.md#water_effects) |
| **Deep link** | A URL `#hash` that opens the game at a node, with trouble staged | [Deep links](interface.md#deep_links) |

## Operations terms (the real-world side)

| Term | Meaning | More |
|---|---|---|
| **On call** | Being the person who responds when systems break | [SRE: Incidents and operability](https://go.fridge.run/certs/learn/sre?n=5) |
| **DDoS** | A flood of requests from many sources, meant to overwhelm a service | [Wikipedia](https://en.wikipedia.org/wiki/Denial-of-service_attack) · [in the game](failures.md#ddos_flood) |
| **Rate limiting** | Capping how many requests a client may make | [Wikipedia](https://en.wikipedia.org/wiki/Rate_limiting) · [Matt's guide: usage plans](https://go.fridge.run/certs/learn/matt?n=4) |
| **Worm** | Malware that copies itself across a network on its own | [Wikipedia](https://en.wikipedia.org/wiki/Computer_worm) · [in the game](failures.md#worm_infection) |
| **Quarantine** | Isolating a host so a problem cannot spread | [Security+: Incident response](https://go.fridge.run/certs/learn/secplus?n=4) |
| **Replica / split-brain** | Copies of data; split-brain is when copies diverge | [Wikipedia](https://en.wikipedia.org/wiki/Split-brain_(computing)) · [in the game](failures.md#replica_disparity) |
| **Rollback** | Returning to the previous version | [in the game](failures.md#bad_deploy) |
| **Fall forward / roll forward** | Fixing a bad release with a new one, because going back is impossible (e.g. after a schema migration) | [in the game](failures.md#bad_deploy) |
| **Schema migration** | A change to the shape of stored data; often one-way | [Data & Databases course](https://go.fridge.run/certs/t/data) |
| **Canary release** | Sending a new version to a small slice of traffic first | [Martin Fowler](https://martinfowler.com/bliki/CanaryRelease.html) |
| **Failover** | Switching to standby capacity when the primary fails | [Wikipedia](https://en.wikipedia.org/wiki/Failover) |
| **TLS certificate** | Proof of a server's identity; expires | [Wikipedia](https://en.wikipedia.org/wiki/Transport_Layer_Security) · [Cryptography course](https://go.fridge.run/certs/t/crypto) |
| **Memory leak** | A program holding memory it no longer needs | [Wikipedia](https://en.wikipedia.org/wiki/Memory_leak) |
| **Cascading failure** | One failure overloading its neighbours into failing too | [Wikipedia](https://en.wikipedia.org/wiki/Cascading_failure) |
| **DNS, load balancer, CDN, message queue** | The services behind the new places on the map | [Network+: services every network runs](https://go.fridge.run/certs/learn/netplus?n=4) · [Wikipedia: DNS](https://en.wikipedia.org/wiki/Domain_Name_System) · [load balancing](https://en.wikipedia.org/wiki/Load_balancing_(computing)) · [CDN](https://en.wikipedia.org/wiki/Content_delivery_network) · [message queue](https://en.wikipedia.org/wiki/Message_queue) |

## Engine and code terms

| Term | Meaning | More |
|---|---|---|
| **Bevy** | The Rust game engine the game runs on (0.20) | [bevyengine.org](https://bevyengine.org) · [Engine](engine.md) |
| **ECS** | Entity-Component-System: things are IDs with data attached, behaviour lives in systems | [ECS in five minutes](engine.md#ecs_in_five_minutes) |
| **Entity / component / system / resource / message** | The parts of ECS | [table](engine.md#ecs_in_five_minutes) |
| **Plugin** | A module's registration function | [How the app is assembled](engine.md#how_the_app_is_assembled) |
| **Schedule** | When systems run (Startup, Update…) | [When each system runs](engine.md#when_each_system_runs) |
| **WebAssembly (wasm)** | Compact binary code browsers run fast | [webassembly.org](https://webassembly.org) |
| **wasm-bindgen** | Generates the JavaScript glue that loads the wasm | [GitHub](https://github.com/wasm-bindgen/wasm-bindgen) · [Build](build-deploy.md#the_toolchain_on_fridge_user_fridge_incargo) |
| **WebGL2** | The browser's GPU drawing API | [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebGL2RenderingContext) |
| **Isometric projection** | Drawing a grid at an angle so it looks 3D without perspective | [Wikipedia](https://en.wikipedia.org/wiki/Isometric_video_game_graphics) · [the maths](world.md#the_isometric_maths) |
| **Texture atlas** | Many small pictures packed into one image | [Wikipedia](https://en.wikipedia.org/wiki/Texture_atlas) · [tiles](world.md#tiles_and_the_alignment_bug_that_started_it_all) |
| **Mesh** | Triangles with positions, texture coordinates and colours | [terrain mesh](world.md#the_terrain_mesh) |
| **z-order** | Draw order in 2D: higher z on top | [depth](engine.md#renderingdepth_in_a_2d_isometric_world) |
| **Dijkstra's algorithm** | Shortest path on a weighted graph | [Wikipedia](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm) · [`fn route`](source/ops.md#fn_route) · [Algorithms course](https://go.fridge.run/certs/t/algo) |
| **xorshift** | A tiny fast random-number generator | [Wikipedia](https://en.wikipedia.org/wiki/Xorshift) · [`struct Rng`](source/main.md#struct_rng) |
| **Font subset** | A font cut down to only the characters used | [`subset-fonts.py`](source/subset-fonts.md) |
| **fridge-next** | The shared Next.js container image on fridge | [Runtimes](:dev:runtimes) · [fridge-next-runtime](:git:next-runtime:start) |

Back to the [documentation](index.md)
