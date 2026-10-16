/** Prefix a public/ path with the app's base path (e.g. /luckymonkey). */
export const asset = (path: string) => `${process.env.NEXT_PUBLIC_BASE_PATH ?? ""}${path}`
