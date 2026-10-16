export const TILE_WIDTH = 64
export const TILE_HEIGHT = 32

/** Isometric grid position -> screen offset (2:1 diamonds). */
export function isoToScreen(x: number, y: number, height = 0) {
  if (!Number.isFinite(x) || !Number.isFinite(y)) return { x: 0, y: 0 }
  const h = Number.isFinite(height) ? height : 0
  return { x: (x - y) * (TILE_WIDTH / 2), y: (x + y) * (TILE_HEIGHT / 2) - h * 8 }
}

/** Small deterministic PRNG so the world looks the same on every visit. */
export function mulberry32(seed: number) {
  let a = seed >>> 0
  return () => {
    a = (a + 0x6d2b79f5) >>> 0
    let t = a
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

/** Stable pseudo-random number in [0,1) for a grid cell. */
export function cellHash(x: number, y: number) {
  let h = Math.imul(x | 0, 374761393) + Math.imul(y | 0, 668265263)
  h = Math.imul(h ^ (h >>> 13), 1274126177)
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296
}

export const tileKey = (x: number, y: number) => `${Math.round(x)},${Math.round(y)}`
