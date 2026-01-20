'use client'

import { useState, useEffect, type ReactNode } from 'react'

type TooltipPosition = 'top' | 'bottom' | 'left' | 'right' | 'bottom-end' | 'bottom-start'

interface TooltipProps {
  content: ReactNode
  children: ReactNode
  position?: TooltipPosition
}

export function Tooltip({
  content,
  children,
  position = 'top',
}: TooltipProps) {
  // Render tooltip content only on client to avoid hydration mismatch
  // (metric values change between SSR and client)
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    // This is intentional - we need to detect client-side rendering to avoid hydration mismatch
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setMounted(true)
  }, [])

  return (
    <span className={`tooltip-trigger tooltip-${position}`}>
      {children}
      <span className="tooltip-bubble" role="tooltip">
        {mounted ? content : null}
      </span>
    </span>
  )
}

// Keep for backwards compatibility but not used
export const TooltipProvider = ({ children }: { children: ReactNode; delayDuration?: number }) => children
