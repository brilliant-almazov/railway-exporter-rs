/**
 * Calculate stable column widths based on max translation length across all languages
 * This ensures columns don't jump when switching languages
 */

import uiTranslations from '@/i18n/ui.json'

// Column header keys in order
const COLUMN_KEYS = [
  'service',
  'group',
  'cost',
  'forecast',
  'cpuMin',
  'avgVcpu',
  'ramGbMin',
  'avgRam',
  'diskGbMin',
  'avgDisk',
  'txGb',
] as const

type ColumnKey = typeof COLUMN_KEYS[number]

// Get character length (Chinese/Japanese characters count as 2)
function getVisualLength(str: string): number {
  let len = 0
  for (const char of str) {
    // CJK characters are wider
    if (char.charCodeAt(0) > 0x2E80) {
      len += 2
    } else {
      len += 1
    }
  }
  return len
}

// Calculate max visual length for each column across all languages
function calculateMaxLengths(): Record<ColumnKey, number> {
  const languages = Object.keys(uiTranslations) as (keyof typeof uiTranslations)[]
  const result = {} as Record<ColumnKey, number>

  for (const key of COLUMN_KEYS) {
    let maxLen = 0
    for (const lang of languages) {
      const text = uiTranslations[lang][key] || ''
      const len = getVisualLength(text)
      if (len > maxLen) maxLen = len
    }
    result[key] = maxLen
  }

  return result
}

// Convert visual length to pixel width (approx 8px per character + padding)
function lengthToWidth(len: number): number {
  const charWidth = 8
  const padding = 24 // sort indicator + padding
  return len * charWidth + padding
}

// Pre-calculated column widths
const MAX_LENGTHS = calculateMaxLengths()

export const COLUMN_WIDTHS: Record<ColumnKey, number> = {
  service: 0, // Auto (takes remaining space)
  group: lengthToWidth(MAX_LENGTHS.group),
  cost: lengthToWidth(MAX_LENGTHS.cost),
  forecast: lengthToWidth(MAX_LENGTHS.forecast),
  cpuMin: lengthToWidth(MAX_LENGTHS.cpuMin),
  avgVcpu: lengthToWidth(MAX_LENGTHS.avgVcpu),
  ramGbMin: lengthToWidth(MAX_LENGTHS.ramGbMin),
  avgRam: lengthToWidth(MAX_LENGTHS.avgRam),
  diskGbMin: lengthToWidth(MAX_LENGTHS.diskGbMin),
  avgDisk: lengthToWidth(MAX_LENGTHS.avgDisk),
  txGb: lengthToWidth(MAX_LENGTHS.txGb),
}

// Get column widths as CSS style object for colgroup
export function getColumnStyles(): Record<ColumnKey, React.CSSProperties> {
  return {
    service: { width: 'auto' },
    group: { width: `${COLUMN_WIDTHS.group}px` },
    cost: { width: `${COLUMN_WIDTHS.cost}px` },
    forecast: { width: `${COLUMN_WIDTHS.forecast}px` },
    cpuMin: { width: `${COLUMN_WIDTHS.cpuMin}px` },
    avgVcpu: { width: `${COLUMN_WIDTHS.avgVcpu}px` },
    ramGbMin: { width: `${COLUMN_WIDTHS.ramGbMin}px` },
    avgRam: { width: `${COLUMN_WIDTHS.avgRam}px` },
    diskGbMin: { width: `${COLUMN_WIDTHS.diskGbMin}px` },
    avgDisk: { width: `${COLUMN_WIDTHS.avgDisk}px` },
    txGb: { width: `${COLUMN_WIDTHS.txGb}px` },
  }
}

// Debug: log calculated widths
export function debugColumnWidths(): void {
  console.log('Column max lengths:', MAX_LENGTHS)
  console.log('Column widths (px):', COLUMN_WIDTHS)
}
