'use client'

import type { ReactNode } from 'react'

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
  return (
    <span className={`tooltip-trigger tooltip-${position}`}>
      {children}
      <span className="tooltip-bubble" role="tooltip">
        {content}
      </span>
    </span>
  )
}

// Keep for backwards compatibility but not used
export const TooltipProvider = ({ children }: { children: ReactNode; delayDuration?: number }) => children
