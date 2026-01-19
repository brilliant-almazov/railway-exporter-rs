import type { ApiMetricsJson, ParsedMetrics, ServiceMetrics } from '@/types'

/**
 * Map API response to internal metrics format
 * Note: isDeleted flag comes from the server (Rust backend detects deleted services by UUID pattern)
 */
export function mapApiToMetrics(api: ApiMetricsJson): ParsedMetrics {
  const daysInPeriod = api.project.days_elapsed
  const minutesInPeriod = daysInPeriod * 24 * 60

  const services: ServiceMetrics[] = api.services.map(svc => ({
    name: svc.name,
    icon: svc.icon,
    group: svc.group || 'ungrouped',
    cost: svc.cost_usd,
    estimatedMonthly: svc.estimated_monthly_usd,
    cpuMinutes: svc.cpu_usage,
    memoryGbMinutes: svc.memory_usage,
    diskGbMinutes: svc.disk_usage,
    networkTxGb: svc.network_tx,
    avgCpu: minutesInPeriod > 0 ? svc.cpu_usage / minutesInPeriod : 0,
    avgMemory: minutesInPeriod > 0 ? svc.memory_usage / minutesInPeriod : 0,
    avgDisk: minutesInPeriod > 0 ? svc.disk_usage / minutesInPeriod : 0,
    isDeleted: svc.isDeleted, // Server-side detection
  })).sort((a, b) => b.cost - a.cost)

  return {
    project: api.project.name,
    plan: 'Pro', // TODO: add to backend response
    daysInPeriod,
    daysRemaining: api.project.days_remaining,
    currentUsage: api.project.current_usage_usd,
    estimatedMonthly: api.project.estimated_monthly_usd,
    dailyAverage: api.project.daily_average_usd,
    services,
    scrapeSuccess: 1,
    scrapeDuration: api.scrape_duration_seconds,
  }
}

/**
 * Fetch metrics from API (JSON format)
 */
export async function fetchMetrics(url: string): Promise<ParsedMetrics> {
  const response = await fetch(url, {
    headers: { Accept: 'application/json' },
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`)
  }
  const data: ApiMetricsJson = await response.json()
  return mapApiToMetrics(data)
}
