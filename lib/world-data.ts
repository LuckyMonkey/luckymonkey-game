import { mulberry32 } from "@/lib/iso"

const WORLD_SEED = 8675309
const rand = mulberry32(WORLD_SEED)

export interface Position {
  x: number
  y: number
  height?: number
  isLagoon?: boolean
  isWaterfall?: boolean
}

export interface TileData {
  x: number
  y: number
  type: number
  height: number
  isWaterfall?: boolean
  isLagoon?: boolean
}

export interface ChunkData {
  id: string
  center: Position
  tiles: TileData[]
  priority: number // Higher priority chunks load first
}

export interface NodeData {
  id: string
  name: string
  type: string
  ipAddress: string
  emoji: string
  description: string
  position: Position
  connections?: string[]
  status?: "online" | "offline" | "unstable" | "erupting"
  securityLevel?: number
  assets?: string[]
}

export interface WorldData {
  meta: {
    seed: number
    biome: string
    lastUpdated: string
  }
  chunks: ChunkData[]
  nodes: NodeData[]
  rivers: Position[]
  decorations: {
    type: string
    position: Position
  }[]
}

// Centrelines the data frames float down (filled by generateRiverSystem)
const MAIN_RIVER_LINE: Position[] = []

// Where the Digital River ends
const DATA_LAKE = { x: 11, y: 38 }

// Digital River, continued: leaves the small lagoon (7,12), runs down the west edge, bends into the Data Lake
const DIGITAL_RIVER_BRANCH: Position[] = [
  { x: 7, y: 15 }, { x: 6, y: 16 }, { x: 5, y: 17 }, { x: 4, y: 18 }, { x: 3, y: 19 }, { x: 2, y: 20 },
  ...Array.from({ length: 15 }, (_, i) => ({ x: 1, y: 21 + i })),
  { x: 2, y: 36 }, { x: 3, y: 37 }, { x: 4, y: 38 }, { x: 5, y: 38 }, { x: 6, y: 38 }, { x: 7, y: 38 }, { x: 8, y: 38 },
].map((p) => ({ ...p, height: 0 }))

