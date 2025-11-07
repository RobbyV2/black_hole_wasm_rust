import "../common.just"

# Build commands

# Build Rust for production (optimized)
build:
  cd {{ROOT}} && cargo build --release

# Build the API server binary for production
build-api:
  cd {{ROOT}} && cargo build --release --bin server

# Build Next.js for production
build-frontend:
  cd {{ROOT}} && bun run build
  @echo "Copying static assets for standalone build..."
  cd {{ROOT}} && cp -r .next/static .next/standalone/.next/static
  cd {{ROOT}} && cp -r public .next/standalone/public

# Build everything for production
build-all: build-api build-frontend
  @echo ""
  @echo "✓ Production build complete!"
  @echo ""
  @echo "Production deployment (two servers):"
  @echo "  API Binary: {{ROOT}}/target/release/server"
  @echo "  Frontend: {{ROOT}}/.next/standalone/server.js"
  @echo ""
  @echo "Run both with: just src prod"
  @echo "Or separately:"
  @echo "  API: just src start-api"
  @echo "  Frontend: just src start-frontend"

# Check Rust code without building
check:
  cd {{ROOT}} && cargo check

# Format and lint all code (Rust + TypeScript)
fmt:
  cd {{ROOT}} && cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings
  cd {{ROOT}} && cargo fmt --all
  cd {{ROOT}} && taplo fmt
  cd {{ROOT}} && bun run lint --fix
  cd {{ROOT}} && bunx prettier --write "**/*.{ts,tsx,js,jsx,json,md}"

# Check formatting and linting without making changes
fmt-check:
  cd {{ROOT}} && cargo fmt --check --all
  cd {{ROOT}} && cargo clippy --all-targets --all-features -- -D warnings
  cd {{ROOT}} && taplo fmt --check
  cd {{ROOT}} && bun run lint
  cd {{ROOT}} && bunx prettier --check "**/*.{ts,tsx,js,jsx,json,md}"

# Format Rust only
fmt-rust:
  cd {{ROOT}} && cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings
  cd {{ROOT}} && cargo fmt --all
  cd {{ROOT}} && taplo fmt

# Format TypeScript/Next.js only
fmt-ts:
  cd {{ROOT}} && bun run lint --fix
  cd {{ROOT}} && bunx prettier --write "**/*.{ts,tsx,js,jsx,json,md}"

# Clean build artifacts
clean:
  cd {{ROOT}} && cargo clean
  cd {{ROOT}} && rm -rf .next node_modules/.cache
