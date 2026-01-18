// Service metrics data
export interface ServiceMetrics {
  name: string
  icon: string
  group: string
  cost: number
  estimatedMonthly: number
  cpuMinutes: number
  memoryGbMinutes: number
  diskGbMinutes: number
  networkTxGb: number
  networkRxGb: number
  avgCpu: number
  avgMemory: number
  avgDisk: number
  isDeleted: boolean
}

// Parsed metrics from API
export interface ParsedMetrics {
  project: string
  plan: string
  currentUsage: number
  estimatedMonthly: number
  dailyAverage: number
  daysInPeriod: number
  daysRemaining: number
  services: ServiceMetrics[]
  scrapeSuccess: number
  scrapeDuration: number
}

// API response types (from Rust backend)
export interface ApiProjectSummary {
  name: string
  current_usage_usd: number
  estimated_monthly_usd: number
  daily_average_usd: number
  days_elapsed: number
  days_remaining: number
}

export interface ApiServiceData {
  id: string
  name: string
  icon: string
  group: string
  cpu_usage: number
  memory_usage: number
  disk_usage: number
  network_tx: number
  network_rx: number
  cost_usd: number
  estimated_monthly_usd: number
  isDeleted: boolean
}

export interface ApiMetricsJson {
  project: ApiProjectSummary
  services: ApiServiceData[]
  scrape_timestamp: number
  scrape_duration_seconds: number
}

// WebSocket message types
export interface WsMessage {
  type: 'metrics' | 'status'
  data: ApiMetricsJson | WsStatus
}

export interface WsStatus {
  uptime_seconds: number
  api: {
    last_success: string | null
    last_error: string | null
    total_scrapes: number
    failed_scrapes: number
  }
  ws_clients: number
}

// Sort configuration
export type SortField = keyof ServiceMetrics
export type SortDirection = 'asc' | 'desc'

export interface SortConfig {
  field: SortField
  direction: SortDirection
}

// Filter configuration
export interface Filters {
  group: string
  search: string
  showDeleted: boolean
}

// Totals for filtered services
export interface FilteredTotals {
  cost: number
  estimatedMonthly: number
  cpuMinutes: number
  memoryGbMinutes: number
  diskGbMinutes: number
  networkTxGb: number
  networkRxGb: number
  avgCpu: number
  avgMemory: number
  avgDisk: number
}

// Server status response from /status endpoint
export interface ApiStatusResponse {
  version: string
  project_name: string
  uptime_seconds: number
  endpoints: {
    prometheus: boolean
    json: boolean
    websocket: boolean
    health: boolean
  }
  config: {
    plan: string
    scrape_interval_seconds: number
    api_url: string
    service_groups: string[]
  }
  process: {
    pid: number
    memory_mb: number
    cpu_percent: number
  }
  api: {
    last_success: number | null
    last_error: string | null
    total_scrapes: number
    failed_scrapes: number
  }
}
