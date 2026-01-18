'use client'

import type { ServiceMetrics } from '@/types'

interface SortIndicatorProps {
  column: keyof ServiceMetrics
  sortColumn: keyof ServiceMetrics
  sortDirection: 'asc' | 'desc'
}

export function SortIndicator({ column, sortColumn, sortDirection }: SortIndicatorProps) {
  const isActive = sortColumn === column
  const arrow = isActive ? (sortDirection === 'asc' ? '↑' : '↓') : '↕'

  return (
    <span className={`sort-indicator ${isActive ? 'active' : 'inactive'}`}>
      {arrow}
    </span>
  )
}
