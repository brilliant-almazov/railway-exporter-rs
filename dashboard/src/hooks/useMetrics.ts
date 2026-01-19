'use client'

import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useRef, useCallback } from 'react'
import { fetchMetrics, mapApiToMetrics } from '@/lib/api'
import type { ParsedMetrics, WsMessage, ApiMetricsJson } from '@/types'

const METRICS_KEY = ['metrics']

interface UseMetricsOptions {
  apiHost: string
  useWebSocket?: boolean
  pollingInterval?: number
  initialData?: ParsedMetrics | null
}

export function useMetrics({
  apiHost,
  useWebSocket = true,
  pollingInterval = 30000,
  initialData,
}: UseMetricsOptions) {
  const queryClient = useQueryClient()
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  // Skip first WS update if we have SSR data (prevents hydration jump)
  const skipFirstWsUpdate = useRef(!!initialData)

  const metricsUrl = `http://${apiHost}/metrics`
  const wsUrl = `ws://${apiHost}/ws`

  // Fetch metrics via HTTP
  const query = useQuery({
    queryKey: METRICS_KEY,
    queryFn: () => fetchMetrics(metricsUrl),
    refetchInterval: useWebSocket ? false : pollingInterval,
    initialData: initialData ?? undefined,
    // Prevent immediate refetch if we have SSR data
    staleTime: initialData ? pollingInterval : 0,
    // Disable automatic refetches that cause "jumping"
    refetchOnMount: !initialData,
    refetchOnWindowFocus: false,
  })

  // Handle WebSocket message
  const handleWsMessage = useCallback((data: WsMessage) => {
    if (data.type === 'metrics') {
      // Skip first WS update if we have SSR data (prevents hydration jump)
      if (skipFirstWsUpdate.current) {
        skipFirstWsUpdate.current = false
        return
      }
      const metrics = mapApiToMetrics(data.data as ApiMetricsJson)
      queryClient.setQueryData<ParsedMetrics>(METRICS_KEY, metrics)
    }
  }, [queryClient])

  // WebSocket connection - delay to ensure hydration completes first
  useEffect(() => {
    if (!useWebSocket) return

    const connect = () => {
      const ws = new WebSocket(wsUrl)
      wsRef.current = ws

      ws.onopen = () => {
        console.log('WebSocket connected')
      }

      ws.onmessage = (event) => {
        try {
          const data: WsMessage = JSON.parse(event.data)
          handleWsMessage(data)
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e)
        }
      }

      ws.onclose = () => {
        console.log('WebSocket disconnected, reconnecting in 5s...')
        reconnectTimeoutRef.current = setTimeout(connect, 5000)
      }

      ws.onerror = (error) => {
        console.error('WebSocket error:', error)
        ws.close()
      }
    }

    // Longer delay to let hydration complete and prevent immediate data jump
    const initTimeout = setTimeout(connect, 1000)

    return () => {
      clearTimeout(initTimeout)
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [wsUrl, useWebSocket, handleWsMessage])

  return {
    metrics: query.data,
    isLoading: query.isLoading,
    error: query.error,
    refetch: query.refetch,
    isRefetching: query.isRefetching,
  }
}
