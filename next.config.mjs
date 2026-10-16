/** @type {import('next').NextConfig} */
const basePath = process.env.LM_BASE_PATH ?? "/luckymonkey"

const nextConfig = {
  basePath,
  reactStrictMode: true,
  poweredByHeader: false,
  images: { unoptimized: true },
  env: { NEXT_PUBLIC_BASE_PATH: basePath },
}

export default nextConfig
