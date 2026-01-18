'use client'

import { formatUptime, formatMb, formatMs } from '@/lib/formatters'
import type { ApiStatusResponse, ParsedMetrics } from '@/types'

interface FooterProps {
  serverStatus: ApiStatusResponse | null
  metrics: ParsedMetrics | null
  uptime: number
  lastUpdate: Date | null
}

export function Footer({ serverStatus, metrics, uptime, lastUpdate }: FooterProps) {
  return (
    <footer>
      <div className="footer-inner">
        <span className="footer-left">
          SpendPulse v{serverStatus?.version || '?'}
          {serverStatus && (
            <span className="footer-server">
              • PID {serverStatus.process.pid}
              • {formatMb(serverStatus.process.memory_mb)}
              • ⏱ {formatUptime(uptime)}
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
