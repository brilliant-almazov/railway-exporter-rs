'use client'

import { useState, useEffect } from 'react'
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
  // Client-only rendering for lastUpdate to avoid hydration mismatch
  const [uiTime, setUiTime] = useState<string | null>(null)

  useEffect(() => {
    if (lastUpdate) {
      setUiTime(lastUpdate.toLocaleTimeString(locale))
    }
  }, [lastUpdate, locale])

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
            <span className="footer-api" style={{ minWidth: '10em', display: 'inline-block' }}>
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
          {uiTime && (
            <span className="footer-update">
              • UI: {uiTime}
            </span>
          )}
        </span>
      </div>
    </footer>
  )
}
