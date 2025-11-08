'use client'

import { useEffect, useRef, useState } from 'react'
import { getBasePath } from '../lib/basePath'

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
  const cameraInfoIntervalRef = useRef<number | null>(null)

  useEffect(() => {
    let mounted = true

    async function initWasm() {
      try {
        if (!canvasRef.current) {
          setError('Canvas not found')
          return
        }

        setStatus('Loading WASM module...')

        const basePath = getBasePath()
        const wasm = (await import(
          /* webpackIgnore: true */ `${basePath}/wasm/black_hole_wasm.js`
        )) as WasmModule

        if (!mounted) return

        await wasm.default(`${basePath}/wasm/black_hole_wasm_bg.wasm`)

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
              animationFrameRef.current = requestAnimationFrame(animate)
            } catch (err) {
              console.error('Render error:', err)
              setError(`Render error: ${err}`)
            }
          }
        }

        const updateCameraInfo = () => {
          if (rendererRef.current && mounted) {
            const info = rendererRef.current.camera_info()
            setCameraInfo(info)
          }
        }

        animate()
        updateCameraInfo()
        cameraInfoIntervalRef.current = window.setInterval(updateCameraInfo, 100)
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
      if (cameraInfoIntervalRef.current) {
        clearInterval(cameraInfoIntervalRef.current)
      }
    }
  }, [])

  useEffect(() => {
    const canvas = canvasRef.current
    const renderer = rendererRef.current
    if (!canvas || !renderer) return

    const handleResize = () => {
      if (canvas && renderer) {
        const { clientWidth, clientHeight } = canvas
        canvas.width = clientWidth
        canvas.height = clientHeight
        renderer.resize(clientWidth, clientHeight)
      }
    }

    window.addEventListener('resize', handleResize)
    handleResize()

    const handleMouseDown = (e: MouseEvent) => {
      if (renderer) {
        const rect = canvas.getBoundingClientRect()
        const x = e.clientX - rect.left
        const y = e.clientY - rect.top
        renderer.on_mouse_button(e.button, true, x, y)
      }
    }

    const handleMouseUp = (e: MouseEvent) => {
      if (renderer) {
        const rect = canvas.getBoundingClientRect()
        const x = e.clientX - rect.left
        const y = e.clientY - rect.top
        renderer.on_mouse_button(e.button, false, x, y)
      }
    }

    const handleMouseMove = (e: MouseEvent) => {
      if (renderer) {
        const rect = canvas.getBoundingClientRect()
        const x = e.clientX - rect.left
        const y = e.clientY - rect.top
        renderer.on_mouse_move(x, y)
      }
    }

    const handleWheel = (e: WheelEvent) => {
      e.preventDefault()
      if (renderer) {
        renderer.on_wheel(e.deltaY * 0.01)
      }
    }

    window.addEventListener('resize', handleResize)
    canvas.addEventListener('mousedown', handleMouseDown)
    canvas.addEventListener('mouseup', handleMouseUp)
    canvas.addEventListener('mousemove', handleMouseMove)
    canvas.addEventListener('wheel', handleWheel, { passive: false })

    return () => {
      window.removeEventListener('resize', handleResize)
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
