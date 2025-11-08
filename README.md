# Black Hole Simulation

WebGPU-based black hole raytracer with gravitational lensing

## Tech Stack

Rust WASM + WebGPU compute shaders + Next.js

## Features

Geodesic raytracing around Schwarzschild black hole

Gravitational lensing of Milky Way background

Accretion disk visualization

Elliptical planet orbit with Keplerian dynamics

Interactive camera controls

## Physics

Leapfrog integration in geometric units (r_s = 2)
2000 integration steps per ray
Schwarzschild metric: u'' = -u(1 - 1.5u²)
Event horizon at r = 2M

## Running

```bash
# build wasm files first
cd wasm && wasm-pack build --target web --out-dir ../public/wasm --release
just src dev
# or for production
just src prod
```

Visit http://localhost:3000/blackhole

## Controls

Left-click drag: orbit camera
Scroll: zoom

## Deployment

GitHub Actions auto-deploys to GitHub Pages on push to main
Enable Pages in repo settings: Settings > Pages > Source: GitHub Actions
