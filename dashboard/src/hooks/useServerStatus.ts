'use client'

import { useQuery } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import type { ApiStatusResponse } from '@/types'

const STATUS_KEY = ['server-status']

interface UseServerStatusOptions {
  apiHost: string
  initialData?: ApiStatusResponse | null
}

export function useServerStatus({ apiHost, initialData }: UseServerStatusOptions) {
  const [uptime, setUptime] = useState(initialData?.uptime_seconds ?? 0)

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

  // Initialize uptime from server status
  useEffect(() => {
    if (query.data) {
      setUptime(query.data.uptime_seconds)
    }
  }, [query.data])

  // Uptime ticker - increment every second
  useEffect(() => {
    const timer = setInterval(() => {
      setUptime(prev => prev + 1)
    }, 1000)
    return () => clearInterval(timer)
  }, [])

  return {
    serverStatus: query.data ?? null,
    uptime,
    isLoading: query.isLoading,
    error: query.error,
    refetch: query.refetch,
  }
}
