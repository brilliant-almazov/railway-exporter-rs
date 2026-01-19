'use client'

import { Tooltip } from '@/components/Common/Tooltip'
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
  const cacheTooltipContent = cacheEnabled && cacheStats ? (
    <div style={{ textAlign: 'left', lineHeight: 1.5 }}>
      <div>Count: {cacheStats.count}/{cacheConfig?.max_count ?? '?'}</div>
      <div>Size: {formatBytes(cacheStats.total_bytes)}</div>
      <div>Avg: {formatBytes(cacheStats.avg_bytes)}</div>
      <div>Min: {formatBytes(cacheStats.min_bytes)}</div>
      <div>Max: {formatBytes(cacheStats.max_bytes)}</div>
    </div>
  ) : null

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
            cacheTooltipContent ? (
              <Tooltip content={cacheTooltipContent}>
                <span className="footer-server">
                  • Cache: on
                </span>
              </Tooltip>
            ) : (
              <span className="footer-server">
                • Cache: off
              </span>
            )
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
