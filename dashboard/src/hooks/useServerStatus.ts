'use client'

import { useQuery } from '@tanstack/react-query'
import type { ApiStatusResponse } from '@/types'

const STATUS_KEY = ['server-status']

interface UseServerStatusOptions {
  apiHost: string
  initialData?: ApiStatusResponse | null
}

export function useServerStatus({ apiHost, initialData }: UseServerStatusOptions) {
  const query = useQuery({
    queryKey: STATUS_KEY,
    queryFn: async () => {
      const response = await fetch(`http://${apiHost}/status`)
      const status: ApiStatusResponse = await response.json()
      return status
    },
    refetchInterval: 30000, // Refetch every 30 seconds
    initialData: initialData ?? undefined,
    // Prevent immediate refetch if we have SSR data
    staleTime: initialData ? 30000 : 0,
    // Disable automatic refetches that cause "jumping"
    refetchOnMount: !initialData,
    refetchOnWindowFocus: false,
  })

  return {
    serverStatus: query.data ?? null,
    isLoading: query.isLoading,
    error: query.error,
    refetch: query.refetch,
  }
}
