'use client'

import { QueryClientProvider } from '@tanstack/react-query'
import { TooltipProvider } from '@/components/Common/Tooltip'
import { queryClient } from '@/lib/queryClient'
import { ReactNode } from 'react'

interface QueryProviderProps {
  children: ReactNode
}

export function QueryProvider({ children }: QueryProviderProps) {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider delayDuration={150}>
        {children}
      </TooltipProvider>
    </QueryClientProvider>
  )
}
