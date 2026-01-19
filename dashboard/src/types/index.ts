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

// Totals for filtered services
export interface FilteredTotals {
  cost: number
  estimatedMonthly: number
  cpuMinutes: number
  memoryGbMinutes: number
  diskGbMinutes: number
  networkTxGb: number
  avgCpu: number
  avgMemory: number
  avgDisk: number
}

// Icon cache statistics from backend
export interface IconCacheStats {
  count: number
  total_bytes: number
  min_bytes: number
  max_bytes: number
  median_bytes: number
  avg_bytes: number
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
    prices: {
      cpu_per_vcpu_minute: number
      memory_per_gb_minute: number
      disk_per_gb_minute: number
      network_egress_per_gb: number
    }
    gzip: {
      enabled: boolean
      min_size: number
      level: number
    }
    icon_cache: {
      enabled: boolean
      mode: 'base64' | 'link'
      max_count?: number  // Present in both modes
      max_age?: number    // Only in link mode
      base_url?: string   // Only in link mode
    }
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
  // Icon cache statistics (present when cache is enabled)
  icon_cache?: IconCacheStats
}
