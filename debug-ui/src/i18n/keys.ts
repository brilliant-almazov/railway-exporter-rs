// Translation keys as constants for type safety
export const T = {
  // Header
  REFRESH: 'refresh',
  SHOW_RAW: 'showRaw',
  HIDE_RAW: 'hideRaw',
  SETTINGS: 'settings',
  USE_WEBSOCKET: 'useWebSocket',
  LAST_UPDATE: 'lastUpdate',

  // Stats
  CURRENT_SPEND: 'currentSpend',
  ESTIMATED_MONTHLY: 'estimatedMonthly',
  DAILY_AVERAGE: 'dailyAverage',
  SERVICES: 'services',
  PROJECTED_TOTAL: 'projectedTotal',
  PER_DAY_COST: 'perDayCost',
  ACTIVE: 'active',

  // Billing
  BILLING_PERIOD: 'billingPeriod',
  DAYS_ELAPSED: 'daysElapsed',
  DAYS_REMAINING: 'daysRemaining',
  TOTAL_DAYS: 'totalDays',
  MINUTES_ELAPSED: 'minutesElapsed',

  // Filters
  FILTER_BY_SERVICE: 'filterByService',
  ALL_GROUPS: 'allGroups',
  SHOW_DELETED: 'showDeleted',
  CLEAR: 'clear',

  // Table headers
  SERVICE: 'service',
  GROUP: 'group',
  COST: 'cost',
  FORECAST: 'forecast',
  CPU_MIN: 'cpuMin',
  AVG_VCPU: 'avgVcpu',
  RAM_GB_MIN: 'ramGbMin',
  AVG_RAM: 'avgRam',
  DISK_GB_MIN: 'diskGbMin',
  AVG_DISK: 'avgDisk',
  TX_GB: 'txGb',
  RX_GB: 'rxGb',
  TOTAL: 'total',

  // Deleted
  DELETED_SERVICES: 'deletedServices',

  // Footer
  AUTO_REFRESH: 'autoRefresh',

  // Raw
  RAW_METRICS: 'rawMetrics',

  // Legend
  LEGEND_TITLE: 'legendTitle',
} as const

export type TranslationKey = typeof T[keyof typeof T]

// Supported languages
export const LANGUAGES = ['en', 'ru', 'uk', 'kk', 'de', 'fr', 'es', 'pt', 'sv', 'no', 'zh', 'he'] as const
export type Language = typeof LANGUAGES[number]

export const LANGUAGE_FLAGS: Record<Language, string> = {
  en: '🇬🇧',
  ru: '🇷🇺',
  uk: '🇺🇦',
  he: '🇮🇱',
  kk: '🇰🇿',
  de: '🇩🇪',
  fr: '🇫🇷',
  es: '🇪🇸',
  pt: '🇵🇹',
  sv: '🇸🇪',
  no: '🇳🇴',
  zh: '🇨🇳',
}

// RTL languages
export const RTL_LANGUAGES: Language[] = ['he']

export const isRtl = (lang: Language): boolean => RTL_LANGUAGES.includes(lang)
