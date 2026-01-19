'use client'

import { useEffect, useState } from 'react'

interface TimeDisplayProps {
  /** Unix timestamp in seconds (not milliseconds) */
  timestamp: number
  locale: string
  placeholder?: string
}

/**
 * Client-only time display component.
 * Shows placeholder on initial render to avoid hydration mismatch,
 * then displays formatted time after hydration.
 *
 * This avoids the Node.js vs Browser toLocaleTimeString() differences.
 */
export function TimeDisplay({ timestamp, locale, placeholder = '—' }: TimeDisplayProps) {
  const [time, setTime] = useState<string | null>(null)

  useEffect(() => {
    // Format time only on client after hydration
    const date = new Date(timestamp * 1000)
    setTime(date.toLocaleTimeString(locale))
  }, [timestamp, locale])

  // Return placeholder during SSR and first client render
  return <>{time ?? placeholder}</>
}
