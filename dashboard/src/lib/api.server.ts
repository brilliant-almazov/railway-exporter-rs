import type { ApiMetricsJson, ApiStatusResponse, ParsedMetrics } from '@/types'
import { mapApiToMetrics } from './api'

/**
 * Server-side data fetching for SSR
 * These functions run on the server and return data for initial render
 */

export interface InitialData {
  metrics: ParsedMetrics | null
  serverStatus: ApiStatusResponse | null
  error: string | null
}

export async function fetchInitialData(apiHost: string): Promise<InitialData> {
  try {
    const [metricsRes, statusRes] = await Promise.all([
      fetch(`http://${apiHost}/metrics`, {
        headers: { Accept: 'application/json' },
        cache: 'no-store', // Don't cache on server
      }),
      fetch(`http://${apiHost}/status`, {
        cache: 'no-store',
      }),
    ])

    if (!metricsRes.ok || !statusRes.ok) {
      return {
        metrics: null,
        serverStatus: null,
        error: `API error: metrics=${metricsRes.status}, status=${statusRes.status}`,
      }
    }

    const [metricsJson, statusJson]: [ApiMetricsJson, ApiStatusResponse] = await Promise.all([
      metricsRes.json(),
      statusRes.json(),
    ])

    return {
      metrics: mapApiToMetrics(metricsJson),
      serverStatus: statusJson,
      error: null,
    }
  } catch (err) {
    // Server might not be running - return empty state
    return {
      metrics: null,
      serverStatus: null,
      error: err instanceof Error ? err.message : 'Failed to fetch',
    }
  }
}
