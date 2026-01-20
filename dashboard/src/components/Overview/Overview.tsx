'use client'

import { StatCard } from './StatCard'
import { formatCurrency, formatNumber, formatInteger } from '@/lib/formatters'
import { colors } from '@/styles/colors'
import type { Translations, TextDirection } from '@/i18n/keys'
import type { ParsedMetrics, FilteredTotals } from '@/types'

interface OverviewProps {
  metrics: ParsedMetrics
  filteredTotals: FilteredTotals
  updated?: boolean
  isCompact?: boolean
  dir: TextDirection
  t: Translations
}

export function Overview({
  metrics,
  filteredTotals,
  updated = false,
  isCompact = false,
  dir,
  t
}: OverviewProps) {
  const className = isCompact ? 'overview compact' : 'overview'

  return (
    <section className={className} dir={dir}>
      <div className="stats-grid">
        <StatCard
          title={t.currentSpend}
          value={formatCurrency(metrics.currentUsage)}
          subtitle={`Day ${metrics.daysInPeriod} / ${metrics.daysInPeriod + metrics.daysRemaining}`}
          color={colors.success}
          updated={updated}
        />
        <StatCard
          title={t.estimatedMonthly}
          value={formatCurrency(metrics.estimatedMonthly)}
          subtitle={t.projectedTotal}
          color={colors.warning}
          updated={updated}
        />
        <StatCard
          title={t.dailyAverage}
          value={formatCurrency(metrics.dailyAverage)}
          subtitle={t.perDayCost}
          updated={updated}
        />
        <StatCard
          title={t.minutesElapsed}
          value={formatInteger(metrics.daysInPeriod * 24 * 60)}
          subtitle={`${metrics.services.length} ${t.services.toLowerCase()}`}
          color={colors.primary}
          updated={updated}
        />
      </div>
      <div className="stats-grid stats-grid-secondary">
        <StatCard
          title="CPU (min)"
          value={formatNumber(filteredTotals.cpuMinutes, 0)}
          subtitle={`Avg: ${formatNumber(filteredTotals.avgCpu, 2)} vCPU`}
          color={colors.cpu}
          updated={updated}
        />
        <StatCard
          title="RAM (GB·min)"
          value={formatNumber(filteredTotals.memoryGbMinutes, 0)}
          subtitle={`Avg: ${formatNumber(filteredTotals.avgMemory, 2)} GB`}
          color={colors.ram}
          updated={updated}
        />
        <StatCard
          title="Disk (GB·min)"
          value={formatNumber(filteredTotals.diskGbMinutes, 0)}
          subtitle={`Avg: ${formatNumber(filteredTotals.avgDisk, 2)} GB`}
          color={colors.disk}
          updated={updated}
        />
        <StatCard
          title="Network TX (GB)"
          value={formatNumber(filteredTotals.networkTxGb, 2)}
          subtitle="Egress traffic"
          color={colors.network}
          updated={updated}
        />
      </div>
    </section>
  )
}