// Update river generation to ensure it flows from top to bottom with proper features
function generateRiverSystem(): Position[] {
  const riverPositions: Position[] = []

  // Extended main river trunk - from top to bottom with more bends
  const riverPath = [
    // Start at the very top (waterfall section)
    { x: 20, y: -5, height: 5, isWaterfall: true },
    { x: 20, y: -4, height: 5, isWaterfall: true },
    { x: 20, y: -3, height: 4, isWaterfall: true },
    { x: 20, y: -2, height: 4, isWaterfall: true },
    { x: 20, y: -1, height: 3, isWaterfall: true },
    { x: 20, y: 0, height: 3, isWaterfall: true },
    { x: 20, y: 1, height: 2, isWaterfall: true },
    { x: 20, y: 2, height: 2, isWaterfall: true },
    { x: 20, y: 3, height: 1, isWaterfall: true },
    { x: 20, y: 4, height: 1, isWaterfall: true },
    { x: 20, y: 5, height: 0 },
    { x: 20, y: 6, height: 0 },

    // Continue straight down
    { x: 20, y: 7, height: 0 },
    { x: 20, y: 8, height: 0 },
    { x: 20, y: 9, height: 0 },

    // First bend to the right
    { x: 21, y: 10, height: 0 },
    { x: 22, y: 11, height: 0 },
    { x: 23, y: 12, height: 0 },
    { x: 24, y: 13, height: 0 },
    { x: 25, y: 14, height: 0 },

    // Continue down
    { x: 25, y: 15, height: 0 },
    { x: 25, y: 16, height: 0 },
    { x: 25, y: 17, height: 0 },

    // Second bend to the left
    { x: 24, y: 18, height: 0 },
    { x: 23, y: 19, height: 0 },
    { x: 22, y: 20, height: 0 },
    { x: 21, y: 21, height: 0 },
    { x: 20, y: 22, height: 0 },

    // Continue down
    { x: 20, y: 23, height: 0 },
    { x: 20, y: 24, height: 0 },
    { x: 20, y: 25, height: 0 },
    { x: 20, y: 26, height: 0 },
    { x: 20, y: 27, height: 0 },
    { x: 20, y: 28, height: 0 },
    { x: 20, y: 29, height: 0 },
    { x: 20, y: 30, height: 0 },

    // Third bend to the right
    { x: 21, y: 31, height: 0 },
    { x: 22, y: 32, height: 0 },
    { x: 23, y: 33, height: 0 },
    { x: 24, y: 34, height: 0 },
    { x: 25, y: 35, height: 0 },
    { x: 26, y: 36, height: 0 },
    { x: 27, y: 37, height: 0 },
    { x: 28, y: 38, height: 0 },
    
    // Continue down to delta
    { x: 28, y: 39, height: 0 },
    { x: 28, y: 40, height: 0 },
    { x: 28, y: 41, height: 0 },
    { x: 28, y: 42, height: 0 },
    { x: 28, y: 43, height: 0 },
    { x: 28, y: 44, height: 0 },
    { x: 28, y: 45, height: 0 },
  ]

  MAIN_RIVER_LINE.splice(0, MAIN_RIVER_LINE.length, ...riverPath)

  // Add the main river path
  riverPath.forEach((point) => {
    // Add the center point
    riverPositions.push(point)

    // Add width to the river (1 tile on each side)
    riverPositions.push({ ...point, x: point.x - 1 })
    riverPositions.push({ ...point, x: point.x + 1 })
  })

  // Create a lagoon near position (17, 7)
  const lagoonCenterX = 17
  const lagoonCenterY = 7
  const lagoonRadius = 4

  for (let dx = -lagoonRadius; dx <= lagoonRadius; dx++) {
    for (let dy = -lagoonRadius; dy <= lagoonRadius; dy++) {
      const distance = Math.sqrt(dx * dx + dy * dy)
      if (distance <= lagoonRadius) {
        riverPositions.push({
          x: lagoonCenterX + dx,
          y: lagoonCenterY + dy,
          height: 0,
          isLagoon: true,
        })
      }
    }
  }

  // Connect lagoon to main river
  const lagoonConnection = [
    { x: 17, y: 7, height: 0 },
    { x: 18, y: 7, height: 0 },
    { x: 19, y: 7, height: 0 },
    { x: 20, y: 7, height: 0 },
  ]

  lagoonConnection.forEach((point) => {
    riverPositions.push(point)
  })

  // Create a smaller lagoon near the Digital River node (position 7, 12)
  const smallLagoonX = 7
  const smallLagoonY = 12
  const smallLagoonRadius = 3

  for (let dx = -smallLagoonRadius; dx <= smallLagoonRadius; dx++) {
    for (let dy = -smallLagoonRadius; dy <= smallLagoonRadius; dy++) {
      const distance = Math.sqrt(dx * dx + dy * dy)
      if (distance <= smallLagoonRadius) {
        riverPositions.push({
          x: smallLagoonX + dx,
          y: smallLagoonY + dy,
          height: 0,
          isLagoon: true,
        })
      }
    }
  }

  // Connect the small lagoon to the main river
  const smallLagoonConnection = [
    { x: 7, y: 12, height: 0 },
    { x: 8, y: 12, height: 0 },
    { x: 9, y: 12, height: 0 },
    { x: 10, y: 12, height: 0 },
    { x: 11, y: 12, height: 0 },
    { x: 12, y: 12, height: 0 },
    { x: 13, y: 12, height: 0 },
    { x: 14, y: 12, height: 0 },
    { x: 15, y: 12, height: 0 },
    { x: 16, y: 12, height: 0 },
    { x: 17, y: 12, height: 0 },
    { x: 18, y: 12, height: 0 },
    { x: 19, y: 12, height: 0 },
    { x: 20, y: 12, height: 0 },
    { x: 21, y: 12, height: 0 },
    { x: 22, y: 12, height: 0 },
    { x: 23, y: 12, height: 0 },
  ]

  smallLagoonConnection.forEach((point) => {
    riverPositions.push(point)
  })

  // The Digital River continues past its lagoon: down the west edge into the Data Lake
  DIGITAL_RIVER_BRANCH.forEach((point) => {
    riverPositions.push(point, { ...point, x: point.x - 1 }, { ...point, x: point.x + 1 })
  })
  for (let dx = -3; dx <= 3; dx++) {
    for (let dy = -3; dy <= 3; dy++) {
      if (Math.hypot(dx, dy) <= 3.2) {
        riverPositions.push({ x: DATA_LAKE.x + dx, y: DATA_LAKE.y + dy, height: 0, isLagoon: true })
      }
    }
  }

  // Remove duplicates
  const uniquePositions = new Map<string, Position>()
  riverPositions.forEach((pos) => {
    const key = `${pos.x},${pos.y}`
    uniquePositions.set(key, pos)
  })

  return Array.from(uniquePositions.values())
}

