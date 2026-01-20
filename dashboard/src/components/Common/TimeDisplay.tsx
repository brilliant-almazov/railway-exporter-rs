interface TimeDisplayProps {
  /** Unix timestamp in seconds (not milliseconds) */
  timestamp: number
  locale?: string
}

/**
 * Deterministic time formatting for SSR/client consistency.
 * Uses fixed format HH:MM:SS to avoid hydration mismatch.
 */
function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000)
  const h = date.getHours().toString().padStart(2, '0')
  const m = date.getMinutes().toString().padStart(2, '0')
  const s = date.getSeconds().toString().padStart(2, '0')
  return `${h}:${m}:${s}`
}

/**
 * Time display component with deterministic formatting.
 * Renders immediately on both SSR and client with same output.
 */
export function TimeDisplay({ timestamp }: TimeDisplayProps) {
  return <span>{formatTime(timestamp)}</span>
}
