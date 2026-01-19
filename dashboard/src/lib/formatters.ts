// Declarative number formatting configuration
// Change values here to adjust display precision across the app

export const FORMAT_CONFIG = {
  // Cost values (USD)
  cost: {
    decimals: 2,
    prefix: '$',
    suffix: '',
  },

  // Percentages
  percent: {
    decimals: 1,
    prefix: '',
    suffix: '%',
  },

  // CPU usage (vCPU hours or similar)
  cpu: {
    decimals: 4,
    prefix: '',
    suffix: '',
  },

  // Memory/Disk (GB)
  gb: {
    decimals: 2,
    prefix: '',
    suffix: ' GB',
  },

  // Network traffic (GB) - more precision
  network: {
    decimals: 4,
    prefix: '',
    suffix: ' GB',
  },

  // Process memory (MB)
  mb: {
    decimals: 2,
    prefix: '',
    suffix: ' MB',
  },

  // Duration in milliseconds
  ms: {
    decimals: 0,
    prefix: '',
    suffix: 'ms',
  },

  // Generic decimal
  decimal: {
    decimals: 2,
    prefix: '',
    suffix: '',
  },

  // Generic integer
  integer: {
    decimals: 0,
    prefix: '',
    suffix: '',
  },
} as const

type FormatKey = keyof typeof FORMAT_CONFIG

// Format a number according to config
export function formatByType(value: number, type: FormatKey, locale = 'en-US'): string {
  const config = FORMAT_CONFIG[type]
  const formatted = value.toLocaleString(locale, {
    minimumFractionDigits: config.decimals,
    maximumFractionDigits: config.decimals,
  })
  return `${config.prefix}${formatted}${config.suffix}`
}

// Convenience functions
export function formatCurrency(value: number, locale = 'en-US'): string {
  return formatByType(value, 'cost', locale)
}

export function formatCpu(value: number, locale = 'en-US'): string {
  return formatByType(value, 'cpu', locale)
}

export function formatGb(value: number, locale = 'en-US'): string {
  return formatByType(value, 'gb', locale)
}

export function formatNetwork(value: number, locale = 'en-US'): string {
  return formatByType(value, 'network', locale)
}

export function formatMb(value: number, locale = 'en-US'): string {
  return formatByType(value, 'mb', locale)
}

export function formatMs(value: number, locale = 'en-US'): string {
  return formatByType(value, 'ms', locale)
}

export function formatPercent(value: number, locale = 'en-US'): string {
  return formatByType(value, 'percent', locale)
}

/**
 * Format number with smart "less than" display for tiny values
 * If value > 0 but rounds to 0, shows "< 0" instead
 */
export function formatNumber(value: number, decimals = 2): string {
  if (value === 0) return '0'

  const threshold = Math.pow(10, -decimals)
  if (value > 0 && value < threshold) {
    return '< 0'
  }

  return value.toFixed(decimals)
}

/**
 * Format number or show dash for zero/empty values
 * Used for metrics where 0 means "not applicable" (e.g., disk when service has no disk)
 * Also shows "< 0" for tiny non-zero values
 */
export function formatOrDash(value: number, decimals = 2): string {
  if (!value || value === 0) return '—'

  const threshold = Math.pow(10, -decimals)
  if (value > 0 && value < threshold) {
    return '< 0'
  }

  return value.toFixed(decimals)
}

export function formatInteger(value: number, locale = 'en-US'): string {
  return Math.floor(value).toLocaleString(locale)
}

/**
 * Format raw value for tooltip display (high precision)
 * Shows full precision with prefix/suffix
 */
export function formatRaw(value: number, prefix = '', suffix = '', precision = 6): string {
  if (value === 0) return `${prefix}0${suffix}`
  return `${prefix}${value.toPrecision(precision)}${suffix}`
}

// Time formatting functions

/**
 * Format seconds as human-readable uptime string
 */
export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${mins}m`
  return `${mins}m`
}

/**
 * Format seconds as interval string (e.g., "5s", "1m", "1h")
 */
export function formatInterval(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
  return `${Math.floor(seconds / 3600)}h`
}

/**
 * Format date as relative time (e.g., "2m ago", "1h ago")
 */
export function formatRelativeTime(date: Date): string {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000)
  if (seconds < 60) return `${seconds}s ago`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  return `${Math.floor(seconds / 3600)}h ago`
}

/**
 * Format bytes as human-readable size (e.g., "1.5 KB", "2.3 MB", "1.2 GB", "0.5 TB")
 */
export function formatBytes(bytes: number): string {
  const KB = 1024
  const MB = KB * 1024
  const GB = MB * 1024
  const TB = GB * 1024

  if (bytes < KB) return `${bytes} B`
  if (bytes < MB) return `${(bytes / KB).toFixed(1)} KB`
  if (bytes < GB) return `${(bytes / MB).toFixed(1)} MB`
  if (bytes < TB) return `${(bytes / GB).toFixed(2)} GB`
  return `${(bytes / TB).toFixed(2)} TB`
}
