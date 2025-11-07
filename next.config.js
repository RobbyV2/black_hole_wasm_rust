/** @type {import('next').NextConfig} */

const basePath = process.env.NEXT_PUBLIC_BASE_PATH || ''

const nextConfig = {
  output: process.env.GITHUB_ACTIONS ? 'export' : 'standalone',
  basePath: basePath,
  assetPrefix: basePath,
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
}

module.exports = nextConfig