// Update decoration generation to be biome-aware
function generateLushDecorations(): { type: string; position: Position }[] {
  const decorations: { type: string; position: Position }[] = []

  // Define biome areas
  const circles = [
    { name: "volcano", center: { x: 8, y: 8 }, radius: 6 },
    { name: "lagoon", center: { x: 17, y: 7 }, radius: 5 },
    { name: "small_lagoon", center: { x: 7, y: 12 }, radius: 4 },
    { name: "temple", center: { x: 17, y: 17 }, radius: 5 },
  ]
  const riverPath = generateRiverSystem()
  const riverCells = new Set(riverPath.map((p) => `${p.x},${p.y}`))
  const RIVER_BUFFER = 3

  // Function to determine biome at a position
  const getBiomeAt = (x: number, y: number): string => {
    for (const biome of circles) {
      if (Math.hypot(x - biome.center.x, y - biome.center.y) <= biome.radius) {
        return biome.name
      }
    }

    // Check river biome
    {
      for (const point of riverPath) {
        const distance = Math.sqrt(Math.pow(x - point.x, 2) + Math.pow(y - point.y, 2))
        if (distance <= RIVER_BUFFER) {
          return "river"
        }
      }
    }

    return "default"
  }

  // Add biome-specific decorations
  for (let x = 0; x < 40; x += 1) {
    for (let y = 0; y < 40; y += 1) {
      const biome = getBiomeAt(x, y)

      // Skip decorations directly on water
      const isOnRiver = riverCells.has(`${x},${y}`)

      if (isOnRiver) continue

      // Add biome-specific decorations with some randomness
      if (rand() < 0.3) {
        // Increased probability for more decorations
        let decorationType = "flower"
        let height = 0

        switch (biome) {
          case "volcano":
            decorationType = rand() < 0.7 ? "crystal" : "rock"
            height = Math.floor(rand() * 3)
            break
          case "lagoon":
          case "small_lagoon":
            decorationType = rand() < 0.6 ? "lotus" : "flower"
            break
          case "temple":
            decorationType = rand() < 0.6 ? "bamboo" : "tree"
            break
          case "river":
            decorationType = rand() < 0.7 ? "flower" : "bush"
            break
          default:
            decorationType = ["tree", "bush", "flower", "bamboo", "mushroom"][Math.floor(rand() * 5)]
        }

        decorations.push({
          type: decorationType,
          position: { x, y, height },
        })
      }
    }
  }

  // Add specific decorations around volcano
  const volcanoCenter = circles[0].center
  for (let i = 0; i < 20; i++) {
    // More decorations
    const angle = (i / 20) * Math.PI * 2
    const distance = 3 + rand() * 2
    const x = Math.round(volcanoCenter.x + Math.cos(angle) * distance)
    const y = Math.round(volcanoCenter.y + Math.sin(angle) * distance)

    decorations.push({
      type: "crystal",
      position: { x, y, height: Math.floor(rand() * 3) + 1 },
    })
  }

  // Add more trees along the river
  for (let i = 0; i < riverPath.length; i += 2) {
    // More frequent decorations
    const point = riverPath[i]

    // Add trees on both sides of the river
    for (let side = -1; side <= 1; side += 2) {
      if (rand() < 0.8) {
        // Higher probability
        decorations.push({
          type: rand() < 0.7 ? "tree" : "bush",
          position: { x: point.x + side * 2, y: point.y, height: 0 },
        })
      }
    }
  }

  return decorations
}

