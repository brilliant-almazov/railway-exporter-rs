'use client'

import { formatUptime, formatMb, formatMs, formatBytes } from '@/lib/formatters'
import type { ApiStatusResponse, ParsedMetrics } from '@/types'

interface FooterProps {
  serverStatus: ApiStatusResponse | null
  metrics: ParsedMetrics | null
  uptime: number
  lastUpdate: Date | null
}

export function Footer({ serverStatus, metrics, uptime, lastUpdate }: FooterProps) {
  const cacheConfig = serverStatus?.config.icon_cache
  const cacheStats = serverStatus?.icon_cache
  const cacheEnabled = cacheConfig?.enabled ?? false

  // Build tooltip content for enabled cache
  const cacheTooltip = cacheEnabled && cacheStats ? [
    `Count: ${cacheStats.count}/${cacheConfig?.max_count ?? '?'}`,
    `Size: ${formatBytes(cacheStats.total_bytes)}`,
    `Avg: ${formatBytes(cacheStats.avg_bytes)}`,
    `Min: ${formatBytes(cacheStats.min_bytes)}`,
    `Max: ${formatBytes(cacheStats.max_bytes)}`,
  ].join('\n') : undefined

  return (
    <footer>
      <div className="footer-inner">
        <span className="footer-left">
          SpendPulse v{serverStatus?.version || '?'}
          {serverStatus && (
            <span className="footer-server">
              • PID {serverStatus.process.pid}
              • {formatMb(serverStatus.process.memory_mb)}
              • {formatUptime(uptime)}
            </span>
          )}
          {cacheConfig && (
            <span className="footer-server" title={cacheTooltip}>
              • Cache: {cacheEnabled ? 'on' : 'off'}
            </span>
          )}
        </span>
        <span className="footer-right">
          {serverStatus && (
            <span className="footer-api">
              Railway:{' '}
              {serverStatus.api.last_success
                ? new Date(serverStatus.api.last_success * 1000).toLocaleTimeString()
                : '—'}
              {metrics && (
                <span className="footer-latency">
                  {' '}
                  ({formatMs(metrics.scrapeDuration * 1000)})
                </span>
              )}
              {serverStatus.api.failed_scrapes > 0 && (
                <span className="footer-errors">
                  {' '}
                  • {serverStatus.api.failed_scrapes} err
                </span>
              )}
            </span>
          )}
          {lastUpdate && (
            <span className="footer-update">
              • UI: {lastUpdate.toLocaleTimeString()}
            </span>
          )}
        </span>
      </div>
    </footer>
  )
}
