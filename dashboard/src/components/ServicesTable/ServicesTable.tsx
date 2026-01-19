'use client'

import { useMemo, useState, useCallback, useEffect } from 'react'
import { CustomSelect } from '../Filters/CustomSelect'
import { SortIndicator } from './SortIndicator'
import { ServiceRow } from './ServiceRow'
import { useUrlFilters } from '@/hooks/useUrlFilters'
import { useDirection } from '@/hooks/useDirection'
import { formatCurrency, formatNumber } from '@/lib/formatters'
import type { ServiceMetrics, FilteredTotals } from '@/types'

interface ServicesTableProps {
  services: ServiceMetrics[]
  groups: string[]
  language: string
  translations: {
    services: string
    service: string
    group: string
    cost: string
    forecast: string
    cpuMin: string
    avgVcpu: string
    ramGbMin: string
    avgRam: string
    diskGbMin: string
    avgDisk: string
    txGb: string
    total: string
    filterByService: string
    allGroups: string
    showDeleted: string
    clear: string
  }
  onTotalsChange?: (totals: FilteredTotals) => void
}

export function ServicesTable({
  services,
  groups,
  language,
  translations: t,
  onTotalsChange
}: ServicesTableProps) {
  const dir = useDirection()
  // URL-synced filters (snake_case params: ?search=x&group=y&show_deleted=true)
  const {
    search: filterService,
    group: filterGroup,
    showDeleted,
    hasActiveFilters,
    setSearch: setFilterService,
    setGroup: setFilterGroup,
    setShowDeleted,
    clearFilters
  } = useUrlFilters()

  const [sortColumn, setSortColumn] = useState<keyof ServiceMetrics>('cost')
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc')

  // Deleted services stats
  const deletedStats = useMemo(() => {
    const deleted = services.filter(s => s.isDeleted)
    return {
      count: deleted.length,
      cost: deleted.reduce((a, s) => a + s.cost, 0)
    }
  }, [services])

  // Sort handler
  const handleSort = useCallback((column: keyof ServiceMetrics) => {
    if (sortColumn === column) {
      setSortDirection(d => d === 'asc' ? 'desc' : 'asc')
    } else {
      setSortColumn(column)
      setSortDirection('desc')
    }
  }, [sortColumn])

  // Filter and sort services
  const filteredServices = useMemo(() => {
    const filtered = services.filter(svc => {
      if (svc.isDeleted && !showDeleted) return false
      const matchesService = !filterService || svc.name.toLowerCase().includes(filterService.toLowerCase())
      const matchesGroup = !filterGroup || svc.group === filterGroup
      return matchesService && matchesGroup
    })

    return filtered.sort((a, b) => {
      const aVal = a[sortColumn]
      const bVal = b[sortColumn]
      if (typeof aVal === 'string' && typeof bVal === 'string') {
        return sortDirection === 'asc' ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal)
      }
      return sortDirection === 'asc'
        ? (aVal as number) - (bVal as number)
        : (bVal as number) - (aVal as number)
    })
  }, [services, filterService, filterGroup, showDeleted, sortColumn, sortDirection])

  // Calculate totals for filtered services
  const filteredTotals = useMemo(() => ({
    cost: filteredServices.reduce((a, s) => a + s.cost, 0),
    estimatedMonthly: filteredServices.reduce((a, s) => a + s.estimatedMonthly, 0),
    cpuMinutes: filteredServices.reduce((a, s) => a + s.cpuMinutes, 0),
    avgCpu: filteredServices.reduce((a, s) => a + s.avgCpu, 0),
    memoryGbMinutes: filteredServices.reduce((a, s) => a + s.memoryGbMinutes, 0),
    avgMemory: filteredServices.reduce((a, s) => a + s.avgMemory, 0),
    diskGbMinutes: filteredServices.reduce((a, s) => a + s.diskGbMinutes, 0),
    avgDisk: filteredServices.reduce((a, s) => a + s.avgDisk, 0),
    networkTxGb: filteredServices.reduce((a, s) => a + s.networkTxGb, 0),
  }), [filteredServices])

  // Notify parent of totals change (in effect to avoid setState during render)
  useEffect(() => {
    onTotalsChange?.(filteredTotals)
  }, [filteredTotals, onTotalsChange])

  const activeCount = services.filter(s => !s.isDeleted).length
  const filteredActiveCount = filteredServices.filter(s => !s.isDeleted).length
  const filteredDeletedCount = filteredServices.filter(s => s.isDeleted).length
  const hasDeleted = services.some(s => s.isDeleted)

  return (
    <div className="services-wrapper">
      <section
        className="services-section"
        dir={dir}
      >
        <div className="services-header">
          <h3>
            {t.services}{' '}
            <span dir="ltr">
              ({filteredActiveCount}/{activeCount}
              {showDeleted && hasDeleted && (
                <span className="deleted-count"> +{filteredDeletedCount}</span>
              )}
              )
            </span>
          </h3>
          <div className="filters-row">
            {deletedStats.count > 0 && (
              <div className="deleted-block">
                {deletedStats.count} deleted • <span className="cost">{formatCurrency(deletedStats.cost)}</span>
              </div>
            )}
            <div className="filters">
              <div className="filter-group">
                <label>{t.service}:</label>
                <input
                  type="text"
                  placeholder={t.filterByService}
                  value={filterService}
                  onChange={(e) => setFilterService(e.target.value || null)}
                  className="filter-input"
                />
              </div>
              <div className="filter-divider" />
              <div className="filter-group">
                <label>{t.group}:</label>
                <CustomSelect
                  value={filterGroup}
                  onChange={(v) => setFilterGroup(v || null)}
                  options={groups.map(g => ({ value: g, label: g }))}
                  placeholder={t.allGroups}
                />
              </div>
              <div className="filter-divider" />
              <label className={`filter-toggle ${showDeleted ? 'active' : ''}`}>
                <input
                  type="checkbox"
                  checked={showDeleted}
                  onChange={(e) => setShowDeleted(e.target.checked || null)}
                />
                {t.showDeleted}
              </label>
            </div>
          </div>
        </div>
        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th className="sortable" onClick={() => handleSort('name')}>
                  {t.service} <SortIndicator column="name" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="sortable" onClick={() => handleSort('group')}>
                  {t.group} <SortIndicator column="group" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right sortable" onClick={() => handleSort('cost')}>
                  {t.cost} <SortIndicator column="cost" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right sortable" onClick={() => handleSort('estimatedMonthly')}>
                  {t.forecast} <SortIndicator column="estimatedMonthly" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right sortable" onClick={() => handleSort('cpuMinutes')}>
                  {t.cpuMin} <SortIndicator column="cpuMinutes" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right highlight-header sortable" onClick={() => handleSort('avgCpu')}>
                  {t.avgVcpu} <SortIndicator column="avgCpu" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right sortable" onClick={() => handleSort('memoryGbMinutes')}>
                  {t.ramGbMin} <SortIndicator column="memoryGbMinutes" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right highlight-header sortable" onClick={() => handleSort('avgMemory')}>
                  {t.avgRam} <SortIndicator column="avgMemory" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right sortable" onClick={() => handleSort('diskGbMinutes')}>
                  {t.diskGbMin} <SortIndicator column="diskGbMinutes" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right highlight-header sortable" onClick={() => handleSort('avgDisk')}>
                  {t.avgDisk} <SortIndicator column="avgDisk" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
                <th className="right sortable" onClick={() => handleSort('networkTxGb')}>
                  {t.txGb} <SortIndicator column="networkTxGb" sortColumn={sortColumn} sortDirection={sortDirection} />
                </th>
              </tr>
            </thead>
            <tbody>
              {filteredServices.length === 0 ? (
                <tr>
                  <td colSpan={11} className="no-results">
                    🔍 {language === 'ru' ? 'Ничего не найдено' : language === 'uk' ? 'Нічого не знайдено' : 'No results found'}
                  </td>
                </tr>
              ) : (
                filteredServices.map((svc) => (
                  <ServiceRow key={svc.name} service={svc} />
                ))
              )}
            </tbody>
            <tfoot>
              <tr className="totals-row">
                <td colSpan={2}><strong>{t.total} ({filteredServices.length})</strong></td>
                <td className="right money"><strong>{formatCurrency(filteredTotals.cost)}</strong></td>
                <td className="right money"><strong>{formatCurrency(filteredTotals.estimatedMonthly)}</strong></td>
                <td className="right"><strong>{formatNumber(filteredTotals.cpuMinutes, 0)}</strong></td>
                <td className="right highlight"><strong>{formatNumber(filteredTotals.avgCpu, 4)}</strong></td>
                <td className="right"><strong>{formatNumber(filteredTotals.memoryGbMinutes, 0)}</strong></td>
                <td className="right highlight"><strong>{formatNumber(filteredTotals.avgMemory, 4)}</strong></td>
                <td className="right"><strong>{formatNumber(filteredTotals.diskGbMinutes, 0)}</strong></td>
                <td className="right highlight"><strong>{formatNumber(filteredTotals.avgDisk, 4)}</strong></td>
                <td className="right"><strong>{formatNumber(filteredTotals.networkTxGb, 4)}</strong></td>
              </tr>
            </tfoot>
          </table>
        </div>
      </section>
      <button
        onClick={clearFilters}
        className={`clear-btn-side ${hasActiveFilters ? 'visible' : ''}`}
        title={t.clear}
      >
        ✕
      </button>
    </div>
  )
}