// Define the nodes with updated positions to connect to rivers
const allNodes: NodeData[] = [
  // Original nodes with updated positions
  {
    id: "monkey-statue",
    name: "Lucky Monkey Guardian",
    type: "monkey",
    ipAddress: "192.168.1.1",
    emoji: "🐵",
    description: "The Guardian Construct. A memetic security avatar that protects the network. Approach with caution.",
    position: { x: 17, y: 17, height: 5 }, // Increased height for temple pyramid
    connections: ["logical-lagoon", "banana-sector", "node-temple-1", "volcano-core"],
    status: "online",
    securityLevel: 5,
    assets: ["guardian_protocol.bin", "security_matrix.dat", "memetic_defense_v3.sys", "avatar_config.json"],
  },
  {
    id: "logical-lagoon",
    name: "Logical Lagoon API",
    type: "river",
    ipAddress: "192.168.1.42",
    emoji: "🌊",
    description: "A sacred API pool where memory fragments drift like lotus petals. Home to bytefish and static frogs.",
    position: { x: 17, y: 7, height: 0 },
    connections: ["monkey-statue", "digital-river"],
    status: "online",
    securityLevel: 3,
    assets: ["api_endpoints.json", "memory_fragments.dat", "lotus_protocol.bin", "static_frog.gif"],
  },
  {
    id: "digital-river",
    name: "Digital River Stream",
    type: "river",
    ipAddress: "192.168.1.101",
    emoji: "🌊",
    description: "Channels raw signal through the jungle. Its rapids form when external pings overload the stream.",
    position: { x: 7, y: 12, height: 0 },
    connections: ["logical-lagoon", "crashed-sector"],
    status: "online",
    securityLevel: 2,
    assets: ["stream_flow.dat", "signal_patterns.bin", "ping_monitor.log", "rapids_config.json"],
  },
  {
    id: "banana-sector",
    name: "Banana Sector Cache",
    type: "banana-cache",
    ipAddress: "192.168.1.99",
    emoji: "🍌",
    description: "A playful but volatile subgrid where corrupted metadata manifests in humorous, unstable objects.",
    position: { x: 27, y: 27, height: 0 },
    connections: ["monkey-statue", "node-temple-2"],
    status: "unstable",
    securityLevel: 1,
    assets: ["corrupted_metadata.bin", "unstable_objects.dat", "humor_algorithm.js", "banana_manifest.json"],
  },
  {
    id: "node-temple-1",
    name: "Node Temple Alpha",
    type: "temple",
    ipAddress: "10.0.0.1",
    emoji: "🏯",
    description:
      "Ancient structure that predates the current grid. Maintained by Ghostpackets, semi-aware maintenance protocols.",
    position: { x: 12, y: 27, height: 0 },
    connections: ["monkey-statue", "crashed-sector"],
    status: "online",
    securityLevel: 4,
    assets: ["ancient_code.bin", "ghostpacket_protocol.dat", "maintenance_routines.sys", "temple_history.txt"],
  },
  {
    id: "node-temple-2",
    name: "Node Temple Beta",
    type: "temple",
    ipAddress: "10.0.0.2",
    emoji: "🏯",
    description:
      "Ancient structure with mysterious origins. Contains a Hash of Belief required to access deeper sectors.",
    position: { x: 27, y: 12, height: 0 },
    connections: ["banana-sector", "server-hub"],
    status: "online",
    securityLevel: 4,
    assets: ["hash_of_belief.key", "mysterious_origins.log", "deeper_sector_map.dat", "ancient_symbols.svg"],
  },
  {
    id: "crashed-sector",
    name: "Crashed Sector 404",
    type: "server",
    ipAddress: "192.168.1.404",
    emoji: "💻",
    description: "A barren dead zone filled with 404 skeletons and server husks. Haunted by Ghostpackets.",
    position: { x: 7, y: 27, height: 0 },
    connections: ["node-temple-1", "digital-river"],
    status: "offline",
    securityLevel: 0,
    assets: ["404_error_log.txt", "server_husks.dat", "ghostpacket_traces.bin", "crash_report.json"],
  },
  {
    id: "server-hub",
    name: "Server Hub Cluster",
    type: "server",
    ipAddress: "192.168.1.200",
    emoji: "💻",
    description:
      "A cluster of ancient servers processing unknown data. The terminals occasionally display cryptic messages.",
    position: { x: 37, y: 17, height: 0 },
    connections: ["node-temple-2"],
    status: "online",
    securityLevel: 3,
    assets: ["unknown_data.bin", "cryptic_messages.log", "terminal_output.txt", "processing_algorithms.js"],
  },
  {
    id: "main-router",
    name: "メインルーター Router",
    type: "router",
    ipAddress: "192.168.0.1",
    emoji: "📡",
    description:
      "Central routing node that manages all network traffic. Elevated position provides optimal signal distribution.",
    position: { x: 21, y: 21, height: 4 },
    connections: ["monkey-statue", "firewall-north", "firewall-south"],
    status: "online",
    securityLevel: 5,
    assets: ["routing_table.dat", "network_traffic.log", "signal_distribution.bin", "router_config.json"],
  },
  {
    id: "firewall-north",
    name: "北ファイアウォール Firewall",
    type: "firewall",
    ipAddress: "192.168.0.10",
    emoji: "🔥",
    description:
      "Northern security barrier that filters incoming traffic. Runs on ancient code that nobody fully understands.",
    position: { x: 31, y: 6, height: 3 },
    connections: ["main-router", "database-cluster"],
    status: "online",
    securityLevel: 4,
    assets: ["filter_rules.dat", "ancient_code.bin", "traffic_logs.txt", "security_protocols.json"],
  },
  {
    id: "firewall-south",
    name: "南ファイアウォール Firewall",
    type: "firewall",
    ipAddress: "192.168.0.11",
    emoji: "🔥",
    description:
      "Southern security barrier with additional encryption layers. Frequently targeted by digital predators.",
    position: { x: 6, y: 31, height: 2 },
    connections: ["main-router"],
    status: "online",
    securityLevel: 4,
    assets: ["encryption_keys.dat", "predator_logs.txt", "defense_patterns.bin", "security_breach_history.json"],
  },
  {
    id: "database-cluster",
    name: "データベース Database",
    type: "database",
    ipAddress: "192.168.0.50",
    emoji: "🗄️",
    description:
      "Massive storage facility containing all historical records. Access requires multiple authentication factors.",
    position: { x: 36, y: 36, height: 0 },
    connections: ["firewall-north"],
    status: "online",
    securityLevel: 5,
    assets: ["historical_records.db", "authentication_factors.dat", "storage_index.bin", "access_logs.txt"],
  },
  {
    id: "volcano-core",
    name: "Volcano Core Processor",
    type: "volcano",
    ipAddress: "192.168.1.666",
    emoji: "🌋",
    description:
      "A dangerous but powerful processing center built inside an active digital volcano. Processes data at extreme speeds but occasionally erupts, causing system-wide instabilities.",
    position: { x: 8, y: 8, height: 7 },
    connections: ["monkey-statue"],
    status: "erupting",
    securityLevel: 4,
    assets: ["magma_cooling.sys", "eruption_schedule.dat", "heat_resistant_protocols.bin", "lava_flow_control.json"],
  },

  // New nodes with positions adjusted to be near rivers
  {
    id: "ai-core-01",
    name: "Neural Processor Alpha",
    type: "ai-core",
    ipAddress: "10.0.1.42",
    emoji: "🧠",
    description:
      "Advanced neural network processor that handles complex pattern recognition and decision-making tasks.",
    position: { x: 25, y: 25, height: 2 },
    connections: ["monkey-statue", "vault-01"],
    status: "online",
    securityLevel: 5,
    assets: ["neural_index.bin", "thought_patterns.dat", "simulation_core.sys"],
  },
  {
    id: "vault-01",
    name: "Backup Vault Node",
    type: "vault",
    ipAddress: "10.0.0.3",
    emoji: "🧱",
    description:
      "Secure storage facility for critical system backups and recovery data. Protected by multiple encryption layers.",
    position: { x: 30, y: 15, height: 1 },
    connections: ["ai-core-01", "firewall-01"],
    status: "online",
    securityLevel: 4,
    assets: ["system_snapshot.bin", "recovery_logic.dat", "archive_index.sys"],
  },
  {
    id: "switch-01",
    name: "Junction Switch",
    type: "switch",
    ipAddress: "10.0.0.2",
    emoji: "🔀",
    description: "Network traffic control point that routes data packets between different sectors of the system.",
    position: { x: 20, y: 30, height: 0 },
    connections: ["firewall-01", "banana-cache-01"],
    status: "online",
    securityLevel: 3,
    assets: ["routing_table.bin", "connection_map.dat", "signal_logs.sys"],
  },
  {
    id: "banana-cache-01",
    name: "Humor Cache Alpha",
    type: "banana-cache",
    ipAddress: "10.0.0.4",
    emoji: "🍌",
    description:
      "Storage node for entertainment subroutines and humor algorithms. Often glitches in unpredictable ways.",
    position: { x: 35, y: 20, height: 0 },
    connections: ["switch-01", "terminal-01"],
    status: "unstable",
    securityLevel: 2,
    assets: ["meme_collection.bin", "joke_database.dat", "banana_manifest.sys"],
  },
  {
    id: "terminal-01",
    name: "Legacy Terminal",
    type: "terminal",
    ipAddress: "10.0.0.5",
    emoji: "💻",
    description:
      "Ancient access point to the system core. Uses outdated command syntax but has deep system privileges.",
    position: { x: 15, y: 35, height: 0 },
    connections: ["banana-cache-01"],
    status: "online",
    securityLevel: 3,
    assets: ["command_history.log", "bootword_list.dat", "access_codes.sys"],
  },
  {
    id: "quantum-nexus",
    name: "Quantum Nexus Core",
    type: "server",
    ipAddress: "192.168.q.∞",
    emoji: "⚛️",
    description:
      "Experimental quantum computing node that exists in multiple states simultaneously. Calculations performed here may have different results each time they're observed.",
    position: { x: 10, y: 31, height: 2 }, // moved off Crashed Sector (they overlapped)
    connections: ["node-temple-1"],
    status: "unstable",
    securityLevel: 5,
    assets: [
      "quantum_states.dat",
      "superposition_matrix.bin",
      "observation_effects.log",
      "uncertainty_calculations.js",
    ],
  },
  {
    id: "data-archive",
    name: "Ancient Data Archive",
    type: "database",
    ipAddress: "10.0.1.42",
    emoji: "📚",
    description:
      "Repository of forgotten code and lost algorithms. The oldest data dates back to the pre-digital era, somehow preserved in binary form.",
    position: { x: 38, y: 24, height: 1 },
    connections: ["banana-sector"],
    status: "online",
    securityLevel: 4,
    assets: ["forgotten_code.bin", "lost_algorithms.dat", "pre_digital_artifacts.sys", "binary_preservation.log"],
  },
  {
    id: "lizard-node",
    name: "Reptile Protocol",
    type: "lizard",
    ipAddress: "10.0.2.42",
    emoji: "🦎",
    description:
      "Cold-blooded system monitor that conserves energy while maintaining vigilance. Specializes in detecting anomalies in data patterns.",
    position: { x: 13, y: 12, height: 0 }, // basks on the Digital River channel, clear of the Guardian
    connections: ["monkey-statue", "digital-river"],
    status: "online",
    securityLevel: 3,
    assets: ["cold_blood.bin", "scale_pattern.dat", "sun_basking.sys"],
  },
  // Second wave of places: the jungle's supporting infrastructure
  {
    id: "data-lake",
    name: "Data Lake",
    type: "lake",
    ipAddress: "10.4.0.1",
    emoji: "🏞️",
    description: "Where the Digital River ends. Every frame that ever flowed downstream settles here, replicated three ways and rarely read.",
    position: { x: 11, y: 38, height: 0 },
    connections: ["digital-river", "terminal-01", "firewall-south"],
    status: "online",
    securityLevel: 3,
    assets: ["lake_manifest.json", "replica_a.dat", "replica_b.dat", "replica_c.dat"],
  },
  {
    id: "dns-shrine",
    name: "DNS Shrine",
    type: "dns",
    ipAddress: "10.0.0.53",
    emoji: "⛩️",
    description: "Monks chant names into numbers. When the shrine forgets, nobody in the jungle can find anybody else.",
    position: { x: 12, y: 20, height: 0 },
    connections: ["monkey-statue", "node-temple-1", "lizard-node", "main-router"],
    status: "online",
    securityLevel: 4,
    assets: ["zonefile.txt", "resolver.sys", "ttl_prayers.log"],
  },
  {
    id: "lb-lilypads",
    name: "Lilypad Load Balancer",
    type: "balancer",
    ipAddress: "10.1.0.10",
    emoji: "🪷",
    description: "Frames hop from pad to pad across the river; the pads take turns so no single one sinks.",
    position: { x: 20, y: 25, height: 0 },
    connections: ["main-router", "switch-01", "banana-sector", "cdn-canopy"],
    status: "online",
    securityLevel: 3,
    assets: ["pool_members.json", "health_checks.log"],
  },
  {
    id: "mq-falls",
    name: "Message Queue Falls",
    type: "queue",
    ipAddress: "10.2.0.5",
    emoji: "📨",
    description: "Messages wait in the pool at the top of the waterfall and drop one at a time, in order. Usually.",
    position: { x: 24, y: 3, height: 2 },
    connections: ["logical-lagoon", "firewall-north", "node-temple-2"],
    status: "online",
    securityLevel: 3,
    assets: ["queue_depth.dat", "dead_letters.log", "consumers.json"],
  },
  {
    id: "watchtower",
    name: "Canopy Watchtower",
    type: "monitor",
    ipAddress: "10.9.0.1",
    emoji: "🔭",
    description: "Monitoring. Sees trouble coming: expiring certs, scheduled deploys, filling disks. If it goes dark, so do your forecasts.",
    position: { x: 31, y: 31, height: 1 },
    connections: ["main-router", "banana-sector", "database-cluster"],
    status: "online",
    securityLevel: 4,
    assets: ["dashboards.json", "alert_rules.yml", "pager.log"],
  },
  {
    id: "auth-gate",
    name: "Auth Gate",
    type: "auth",
    ipAddress: "10.3.0.1",
    emoji: "🛂",
    description: "Checks every monkey's papers. Holds the certificates the rest of the jungle trusts.",
    position: { x: 30, y: 21, height: 0 },
    connections: ["vault-01", "ai-core-01", "banana-cache-01", "main-router"],
    status: "online",
    securityLevel: 5,
    assets: ["ca_bundle.pem", "sessions.dat", "policy.json"],
  },
  {
    id: "cdn-canopy",
    name: "CDN Canopy",
    type: "cdn",
    ipAddress: "10.5.0.1",
    emoji: "🌐",
    description: "Copies of popular bananas cached high in the treetops at the river delta, close to every visitor.",
    position: { x: 28, y: 41, height: 0 },
    connections: ["lb-lilypads", "switch-01", "terminal-01"],
    status: "online",
    securityLevel: 2,
    assets: ["edge_cache.bin", "purge_requests.log"],
  },
]

