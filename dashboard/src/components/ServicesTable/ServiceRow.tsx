'use client'

import { memo } from 'react'
import { ServiceIcon } from '@/components/Common/ServiceIcon'
import { Tooltip } from '@/components/Common/Tooltip'
import { formatCurrency, formatNumber, formatOrDash, formatRaw } from '@/lib/formatters'
import type { ServiceMetrics } from '@/types'

interface ServiceRowProps {
  service: ServiceMetrics
}

function ServiceRowComponent({ service: svc }: ServiceRowProps) {
  return (
    <tr className={svc.isDeleted ? 'deleted-row' : ''}>
      <td>
        <div className="service-cell">
          <ServiceIcon url={svc.icon} name={svc.name} />
          <span className="name">
            {svc.isDeleted ? `${svc.name.slice(0, 8)}...` : svc.name}
          </span>
        </div>
      </td>
      <td>
        {svc.isDeleted ? '—' : <span className="group-badge">{svc.group}</span>}
      </td>
      <td className="right money">
        <Tooltip position="right" content={formatRaw(svc.cost, '$')}>
          {formatCurrency(svc.cost)}
        </Tooltip>
      </td>
      <td className="right money">
        <Tooltip position="right" content={formatRaw(svc.estimatedMonthly, '$')}>
          {formatCurrency(svc.estimatedMonthly)}
        </Tooltip>
      </td>
      <td className="right">
        <Tooltip position="right" content={svc.cpuMinutes}>
          {formatNumber(svc.cpuMinutes, 0)}
        </Tooltip>
      </td>
      <td className="right highlight">
        <Tooltip position="right" content={svc.avgCpu}>
          {formatNumber(svc.avgCpu, 4)}
        </Tooltip>
      </td>
      <td className="right">
        <Tooltip position="right" content={svc.memoryGbMinutes}>
          {formatNumber(svc.memoryGbMinutes, 0)}
        </Tooltip>
      </td>
      <td className="right highlight">
        <Tooltip position="right" content={svc.avgMemory}>
          {formatNumber(svc.avgMemory, 4)}
        </Tooltip>
      </td>
      <td className="right">
        <Tooltip position="right" content={svc.diskGbMinutes}>
          {formatOrDash(svc.diskGbMinutes, 0)}
        </Tooltip>
      </td>
      <td className="right highlight">
        <Tooltip position="right" content={svc.avgDisk}>
          {formatOrDash(svc.avgDisk, 4)}
        </Tooltip>
      </td>
      <td className="right">
        <Tooltip position="right" content={svc.networkTxGb}>
          {formatOrDash(svc.networkTxGb, 4)}
        </Tooltip>
      </td>
    </tr>
  )
}

// Memoize to prevent unnecessary re-renders
export const ServiceRow = memo(ServiceRowComponent)
