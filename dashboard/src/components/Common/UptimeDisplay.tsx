'use client'

import { useEffect, useState } from 'react'
import { formatUptime } from '@/lib/formatters'

interface UptimeDisplayProps {
  initialSeconds: number
}

/**
 * Self-contained uptime display component.
 * Starts from initial value and ticks locally - no syncing with server.
 * Drift corrects on page refresh.
 */
export function UptimeDisplay({ initialSeconds }: UptimeDisplayProps) {
  // Use function initializer to capture initial value only once
  const [seconds, setSeconds] = useState(() => initialSeconds)

  // Tick every second - no dependencies, runs once on mount
  useEffect(() => {
    const timer = setInterval(() => {
      setSeconds(s => s + 1)
    }, 1000)
    return () => clearInterval(timer)
  }, [])

  return <>{formatUptime(seconds)}</>
}
