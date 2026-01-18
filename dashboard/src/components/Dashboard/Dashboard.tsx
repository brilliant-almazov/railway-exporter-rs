'use client'

import { useState, useMemo, useCallback } from 'react'
import { Header } from '../Header/Header'
import { Footer } from '../Footer/Footer'
import { Overview } from '../Overview/Overview'
import { Legend } from '../Legend/Legend'
import { ServicesTable } from '../ServicesTable/ServicesTable'
import { useMetrics } from '@/hooks/useMetrics'
import { useServerStatus } from '@/hooks/useServerStatus'
import { useLanguage } from '@/hooks/useLanguage'
import uiTranslations from '@/i18n/ui.json'
import type { FilteredTotals, ServiceMetrics } from '@/types'
import type { InitialData } from '@/lib/api.server'

interface DashboardProps {
  apiHost: string
  initialData: InitialData
}

// Calculate totals from services (used for SSR initial state)
// Must match ServicesTable calculation exactly!
function calculateTotals(services: ServiceMetrics[]): FilteredTotals {
  const activeServices = services.filter(s => !s.isDeleted)

  return {
    cost: activeServices.reduce((a, s) => a + s.cost, 0),
    estimatedMonthly: activeServices.reduce((a, s) => a + s.estimatedMonthly, 0),
    cpuMinutes: activeServices.reduce((a, s) => a + s.cpuMinutes, 0),
    avgCpu: activeServices.reduce((a, s) => a + s.avgCpu, 0),
    memoryGbMinutes: activeServices.reduce((a, s) => a + s.memoryGbMinutes, 0),
    avgMemory: activeServices.reduce((a, s) => a + s.avgMemory, 0),
    diskGbMinutes: activeServices.reduce((a, s) => a + s.diskGbMinutes, 0),
    avgDisk: activeServices.reduce((a, s) => a + s.avgDisk, 0),
    networkTxGb: activeServices.reduce((a, s) => a + s.networkTxGb, 0),
    networkRxGb: activeServices.reduce((a, s) => a + s.networkRxGb, 0),
  }
}

const defaultTotals: FilteredTotals = {
  cost: 0,
  estimatedMonthly: 0,
  cpuMinutes: 0,
  avgCpu: 0,
  memoryGbMinutes: 0,
  avgMemory: 0,
  diskGbMinutes: 0,
  avgDisk: 0,
  networkTxGb: 0,
  networkRxGb: 0,
}

export function Dashboard({ apiHost, initialData }: DashboardProps) {
  const { language, setLanguage } = useLanguage()
  const [useWebSocket, setUseWebSocket] = useState(false)
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null)
  const [justUpdated, setJustUpdated] = useState(false)
  // Initialize with SSR data to prevent layout shift
  const [filteredTotals, setFilteredTotals] = useState<FilteredTotals>(
    () => initialData.metrics ? calculateTotals(initialData.metrics.services) : defaultTotals
  )

  const { serverStatus, uptime } = useServerStatus({
    apiHost,
    initialData: initialData.serverStatus,
  })

  const { metrics, isLoading, error, refetch, isRefetching } = useMetrics({
    apiHost,
    useWebSocket,
    pollingInterval: (serverStatus?.config.scrape_interval_seconds || 5) * 1000,
    initialData: initialData.metrics,
  })

  // Get translations for current language
  const t = uiTranslations[language] || uiTranslations.en

  // Get groups from server status
  const groups = useMemo(() => {
    if (serverStatus?.config.service_groups) {
      return [...serverStatus.config.service_groups].sort()
    }
    if (!metrics) return []
    const activeServices = metrics.services.filter(s => !s.isDeleted)
    return [...new Set(activeServices.map(s => s.group))].sort()
  }, [serverStatus, metrics])

  // Handle refresh
  const handleRefresh = useCallback(async () => {
    await refetch()
    setLastUpdate(new Date())
    setJustUpdated(true)
    setTimeout(() => setJustUpdated(false), 700)
  }, [refetch])

  // Handle WebSocket toggle
  const handleWebSocketToggle = useCallback(() => {
    setUseWebSocket(prev => !prev)
  }, [])

  // Handle show raw metrics
  const handleShowRaw = useCallback(() => {
    window.open(`http://${apiHost}/metrics`, '_blank')
  }, [apiHost])

  // Handle totals change from ServicesTable
  const handleTotalsChange = useCallback((totals: FilteredTotals) => {
    setFilteredTotals(totals)
  }, [])

  return (
    <div className="app">
      {isRefetching && <div className="loading-bar" />}
      <Header
        serverStatus={serverStatus}
        language={language}
        onLanguageChange={setLanguage}
        useWebSocket={useWebSocket}
        onWebSocketToggle={handleWebSocketToggle}
        onRefresh={handleRefresh}
        onShowRaw={handleShowRaw}
        translations={{
          refresh: t.refresh,
          showRaw: t.showRaw,
          wsRealtime: t.wsRealtime,
          pollInterval: t.pollInterval,
        }}
      />

      {(error || initialData.error) && (
        <div className="error-banner">{String(error || initialData.error)}</div>
      )}
      {isLoading && !initialData.metrics && (
        <div className="loading">Loading metrics...</div>
      )}

      {metrics && (
        <main>
          <Overview
            metrics={metrics}
            filteredTotals={filteredTotals}
            updated={justUpdated}
            translations={{
              currentSpend: t.currentSpend,
              estimatedMonthly: t.estimatedMonthly,
              dailyAverage: t.dailyAverage,
              minutesElapsed: t.minutesElapsed,
              projectedTotal: t.projectedTotal,
              perDayCost: t.perDayCost,
              services: t.services,
            }}
          />

          <Legend language={language} />

          <ServicesTable
            services={metrics.services}
            groups={groups}
            language={language}
            onTotalsChange={handleTotalsChange}
            translations={{
              services: t.services,
              service: t.service,
              group: t.group,
              cost: t.cost,
              forecast: t.forecast,
              cpuMin: t.cpuMin,
              avgVcpu: t.avgVcpu,
              ramGbMin: t.ramGbMin,
              avgRam: t.avgRam,
              diskGbMin: t.diskGbMin,
              avgDisk: t.avgDisk,
              txGb: t.txGb,
              rxGb: t.rxGb,
              total: t.total,
              filterByService: t.filterByService,
              allGroups: t.allGroups,
              showDeleted: t.showDeleted,
              clear: t.clear,
            }}
          />
        </main>
      )}

      <Footer
        serverStatus={serverStatus}
        metrics={metrics ?? null}
        uptime={uptime}
        lastUpdate={lastUpdate}
      />
    </div>
  )
}
