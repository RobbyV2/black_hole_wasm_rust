export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public data?: any
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }))
    throw new ApiError(error.error || response.statusText, response.status, error)
  }

  return response.json()
}

export interface HelloResponse {
  message: string
}

export async function hello(): Promise<HelloResponse> {
  const response = await fetch('/api/hello')
  return handleResponse<HelloResponse>(response)
}
