import { tileKey } from "@/lib/iso"

/** Tile type by "x,y" grid key. */
export type TileIndex = Map<string, number>

export interface NPC {
  id: string
  type: "monkey" | "ghost" | "banana" | "lizard" | "crocodile"
  x: number
  y: number
  height: number
  direction: number // 0-7 for 8 directions
  speed: number
  state: "idle" | "moving" | "hopping" | "chasing" | "fleeing" | "swimming"
  emoji: string
  onTile: boolean // Whether the NPC is on a valid tile
  spawnTime: number // When the NPC was spawned
  lifespan: number // How long the NPC should live in milliseconds
}

// Maximum number of NPCs to show at once
export const MAX_ACTIVE_NPCS = 20

// Spawn chance (per update)
export const NPC_SPAWN_CHANCE = 0.08

// Viewport padding for spawning
export const VIEWPORT_PADDING = 200

// NPC lifespans in milliseconds (30-90 seconds)
export const NPC_LIFESPANS = {
  SHORT: 30000, // 30 seconds
  MEDIUM: 60000, // 60 seconds
  LONG: 90000, // 90 seconds
}

// Initial NPC data
export const initialNPCs: NPC[] = [
  // Monkeys
  {
    id: "monkey-1",
    type: "monkey",
    x: 15,
    y: 18,
    height: 1,
    direction: 0,
    speed: 0.05,
    state: "moving",
    emoji: "🐒",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "monkey-2",
    type: "monkey",
    x: 25,
    y: 15,
    height: 0,
    direction: 2,
    speed: 0.03,
    state: "moving",
    emoji: "🐒",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "monkey-3",
    type: "monkey",
    x: 20,
    y: 25,
    height: 0,
    direction: 4,
    speed: 0.04,
    state: "moving",
    emoji: "🐒",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },

  // Ghosts
  {
    id: "ghost-1",
    type: "ghost",
    x: 12,
    y: 22,
    height: 0,
    direction: 1,
    speed: 0.06,
    state: "moving",
    emoji: "👻",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "ghost-2",
    type: "ghost",
    x: 28,
    y: 18,
    height: 0,
    direction: 3,
    speed: 0.07,
    state: "moving",
    emoji: "👻",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "ghost-3",
    type: "ghost",
    x: 18,
    y: 10,
    height: 0,
    direction: 5,
    speed: 0.05,
    state: "moving",
    emoji: "👻",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },

  // Hopping Banana
  {
    id: "banana-1",
    type: "banana",
    x: 30,
    y: 22,
    height: 0,
    direction: 0,
    speed: 0.08,
    state: "hopping",
    emoji: "🍌",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  // More monkeys
  {
    id: "monkey-4",
    type: "monkey",
    x: 32,
    y: 5,
    height: 0,
    direction: 6,
    speed: 0.04,
    state: "moving",
    emoji: "🐒",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "monkey-5",
    type: "monkey",
    x: 10,
    y: 10,
    height: 3,
    direction: 1,
    speed: 0.05,
    state: "moving",
    emoji: "🐒",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },

  // More ghosts
  {
    id: "ghost-4",
    type: "ghost",
    x: 15,
    y: 28,
    height: 0,
    direction: 7,
    speed: 0.06,
    state: "moving",
    emoji: "👻",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "ghost-5",
    type: "ghost",
    x: 25,
    y: 5,
    height: 3,
    direction: 2,
    speed: 0.05,
    state: "moving",
    emoji: "👻",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },

  // More bananas
  {
    id: "banana-2",
    type: "banana",
    x: 17,
    y: 17,
    height: 4,
    direction: 3,
    speed: 0.07,
    state: "hopping",
    emoji: "🍌",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "banana-3",
    type: "banana",
    x: 25,
    y: 28,
    height: 2,
    direction: 5,
    speed: 0.09,
    state: "hopping",
    emoji: "🍌",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },

  // New lizards along the river
  {
    id: "lizard-1",
    type: "lizard",
    x: 19,
    y: 12,
    height: 0,
    direction: 2,
    speed: 0.03,
    state: "swimming",
    emoji: "🦎",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "lizard-2",
    type: "lizard",
    x: 22,
    y: 16,
    height: 0,
    direction: 4,
    speed: 0.04,
    state: "swimming",
    emoji: "🦎",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "lizard-3",
    type: "lizard",
    x: 18,
    y: 24,
    height: 0,
    direction: 0,
    speed: 0.03,
    state: "swimming",
    emoji: "🦎",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },

  // Crocodiles in the river
  {
    id: "crocodile-1",
    type: "crocodile",
    x: 21,
    y: 14,
    height: 0,
    direction: 3,
    speed: 0.02,
    state: "swimming",
    emoji: "🐊",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "crocodile-2",
    type: "crocodile",
    x: 17,
    y: 20,
    height: 0,
    direction: 5,
    speed: 0.02,
    state: "swimming",
    emoji: "🐊",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  
  // Additional monkeys around the temple
  {
    id: "monkey-6",
    type: "monkey",
    x: 18,
    y: 16,
    height: 4,
    direction: 3,
    speed: 0.04,
    state: "moving",
    emoji: "🐒",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.LONG,
  },
  {
    id: "monkey-7",
    type: "monkey",
    x: 16,
    y: 18,
    height: 3,
    direction: 5,
    speed: 0.05,
    state: "moving",
    emoji: "🐒",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.LONG,
  },
  
  // More ghosts in crashed sector
  {
    id: "ghost-6",
    type: "ghost",
    x: 8,
    y: 28,
    height: 0,
    direction: 2,
    speed: 0.06,
    state: "moving",
    emoji: "👻",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "ghost-7",
    type: "ghost",
    x: 6,
    y: 26,
    height: 0,
    direction: 4,
    speed: 0.05,
    state: "moving",
    emoji: "👻",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  
  // More lizards along the extended river
  {
    id: "lizard-4",
    type: "lizard",
    x: 23,
    y: 33,
    height: 0,
    direction: 1,
    speed: 0.03,
    state: "swimming",
    emoji: "🦎",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  {
    id: "lizard-5",
    type: "lizard",
    x: 27,
    y: 38,
    height: 0,
    direction: 6,
    speed: 0.04,
    state: "swimming",
    emoji: "🦎",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.MEDIUM,
  },
  
  // More bananas bouncing around
  {
    id: "banana-4",
    type: "banana",
    x: 28,
    y: 25,
    height: 1,
    direction: 7,
    speed: 0.08,
    state: "hopping",
    emoji: "🍌",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.SHORT,
  },
  {
    id: "banana-5",
    type: "banana",
    x: 12,
    y: 30,
    height: 0,
    direction: 2,
    speed: 0.07,
    state: "hopping",
    emoji: "🍌",
    onTile: true,
    spawnTime: Date.now(),
    lifespan: NPC_LIFESPANS.SHORT,
  },
]

// Direction vectors for 8 directions
export const directionVectors = [
  { x: 0, y: -1 }, // North
  { x: 1, y: -1 }, // Northeast
  { x: 1, y: 0 }, // East
  { x: 1, y: 1 }, // Southeast
  { x: 0, y: 1 }, // South
  { x: -1, y: 1 }, // Southwest
  { x: -1, y: 0 }, // West
  { x: -1, y: -1 }, // Northwest
]

// Map boundaries
const MAP_MIN_X = 5
const MAP_MAX_X = 35
const MAP_MIN_Y = 5
const MAP_MAX_Y = 35

// Check if a position is within map boundaries
function isWithinMapBounds(x: number, y: number): boolean {
  return x >= MAP_MIN_X && x <= MAP_MAX_X && y >= MAP_MIN_Y && y <= MAP_MAX_Y
}

// Update NPC positions based on their state and direction
export function updateNPCs(npcs: NPC[], validTiles: TileIndex): NPC[] {
  return npcs.map((npc) => {
    // Clone the NPC to avoid mutating the original
    const updatedNPC = { ...npc }

    // Handle different NPC types and states
    switch (npc.type) {
      case "monkey":
        return updateMonkey(updatedNPC, validTiles)
      case "ghost":
        return updateGhost(updatedNPC, validTiles)
      case "banana":
        return updateBanana(updatedNPC)
      case "lizard":
        return updateLizard(updatedNPC, validTiles)
      case "crocodile":
        return updateCrocodile(updatedNPC, validTiles)
      default:
        return updatedNPC
    }
  })
}

// Update monkey NPCs
function updateMonkey(npc: NPC, validTiles: TileIndex): NPC {
  // Get the direction vector
  const dirVector = directionVectors[npc.direction]

  // Calculate new position
  const newX = npc.x + dirVector.x * npc.speed
  const newY = npc.y + dirVector.y * npc.speed

  // Check if new position is within map bounds and on a valid tile
  if (isWithinMapBounds(newX, newY) && isOnValidTile(newX, newY, validTiles)) {
    npc.x = newX
    npc.y = newY
    npc.onTile = true
  } else {
    // Turn around if hitting map edge or invalid tile
    npc.direction = (npc.direction + 4) % 8
    npc.onTile = false
  }

  // Randomly change direction occasionally, but only if on a tile
  if (npc.onTile && Math.random() < 0.02) {
    npc.direction = Math.floor(Math.random() * 8)
  }

  return npc
}

// Update ghost NPCs
function updateGhost(npc: NPC, validTiles: TileIndex): NPC {
  // Ghosts move more erratically
  const dirVector = directionVectors[npc.direction]

  // Calculate new position
  const newX = npc.x + dirVector.x * npc.speed
  const newY = npc.y + dirVector.y * npc.speed

  // Check if new position is within map bounds and on a valid tile
  if (isWithinMapBounds(newX, newY) && isOnValidTile(newX, newY, validTiles)) {
    npc.x = newX
    npc.y = newY
    npc.onTile = true
  } else {
    // Turn around if hitting map edge or invalid tile
    npc.direction = (npc.direction + 4) % 8
    npc.onTile = false
  }

  // Ghosts change direction more frequently, but only if on a tile
  if (npc.onTile && Math.random() < 0.05) {
    npc.direction = Math.floor(Math.random() * 8)
  }

  return npc
}

// Update banana NPCs - bananas can move anywhere but change direction more often
function updateBanana(npc: NPC): NPC {
  // Bananas hop in their direction
  const dirVector = directionVectors[npc.direction]

  // Calculate new position
  const newX = npc.x + dirVector.x * npc.speed
  const newY = npc.y + dirVector.y * npc.speed

  // Check if new position is within map bounds
  if (isWithinMapBounds(newX, newY)) {
    npc.x = newX
    npc.y = newY
    npc.onTile = true
  } else {
    // Turn around if hitting map edge
    npc.direction = (npc.direction + 4) % 8
    npc.onTile = false
  }

  // Simulate hopping by changing height
  npc.height = Math.abs(Math.sin(Date.now() * 0.01)) * 0.5

  // Change direction more frequently for bananas
  if (Math.random() < 0.05) {
    npc.direction = Math.floor(Math.random() * 8)
  }

  return npc
}

// Update lizard NPCs
function updateLizard(npc: NPC, validTiles: TileIndex): NPC {
  // Lizards move along the river
  const dirVector = directionVectors[npc.direction]

  // Calculate new position
  const newX = npc.x + dirVector.x * npc.speed
  const newY = npc.y + dirVector.y * npc.speed

  // Check if new position is within map bounds and on a water tile
  if (isWithinMapBounds(newX, newY) && isOnValidTile(newX, newY, validTiles, true)) {
    npc.x = newX
    npc.y = newY
    npc.onTile = true
  } else {
    // Turn around if hitting map edge or non-water tile
    npc.direction = (npc.direction + 4) % 8
    npc.onTile = false
  }

  // Lizards change direction occasionally
  if (npc.onTile && Math.random() < 0.03) {
    npc.direction = Math.floor(Math.random() * 8)
  }

  return npc
}

// Update crocodile NPCs
function updateCrocodile(npc: NPC, validTiles: TileIndex): NPC {
  // Crocodiles move slowly in the river
  const dirVector = directionVectors[npc.direction]

  // Calculate new position
  const newX = npc.x + dirVector.x * npc.speed
  const newY = npc.y + dirVector.y * npc.speed

  // Check if new position is within map bounds and on a water tile
  if (isWithinMapBounds(newX, newY) && isOnValidTile(newX, newY, validTiles, true)) {
    npc.x = newX
    npc.y = newY
    npc.onTile = true
  } else {
    // Turn around if hitting map edge or non-water tile
    npc.direction = (npc.direction + 4) % 8
    npc.onTile = false
  }

  // Crocodiles rarely change direction
  if (npc.onTile && Math.random() < 0.01) {
    npc.direction = Math.floor(Math.random() * 8)
  }

  return npc
}

// Get a random NPC from the pool based on type
export function getRandomNPC(type?: string): NPC {
  // Filter by type if specified
  const npcPool = type ? initialNPCs.filter((npc) => npc.type === type) : initialNPCs

  // Get a random NPC from the pool
  const randomNPC = { ...npcPool[Math.floor(Math.random() * npcPool.length)] }

  // Generate a unique ID
  randomNPC.id = `${randomNPC.type}-${Date.now()}-${Math.floor(Math.random() * 1000)}`

  // Randomize position slightly
  randomNPC.x += Math.random() * 4 - 2
  randomNPC.y += Math.random() * 4 - 2

  // Randomize direction
  randomNPC.direction = Math.floor(Math.random() * 8)

  // Set spawn time and lifespan
  randomNPC.spawnTime = Date.now()

  // Assign a random lifespan between 30-90 seconds
  const lifespans = [NPC_LIFESPANS.SHORT, NPC_LIFESPANS.MEDIUM, NPC_LIFESPANS.LONG]
  randomNPC.lifespan = lifespans[Math.floor(Math.random() * lifespans.length)]

  return randomNPC
}

// Check if a position is on a valid tile (O(1) lookup instead of scanning every tile)
function isOnValidTile(x: number, y: number, tiles: TileIndex, waterOnly = false): boolean {
  const type = tiles.get(tileKey(x, y))
  if (type === undefined) return false
  return waterOnly ? type === 1 : true
}