function generateVolcano(): TileData[] {
  const volcanoTiles: TileData[] = []
  const centerX = 8
  const centerY = 8
  const baseRadius = 6

  for (let x = centerX - baseRadius - 2; x <= centerX + baseRadius + 2; x++) {
    for (let y = centerY - baseRadius - 2; y <= centerY + baseRadius + 2; y++) {
      const distance = Math.sqrt(Math.pow(x - centerX, 2) + Math.pow(y - centerY, 2))

      if (distance <= baseRadius) {
        // Inner volcano - higher elevation
        const height = Math.max(0, 7 - Math.floor(distance / 2)) // Adjust height calculation
        volcanoTiles.push({ x, y, type: 6, height }) // Type 6 for volcano
      } else if (distance <= baseRadius + 2) {
        // Outer rim - lower elevation
        const height = Math.max(0, 2 - Math.floor((distance - baseRadius) * 2)) // Adjust height calculation
        volcanoTiles.push({ x, y, type: 5, height }) // Type 5 for scorched earth
      }
    }
  }

  return volcanoTiles
}

// Generate only the necessary chunks for the floating river and nodes
function generateMinimalChunks(): ChunkData[] {
  const chunks: ChunkData[] = []
  const chunkMap = new Map<string, TileData[]>()
  const chunkSize = 5
  const riverPositions = generateRiverSystem()
  const volcanoTiles = generateVolcano()

  // Function to add a tile to a chunk
  const addTileToChunk = (tile: TileData) => {
    const chunkX = Math.floor(tile.x / chunkSize)
    const chunkY = Math.floor(tile.y / chunkSize)
    const chunkKey = `chunk-${chunkX}-${chunkY}`

    if (!chunkMap.has(chunkKey)) {
      chunkMap.set(chunkKey, [])
    }

    chunkMap.get(chunkKey)!.push(tile)
  }

  // Add river tiles
  riverPositions.forEach((pos) => {
    addTileToChunk({
      x: pos.x,
      y: pos.y,
      type: 1, // Water type
      height: pos.height || 0,
      isWaterfall: pos.isWaterfall,
      isLagoon: pos.isLagoon,
    })
  })

  // Add volcano tiles
  volcanoTiles.forEach((tile) => {
    addTileToChunk(tile)
  })

  // Add platform tiles around nodes
  allNodes.forEach((node) => {
    // Determine platform radius based on node type
    let radius = 1 // Smaller radius for most nodes
    let tileType = 4 // Default platform type

    if (node.type === "volcano") {
      radius = 3 // Larger platform for volcano
      tileType = 6 // Volcano type
    } else if (node.type === "monkey") {
      radius = 3 // Larger platform for monkey statue
      tileType = 7 // Golden temple type
    } else if (node.type === "ai-core" || node.type === "temple") {
      radius = 2 // Medium platform for important nodes
    }

    // Generate platform tiles
    for (let dx = -radius; dx <= radius; dx++) {
      for (let dy = -radius; dy <= radius; dy++) {
        const distance = Math.sqrt(dx * dx + dy * dy)
        if (distance <= radius) {
          addTileToChunk({
            x: node.position.x + dx,
            y: node.position.y + dy,
            type: tileType,
            height: node.position.height || 0,
          })
        }
      }
    }

    // Add minimal padding around nodes
    const paddingRadius = 2
    for (let dx = -radius - paddingRadius; dx <= radius + paddingRadius; dx++) {
      for (let dy = -radius - paddingRadius; dy <= radius + paddingRadius; dy++) {
        const distance = Math.sqrt(dx * dx + dy * dy)
        if (distance > radius && distance <= radius + paddingRadius) {
          // Only add padding if not already a platform tile
          addTileToChunk({
            x: node.position.x + dx,
            y: node.position.y + dy,
            type: 0, // Grass
            height: Math.max(0, Math.floor((node.position.height || 0) * 0.5)),
          })
        }
      }
    }
  })

  // Create golden temple pyramid for monkey statue
  const centerX = 17
  const centerY = 17
  const pyramidHeight = 5
  const pyramidSize = 5

  for (let layer = 0; layer < pyramidHeight; layer++) {
    const layerSize = pyramidSize - layer

    for (let dx = -layerSize; dx <= layerSize; dx++) {
      for (let dy = -layerSize; dy <= layerSize; dy++) {
        // Make it more pyramid-like by reducing the corners
        if (Math.abs(dx) + Math.abs(dy) <= layerSize * 1.5) {
          addTileToChunk({
            x: centerX + dx,
            y: centerY + dy,
            type: 7, // Golden temple type
            height: pyramidHeight - layer,
          })
        }
      }
    }
  }

  // Convert chunk map to array and assign priorities
  // Center chunks get higher priority
  const centerChunkX = Math.floor(17 / chunkSize)
  const centerChunkY = Math.floor(17 / chunkSize)

  chunkMap.forEach((tiles, key) => {
    const [_, chunkXStr, chunkYStr] = key.split("-")
    const chunkX = Number.parseInt(chunkXStr)
    const chunkY = Number.parseInt(chunkYStr)

    // Calculate distance from center for priority
    const distanceFromCenter = Math.sqrt(Math.pow(chunkX - centerChunkX, 2) + Math.pow(chunkY - centerChunkY, 2))

    // Higher priority (lower number) for chunks closer to center
    const priority = Math.ceil(distanceFromCenter)

    chunks.push({
      id: key,
      center: {
        x: chunkX * chunkSize + Math.floor(chunkSize / 2),
        y: chunkY * chunkSize + Math.floor(chunkSize / 2),
      },
      tiles,
      priority,
    })
  })

  return chunks
}

