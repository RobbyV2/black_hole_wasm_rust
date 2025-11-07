'use client'

import { useEffect, useRef, useState } from 'react'

type WasmModule = {
  BlackHoleRenderer: {
    new: (canvas: HTMLCanvasElement) => Promise<Renderer>
  }
  default(path?: string): Promise<void>
}

type Renderer = {
  render(): void
  resize(width: number, height: number): void
  on_mouse_move(x: number, y: number): void
  on_mouse_button(button: number, pressed: boolean, x: number, y: number): void
  on_wheel(delta_y: number): void
  camera_info(): string
}

export default function BlackHolePage() {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [status, setStatus] = useState<string>('Loading...')
  const [error, setError] = useState<string | null>(null)
  const [cameraInfo, setCameraInfo] = useState<string>('')
  const rendererRef = useRef<Renderer | null>(null)
  const animationFrameRef = useRef<number | null>(null)

  useEffect(() => {
    let mounted = true

    async function initWasm() {
      try {
        if (!canvasRef.current) {
          setError('Canvas not found')
          return
        }

        setStatus('Loading WASM module...')

        // Load the WASM module dynamically at runtime
        const wasmModule = (await fetch('/wasm/black_hole_wasm.js')
          .then(res => {
            if (!res.ok) throw new Error(`Failed to load WASM JS: ${res.status}`)
            return res.text()
          })
          .then(code => {
            // Create a blob URL and import it
            const blob = new Blob([code], { type: 'application/javascript' })
            const url = URL.createObjectURL(blob)
            return import(/* webpackIgnore: true */ url)
          })) as Promise<WasmModule>

        const wasm = await wasmModule

        if (!mounted) return

        // Initialize the WASM module first (loads the .wasm file)
        // Pass the full URL to the .wasm file since we're using a blob URL for the JS
        await wasm.default('/wasm/black_hole_wasm_bg.wasm')

        if (!mounted) return

        setStatus('Initializing WebGPU...')

        // Call the static async new method (not a constructor)
        const renderer = await wasm.BlackHoleRenderer.new(canvasRef.current)

        if (!mounted) return

        rendererRef.current = renderer
        setStatus('Ready')
        setError(null)

        const animate = () => {
          if (rendererRef.current && mounted) {
            try {
              rendererRef.current.render()
              const info = rendererRef.current.camera_info()
              setCameraInfo(info)
              animationFrameRef.current = requestAnimationFrame(animate)
            } catch (err) {
              console.error('Render error:', err)
              setError(`Render error: ${err}`)
            }
          }
        }

        animate()
      } catch (err) {
        console.error('WASM init error:', err)
        if (mounted) {
          setError(`Failed to initialize: ${err}`)
          setStatus('Error')
        }
      }
    }

    initWasm()

    return () => {
      mounted = false
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current)
      }
    }
  }, [])

  useEffect(() => {
    if (!canvasRef.current || !rendererRef.current) return

    const handleResize = () => {
      if (canvasRef.current && rendererRef.current) {
        const { clientWidth, clientHeight } = canvasRef.current
        canvasRef.current.width = clientWidth
        canvasRef.current.height = clientHeight
        rendererRef.current.resize(clientWidth, clientHeight)
      }
    }

    window.addEventListener('resize', handleResize)
    handleResize()

    return () => window.removeEventListener('resize', handleResize)
  }, [status])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || !rendererRef.current) return

    const handleMouseDown = (e: MouseEvent) => {
      if (rendererRef.current && canvas) {
        const rect = canvas.getBoundingClientRect()
        const x = e.clientX - rect.left
        const y = e.clientY - rect.top
        rendererRef.current.on_mouse_button(e.button, true, x, y)
      }
    }

    const handleMouseUp = (e: MouseEvent) => {
      if (rendererRef.current && canvas) {
        const rect = canvas.getBoundingClientRect()
        const x = e.clientX - rect.left
        const y = e.clientY - rect.top
        rendererRef.current.on_mouse_button(e.button, false, x, y)
      }
    }

    const handleMouseMove = (e: MouseEvent) => {
      if (rendererRef.current && canvas) {
        const rect = canvas.getBoundingClientRect()
        const x = e.clientX - rect.left
        const y = e.clientY - rect.top
        rendererRef.current.on_mouse_move(x, y)
      }
    }

    const handleWheel = (e: WheelEvent) => {
      e.preventDefault()
      if (rendererRef.current) {
        rendererRef.current.on_wheel(e.deltaY * 0.01)
      }
    }

    canvas.addEventListener('mousedown', handleMouseDown)
    canvas.addEventListener('mouseup', handleMouseUp)
    canvas.addEventListener('mousemove', handleMouseMove)
    canvas.addEventListener('wheel', handleWheel, { passive: false })

    return () => {
      canvas.removeEventListener('mousedown', handleMouseDown)
      canvas.removeEventListener('mouseup', handleMouseUp)
      canvas.removeEventListener('mousemove', handleMouseMove)
      canvas.removeEventListener('wheel', handleWheel)
    }
  }, [status])

  return (
    <div className="relative w-screen h-screen overflow-hidden bg-black">
      <canvas ref={canvasRef} className="w-full h-full" width={800} height={600} />

      {/* Overlay controls - hidden by default, show on hover */}
      <div className="absolute top-0 left-0 p-4 text-white opacity-0 hover:opacity-100 transition-opacity duration-300 bg-gradient-to-r from-black/70 to-transparent pointer-events-none">
        <div className="pointer-events-auto">
          <h1 className="text-xl font-bold">Black Hole Simulation</h1>
          <div className="mt-2 text-sm">
            <span className="mr-4">
              Status:{' '}
              <span className={status === 'Ready' ? 'text-green-400' : 'text-yellow-400'}>
                {status}
              </span>
            </span>
            {error && <div className="text-red-400">Error: {error}</div>}
          </div>
          {cameraInfo && <div className="mt-2 text-xs text-gray-400">{cameraInfo}</div>}
          <div className="mt-3 text-xs text-gray-400">
            <p>Controls: Left-click + drag to orbit | Scroll to zoom</p>
            <p className="mt-1">Simulating Sagittarius A* with WebGPU geodesic raytracing</p>
          </div>
        </div>
      </div>
    </div>
  )
}
