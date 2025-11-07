# Rust + Next.js Full-Stack Template

A production-ready full-stack template with Rust backend (Axum) and Next.js frontend.

## Architecture

This template uses a two-server architecture where Rust is the main entry point:

- **Rust Server** (Port 3000) - Main entry point handling API routes
- **Next.js Server** (Port 3001) - Frontend with hot reload in dev, optimized build in production
- Rust proxies non-API requests to Next.js
- Browser connects to `http://localhost:3000` for everything

## Features

- **Rust Backend**: Fast, type-safe API server using Axum
- **Next.js Frontend**: Modern React framework with TypeScript
- **Single Port Access**: All requests go through Rust server on port 3000
- **CORS Handling**: Automatic CORS configuration
- **Hot Reload**: Development mode with Next.js hot module replacement
- **Production Ready**: Standalone Next.js build for optimal performance

## Quick Start

### Prerequisites

- Rust (latest stable)
- Node.js 18+
- [just](https://github.com/casey/just) command runner

### Development Mode

```bash
just src dev
```

This starts:

- Rust server on port 3000
- Next.js dev server on port 3001
- Visit `http://localhost:3000`

### Production Mode

```bash
just src prod
```

This starts:

- Rust server on port 3000
- Next.js standalone server on port 3001
- Visit `http://localhost:3000`

## Project Structure

```
.
├── src/
│   ├── api/           # API route handlers
│   ├── server/        # Server configuration and routing
│   └── bin/server.rs  # Main Rust entry point
├── app/
│   ├── lib/           # Frontend utilities and API client
│   ├── page.tsx       # Home page
│   └── layout.tsx     # Root layout
├── Cargo.toml         # Rust dependencies
├── package.json       # Node.js dependencies
└── justfile          # Build and run commands
```

## Environment Variables

Create a `.env.local` file:

```env
SERVER_PORT=3000
SERVER_HOST=127.0.0.1
PORT=3001
HOSTNAME=localhost
RUST_LOG=info
```

For remote access (e.g., Codespaces):

- Set `SERVER_HOST=0.0.0.0`
- Set `HOSTNAME=0.0.0.0` (dev mode only)

## Adding New API Routes

1. Add route handler in `src/api/mod.rs`:

```rust
async fn my_route() -> Json<MyResponse> {
    Json(MyResponse { /* ... */ })
}

pub fn routes() -> Router {
    Router::new()
        .route("/hello", get(hello))
        .route("/my-route", get(my_route))  // Add here
}
```

2. Call from frontend in `app/lib/api.ts`:

```typescript
export async function myRoute(): Promise<MyResponse> {
  const response = await fetch('/api/my-route')
  return handleResponse<MyResponse>(response)
}
```

## Customization

### Rename Project

1. Update `Cargo.toml`: Change `name = "rust-nextjs-template"`
2. Update `package.json`: Change `name` and `description`
3. Update `src/bin/server.rs`: Update import to use new crate name
4. Update `app/layout.tsx`: Change metadata title

### Update Dependencies

```bash
# Rust dependencies
cargo update

# Node.js dependencies
npm update
```

## License

MIT
