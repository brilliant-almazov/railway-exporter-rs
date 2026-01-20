'use client'

import { useState, useEffect } from 'react'

/**
 * Detect when user scrolls past a threshold and switch to compact mode.
 * @param threshold - Scroll position in pixels to trigger compact mode (default 50)
 */
export function useCompactMode(threshold = 50): boolean {
  const [isCompact, setIsCompact] = useState(false)

  useEffect(() => {
    const handleScroll = () => {
      setIsCompact(window.scrollY > threshold)
    }

    window.addEventListener('scroll', handleScroll, { passive: true })
    handleScroll() // Check initial position

    return () => window.removeEventListener('scroll', handleScroll)
  }, [threshold])

  return isCompact
}
