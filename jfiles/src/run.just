import "../common.just"

# Run commands

# === DEVELOPMENT MODE ===
# Rust server on SERVER_PORT (3000) - proxies to Next.js dev server
# Next.js dev server on PORT (3001) - hot reload

# Run development mode (Rust + Next.js dev server)
dev:
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ROOT}}"
  trap 'kill 0' SIGINT
  PORT=3001 bun run dev 2>&1 | sed 's/^/[WEB] /' &
  sleep 2
  SERVER_PROXY_URL=http://127.0.0.1:3001 cargo run --bin server 2>&1 | sed 's/^/[RUST] /' &
  wait

# Run Rust server only (dev mode with proxy)
api:
  cd {{ROOT}} && SERVER_PROXY_URL=http://127.0.0.1:3001 cargo run --bin server

# Run Next.js dev server only
frontend:
  cd {{ROOT}} && PORT=3001 bun run dev

# === PRODUCTION MODE ===
# Rust server on SERVER_PORT (3000) - proxies to Next.js standalone server
# Next.js standalone server on PORT (3001) - optimized production server

# Build and run production
prod:
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ROOT}}"
  echo "Building frontend..."
  bun run build
  echo "Copying static files for standalone..."
  cp -r public .next/standalone/ 2>/dev/null || true
  cp -r .next/static .next/standalone/.next/ 2>/dev/null || true
  echo "Building backend..."
  cargo build --release --bin server
  echo "Starting servers..."
  trap 'kill 0' SIGINT
  PORT=3001 HOSTNAME=127.0.0.1 bun run ./.next/standalone/server.js 2>&1 | sed 's/^/[WEB] /' &
  sleep 2
  SERVER_PROXY_URL=http://127.0.0.1:3001 ./target/release/server 2>&1 | sed 's/^/[RUST] /' &
  wait

# Build everything for production
build-prod:
  #!/usr/bin/env bash
  cd "{{ROOT}}"
  echo "Building frontend..."
  bun run build
  echo "Copying static files..."
  cp -r public .next/standalone/ 2>/dev/null || true
  cp -r .next/static .next/standalone/.next/ 2>/dev/null || true
  echo "Building backend..."
  cargo build --release --bin server

# Run production (assumes already built)
start-prod:
  #!/usr/bin/env bash
  cd "{{ROOT}}"
  trap 'kill 0' SIGINT
  PORT=3001 bun run ./.next/standalone/server.js 2>&1 | sed 's/^/[WEB] /' &
  sleep 2
  SERVER_PROXY_URL=http://127.0.0.1:3001 ./target/release/server 2>&1 | sed 's/^/[RUST] /' &
  wait