// Define floating clouds
// Generate world data
export const riverLines = {
  main: MAIN_RIVER_LINE,
  // From the main river west along the channel to the small lagoon, then down the new branch
  digital: [...Array.from({ length: 17 }, (_, i) => ({ x: 23 - i, y: 12, height: 0 })), ...DIGITAL_RIVER_BRANCH],
}

export const worldData: WorldData = {
  meta: {
    seed: WORLD_SEED,
    biome: "jungle",
    lastUpdated: "2025-05-10T15:00:00Z",
  },
  chunks: generateMinimalChunks(),
  nodes: allNodes,
  rivers: generateRiverSystem(),
  decorations: generateLushDecorations(),
}

// Adjust node positions to bring them closer
export function adjustNodePositions() {
  // Find nodes that are too far away
  const farNodes = worldData.nodes.filter((node) => {
    const distance = Math.sqrt(Math.pow(node.position.x - 20, 2) + Math.pow(node.position.y - 20, 2))
    return distance > 25 // Nodes more than 25 units from center
  })

  // Adjust their positions to be closer to the center
  farNodes.forEach((node) => {
    // Calculate vector from center to node
    const dx = node.position.x - 20
    const dy = node.position.y - 20
    const distance = Math.sqrt(dx * dx + dy * dy)

    // Scale the vector to be at most 25 units long
    const scale = Math.min(1, 25 / distance)

    // Update node position
    node.position.x = 20 + dx * scale
    node.position.y = 20 + dy * scale
  })

  return worldData
}

// Call this function to adjust node positions
adjustNodePositions()
