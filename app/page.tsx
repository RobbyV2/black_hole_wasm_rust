'use client'

import { useState } from 'react'
import { hello, HelloResponse } from './lib/api'

export default function Home() {
  const [response, setResponse] = useState<HelloResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleClick = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await hello()
      setResponse(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="bg-white rounded-lg shadow-xl p-8 max-w-md w-full">
        <h1 className="text-3xl font-bold text-gray-800 mb-6 text-center">
          Rust + Next.js Template
        </h1>

        <p className="text-gray-600 mb-8 text-center">
          A full-stack template with Rust backend and Next.js frontend
        </p>

        <button
          onClick={handleClick}
          disabled={loading}
          className="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors disabled:bg-gray-400 disabled:cursor-not-allowed"
        >
          {loading ? 'Loading...' : 'Call Rust API'}
        </button>

        {response && (
          <div className="mt-6 p-4 bg-green-50 border border-green-200 rounded-lg">
            <p className="text-green-800 font-medium">Response from Rust:</p>
            <p className="text-green-700 mt-2">{response.message}</p>
          </div>
        )}

        {error && (
          <div className="mt-6 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-800 font-medium">Error:</p>
            <p className="text-red-700 mt-2">{error}</p>
          </div>
        )}
      </div>
    </div>
  )
}
