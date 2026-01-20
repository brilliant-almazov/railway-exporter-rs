'use client'

import { Tooltip } from '@/components/Common/Tooltip'
import { formatBytes } from '@/lib/formatters'
import type { ApiStatusResponse } from '@/types'

interface CacheInfoProps {
  serverStatus: ApiStatusResponse | null
}

/** Format TTL seconds to human-readable string */
const formatTtl = (seconds: number) => {
  if (seconds >= 86400) return `${Math.floor(seconds / 86400)}d`
  if (seconds >= 3600) return `${Math.floor(seconds / 3600)}h`
  if (seconds >= 60) return `${Math.floor(seconds / 60)}m`
  return `${seconds}s`
}

/** Tooltip content showing cache stats */
function CacheTooltip({ config, stats }: {
  config: ApiStatusResponse['config']['icon_cache']
  stats: ApiStatusResponse['icon_cache']
}) {
  if (!stats) return null

  const isLink = config.mode === 'link'

  return (
    <div style={{ textAlign: 'left', lineHeight: 1.5 }}>
      <div>Mode: {config.mode}</div>
      <div>Icons: {stats.count}/{config.max_count ?? '?'}</div>
      <div>Size: {formatBytes(stats.total_bytes)}</div>
      {isLink && <div>TTL: {formatTtl(config.max_age ?? 0)}</div>}
    </div>
  )
}

/** Cache info display in footer */
export function CacheInfo({ serverStatus }: CacheInfoProps) {
  const config = serverStatus?.config.icon_cache
  if (!config) return null

  // Cache disabled - simple text, no tooltip
  if (!config.enabled) {
    return <span className="footer-server">• Cache: off</span>
  }

  const tooltip = <CacheTooltip config={config} stats={serverStatus?.icon_cache} />

  return (
    <Tooltip content={tooltip}>
      <span className="footer-server">• Cache: on</span>
    </Tooltip>
  )
}
