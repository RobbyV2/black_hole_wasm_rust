// Get the base path for the application
// In development, this is empty. In production on GitHub Pages, it's the repo name
export function getBasePath(): string {
  if (typeof window === 'undefined') {
    return ''
  }

  // Try multiple methods to get the base path

  // Method 1: Check __NEXT_DATA__.p (basePath)
  const nextData = (window as any).__NEXT_DATA__
  if (nextData?.p) {
    return nextData.p.trim()
  }

  // Method 2: Check assetPrefix
  if (nextData?.assetPrefix) {
    return nextData.assetPrefix.trim()
  }

  // Method 3: Parse from current pathname
  // If we're on GitHub Pages, the pathname will be /repo-name/...
  const pathname = window.location.pathname
  const match = pathname.match(/^(\/[^\/]+)\//)
  if (match && match[1] !== '/blackhole') {
    return match[1]
  }

  return ''
}
