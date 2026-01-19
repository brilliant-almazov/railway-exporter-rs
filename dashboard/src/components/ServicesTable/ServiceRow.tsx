'use client'

import { memo } from 'react'
import { ServiceIcon } from '../common/ServiceIcon'
import { formatCurrency, formatNumber, formatOrDash } from '@/lib/formatters'
import type { ServiceMetrics } from '@/types'

interface ServiceRowProps {
  service: ServiceMetrics
}

function ServiceRowComponent({ service: svc }: ServiceRowProps) {
  return (
    <tr className={svc.isDeleted ? 'deleted-row' : ''}>
      <td className="service-cell" title={svc.name}>
        <ServiceIcon url={svc.icon} name={svc.name} />
        <span className="name">
          {svc.isDeleted ? `${svc.name.slice(0, 8)}...` : svc.name}
        </span>
      </td>
      <td title={svc.isDeleted ? 'Deleted' : svc.group}>
        {svc.isDeleted ? '—' : <span className="group-badge">{svc.group}</span>}
      </td>
      <td className="right money" title={String(svc.cost)}>
        {formatCurrency(svc.cost)}
      </td>
      <td className="right money" title={String(svc.estimatedMonthly)}>
        {formatCurrency(svc.estimatedMonthly)}
      </td>
      <td className="right" title={String(svc.cpuMinutes)}>
        {formatNumber(svc.cpuMinutes, 0)}
      </td>
      <td className="right highlight" title={String(svc.avgCpu)}>
        {formatNumber(svc.avgCpu, 4)}
      </td>
      <td className="right" title={String(svc.memoryGbMinutes)}>
        {formatNumber(svc.memoryGbMinutes, 0)}
      </td>
      <td className="right highlight" title={String(svc.avgMemory)}>
        {formatNumber(svc.avgMemory, 4)}
      </td>
      <td className="right" title={String(svc.diskGbMinutes)}>
        {formatOrDash(svc.diskGbMinutes, 0)}
      </td>
      <td className="right highlight" title={String(svc.avgDisk)}>
        {formatOrDash(svc.avgDisk, 4)}
      </td>
      <td className="right" title={String(svc.networkTxGb)}>
        {formatNumber(svc.networkTxGb, 4)}
      </td>
    </tr>
  )
}

// Memoize to prevent unnecessary re-renders
export const ServiceRow = memo(ServiceRowComponent)
