'use client'

import { formatMb, formatMs } from '@/lib/formatters'
import { CacheInfo } from './CacheInfo'
import { UptimeDisplay } from '@/components/Common/UptimeDisplay'
import { TimeDisplay } from '@/components/Common/TimeDisplay'
import type { ApiStatusResponse, ParsedMetrics } from '@/types'

interface FooterProps {
  serverStatus: ApiStatusResponse | null
  metrics: ParsedMetrics | null
  lastUpdate: Date | null
  locale: string
}

export function Footer({ serverStatus, metrics, lastUpdate, locale }: FooterProps) {

  return (
    <footer>
      <div className="footer-inner">
        <span className="footer-left">
          SpendPulse v{serverStatus?.version || '?'}
          {serverStatus && (
            <span className="footer-server">
              • PID {serverStatus.process.pid}
              • {formatMb(serverStatus.process.memory_mb, locale)}
              • <UptimeDisplay initialSeconds={serverStatus.uptime_seconds} />
            </span>
          )}
          <CacheInfo serverStatus={serverStatus} />
        </span>
        <span className="footer-right">
          {serverStatus && (
            <span className="footer-api">
              Railway:{' '}
              {serverStatus.api.last_success
                ? <TimeDisplay timestamp={serverStatus.api.last_success} locale={locale} />
                : '—'}
              {metrics && (
                <span className="footer-latency">
                  {' '}
                  ({formatMs(metrics.scrapeDuration * 1000, locale)})
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
              • UI: {lastUpdate.toLocaleTimeString(locale)}
            </span>
          )}
        </span>
      </div>
    </footer>
  )
}
