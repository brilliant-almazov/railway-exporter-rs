import {useEffect, useState, useCallback, useMemo, useRef} from 'react'
import './App.css'
import uiTranslations from './i18n/ui.json'
import {LANGUAGES, LANGUAGE_FLAGS, type Language} from './i18n/keys'
import {colors} from './colors'

interface ServiceMetrics {
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

// UUID pattern to detect deleted services (Railway uses UUID as name for deleted services)
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

function ServiceIcon({url, name}: { url: string; name: string }) {
  if (!url || url === '') {
    return <span className="icon-emoji">📦</span>
  }
  return (
    <img
      src={url}
      alt={name}
      className="service-icon"
      onError={(e) => {
        (e.target as HTMLImageElement).style.display = 'none'
      }}
    />
  )
}

// Custom styled select component
interface CustomSelectProps {
  value: string
  onChange: (value: string) => void
  options: { value: string; label: string }[]
  placeholder?: string
  allowDeselect?: boolean  // Allow selecting placeholder to clear value
}

function CustomSelect({value, onChange, options, placeholder = 'Select...', allowDeselect = true}: CustomSelectProps) {
  const [isOpen, setIsOpen] = useState(false)
  const selectRef = useRef<HTMLDivElement>(null)

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (selectRef.current && !selectRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const selectedOption = options.find(o => o.value === value)
  const displayValue = selectedOption ? selectedOption.label : placeholder

  return (
    <div className="custom-select" ref={selectRef}>
      <div
        className={`custom-select-trigger ${isOpen ? 'open' : ''}`}
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className={value ? '' : 'placeholder'}>{displayValue}</span>
        <span className="custom-select-arrow">{isOpen ? '▲' : '▼'}</span>
      </div>
      {isOpen && (
        <div className="custom-select-options">
          {allowDeselect && (
            <div
              className={`custom-select-option ${!value ? 'selected' : ''}`}
              onClick={() => {
                onChange('');
                setIsOpen(false)
              }}
            >
              {placeholder}
            </div>
          )}
          {options.map(opt => (
            <div
              key={opt.value}
              className={`custom-select-option ${value === opt.value ? 'selected' : ''}`}
              onClick={() => {
                onChange(opt.value);
                setIsOpen(false)
              }}
            >
              {opt.label}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

interface ParsedMetrics {
  project: string
  plan: string
  daysInPeriod: number
  daysRemaining: number
  currentUsage: number
  estimatedMonthly: number
  dailyAverage: number
  services: ServiceMetrics[]
  scrapeSuccess: number
  scrapeDuration: number
}

// Backend JSON response types (snake_case from Rust)
interface ApiServiceData {
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

interface ApiProjectSummary {
  name: string
  current_usage_usd: number
  estimated_monthly_usd: number
  daily_average_usd: number
  days_elapsed: number
  days_remaining: number
}

interface ApiMetricsJson {
  project: ApiProjectSummary
  services: ApiServiceData[]
  scrape_timestamp: number
  scrape_duration_seconds: number
}

// /status endpoint response
interface ApiStatusResponse {
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

// WebSocket message types (matches Rust WsMessage enum)
interface WsMetricsMessage {
  type: 'metrics'
  data: ApiMetricsJson
}

interface WsStatusMessage {
  type: 'status'
  data: {
    uptime_seconds: number
    api: {
      last_success: number | null
      last_error: string | null
      total_scrapes: number
      failed_scrapes: number
    }
    ws_clients: number
  }
}

type WsMessage = WsMetricsMessage | WsStatusMessage

// Convert backend JSON to frontend format
function mapApiToMetrics(api: ApiMetricsJson): ParsedMetrics {
  const daysInPeriod = api.project.days_elapsed
  const minutesInPeriod = daysInPeriod * 24 * 60

  const services: ServiceMetrics[] = api.services.map(svc => {
    const isDeleted = svc.isDeleted || UUID_PATTERN.test(svc.name)
    return {
      name: svc.name,
      icon: svc.icon,
      group: svc.group || 'ungrouped',
      cost: svc.cost_usd,
      estimatedMonthly: svc.estimated_monthly_usd,
      cpuMinutes: svc.cpu_usage,
      memoryGbMinutes: svc.memory_usage,
      diskGbMinutes: svc.disk_usage,
      networkTxGb: svc.network_tx,
      networkRxGb: svc.network_rx,
      avgCpu: minutesInPeriod > 0 ? svc.cpu_usage / minutesInPeriod : 0,
      avgMemory: minutesInPeriod > 0 ? svc.memory_usage / minutesInPeriod : 0,
      avgDisk: minutesInPeriod > 0 ? svc.disk_usage / minutesInPeriod : 0,
      isDeleted,
    }
  }).sort((a, b) => b.cost - a.cost)

  return {
    project: api.project.name,
    plan: 'Pro', // TODO: add to backend response
    daysInPeriod,
    daysRemaining: api.project.days_remaining,
    currentUsage: api.project.current_usage_usd,
    estimatedMonthly: api.project.estimated_monthly_usd,
    dailyAverage: api.project.daily_average_usd,
    services,
    scrapeSuccess: 1,
    scrapeDuration: api.scrape_duration_seconds,
  }
}

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {style: 'currency', currency: 'USD'}).format(value)
}

function formatNumber(value: number, decimals = 2): string {
  if (value === 0) return '—'
  return value.toFixed(decimals)
}

function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}h ${m}m`
  if (m > 0) return `${m}m ${s}s`
  return `${s}s`
}

function formatInterval(seconds: number): string {
  if (seconds >= 3600) {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    return m > 0 ? `${h}h ${m}m` : `${h}h`
  }
  if (seconds >= 60) {
    const m = Math.floor(seconds / 60)
    const s = seconds % 60
    return s > 0 ? `${m}m ${s}s` : `${m}m`
  }
  return `${seconds}s`
}

// Translations for legend
const translations = {
  ru: {
    legendTitle: 'Что означают метрики?',
    cost: {
      title: 'Стоимость', items: [
        {dt: 'Cost', dd: 'Текущие расходы за биллинг-период (в USD)'},
        {dt: 'Forecast', dd: 'Прогноз на месяц при текущем потреблении'}
      ]
    },
    resources: {
      title: 'Ресурсы', items: [
        {dt: 'CPU (min)', dd: 'vCPU-минуты = ядра × время', ex: '2 ядра × 30 мин = 60'},
        {dt: 'RAM (GB·min)', dd: 'Память × время', ex: '4 GB × 60 мин = 240'},
        {dt: 'Disk (GB·min)', dd: 'Диск × время использования', ex: '10 GB × 1440 мин'}
      ]
    },
    averages: {
      title: 'Средние', items: [
        {dt: 'Avg vCPU', dd: 'Среднее число ядер', ex: '0.5 = пол-ядра'},
        {dt: 'Avg RAM', dd: 'Средняя память в GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Средний размер диска в GB'}
      ]
    },
    network: {
      title: 'Сеть', items: [
        {dt: 'TX (GB)', dd: 'Исходящий трафик — платный'},
        {dt: 'RX (GB)', dd: 'Входящий трафик — бесплатный'}
      ]
    }
  },
  en: {
    legendTitle: 'What do metrics mean?',
    cost: {
      title: 'Cost', items: [
        {dt: 'Cost', dd: 'Current billing period cost (USD)'},
        {dt: 'Forecast', dd: 'Projected monthly cost'}
      ]
    },
    resources: {
      title: 'Resources', items: [
        {dt: 'CPU (min)', dd: 'vCPU-minutes = cores × time', ex: '2 cores × 30 min = 60'},
        {dt: 'RAM (GB·min)', dd: 'Memory × time', ex: '4 GB × 60 min = 240'},
        {dt: 'Disk (GB·min)', dd: 'Disk × usage time', ex: '10 GB × 1440 min'}
      ]
    },
    averages: {
      title: 'Averages', items: [
        {dt: 'Avg vCPU', dd: 'Average cores used', ex: '0.5 = half a core'},
        {dt: 'Avg RAM', dd: 'Average memory in GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Average disk size in GB'}
      ]
    },
    network: {
      title: 'Network', items: [
        {dt: 'TX (GB)', dd: 'Outbound traffic — paid'},
        {dt: 'RX (GB)', dd: 'Inbound traffic — free'}
      ]
    }
  },
  zh: {
    legendTitle: '指标含义?',
    cost: {
      title: '费用', items: [
        {dt: 'Cost', dd: '当前计费周期费用（美元）'},
        {dt: 'Forecast', dd: '预计月度费用'}
      ]
    },
    resources: {
      title: '资源', items: [
        {dt: 'CPU (min)', dd: 'vCPU分钟 = 核心 × 时间', ex: '2核心 × 30分钟 = 60'},
        {dt: 'RAM (GB·min)', dd: '内存 × 时间', ex: '4 GB × 60分钟 = 240'},
        {dt: 'Disk (GB·min)', dd: '磁盘 × 使用时间', ex: '10 GB × 1440分钟'}
      ]
    },
    averages: {
      title: '平均值', items: [
        {dt: 'Avg vCPU', dd: '平均使用核心数', ex: '0.5 = 半个核心'},
        {dt: 'Avg RAM', dd: '平均内存（GB）', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: '平均磁盘大小（GB）'}
      ]
    },
    network: {
      title: '网络', items: [
        {dt: 'TX (GB)', dd: '出站流量 — 付费'},
        {dt: 'RX (GB)', dd: '入站流量 — 免费'}
      ]
    }
  },
  he: {
    legendTitle: 'מה המשמעות של המדדים?',
    cost: {
      title: 'עלות', items: [
        {dt: 'Cost', dd: 'עלות תקופת החיוב הנוכחית (USD)'},
        {dt: 'Forecast', dd: 'עלות חודשית צפויה'}
      ]
    },
    resources: {
      title: 'משאבים', items: [
        {dt: 'CPU (min)', dd: 'דקות vCPU = ליבות × זמן', ex: '2 ליבות × 30 דקות = 60'},
        {dt: 'RAM (GB·min)', dd: 'זיכרון × זמן', ex: '4 GB × 60 דקות = 240'},
        {dt: 'Disk (GB·min)', dd: 'דיסק × זמן שימוש', ex: '10 GB × 1440 דקות'}
      ]
    },
    averages: {
      title: 'ממוצעים', items: [
        {dt: 'Avg vCPU', dd: 'ליבות ממוצעות בשימוש', ex: '0.5 = חצי ליבה'},
        {dt: 'Avg RAM', dd: 'זיכרון ממוצע ב-GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'גודל דיסק ממוצע ב-GB'}
      ]
    },
    network: {
      title: 'רשת', items: [
        {dt: 'TX (GB)', dd: 'תעבורה יוצאת — בתשלום'},
        {dt: 'RX (GB)', dd: 'תעבורה נכנסת — חינם'}
      ]
    }
  },
  uk: {
    legendTitle: 'Що означають метрики?',
    cost: {
      title: 'Вартість', items: [
        {dt: 'Cost', dd: 'Поточні витрати за білінг-період (USD)'},
        {dt: 'Forecast', dd: 'Прогноз на місяць'}
      ]
    },
    resources: {
      title: 'Ресурси', items: [
        {dt: 'CPU (min)', dd: 'vCPU-хвилини = ядра × час', ex: '2 ядра × 30 хв = 60'},
        {dt: 'RAM (GB·min)', dd: "Пам'ять × час", ex: '4 GB × 60 хв = 240'},
        {dt: 'Disk (GB·min)', dd: 'Диск × час використання', ex: '10 GB × 1440 хв'}
      ]
    },
    averages: {
      title: 'Середні', items: [
        {dt: 'Avg vCPU', dd: 'Середня кількість ядер', ex: '0.5 = пів-ядра'},
        {dt: 'Avg RAM', dd: "Середня пам'ять в GB", ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Середній розмір диска в GB'}
      ]
    },
    network: {
      title: 'Мережа', items: [
        {dt: 'TX (GB)', dd: 'Вихідний трафік — платний'},
        {dt: 'RX (GB)', dd: 'Вхідний трафік — безкоштовний'}
      ]
    }
  },
  kk: {
    legendTitle: 'Метрикалар нені білдіреді?',
    cost: {
      title: 'Құны', items: [
        {dt: 'Cost', dd: 'Ағымдағы биллинг кезеңінің шығыны (USD)'},
        {dt: 'Forecast', dd: 'Айлық болжам'}
      ]
    },
    resources: {
      title: 'Ресурстар', items: [
        {dt: 'CPU (min)', dd: 'vCPU-минуттар = ядролар × уақыт', ex: '2 ядро × 30 мин = 60'},
        {dt: 'RAM (GB·min)', dd: 'Жады × уақыт', ex: '4 GB × 60 мин = 240'},
        {dt: 'Disk (GB·min)', dd: 'Диск × пайдалану уақыты', ex: '10 GB × 1440 мин'}
      ]
    },
    averages: {
      title: 'Орташа', items: [
        {dt: 'Avg vCPU', dd: 'Орташа ядро саны', ex: '0.5 = жарты ядро'},
        {dt: 'Avg RAM', dd: 'Орташа жады GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Орташа диск өлшемі GB'}
      ]
    },
    network: {
      title: 'Желі', items: [
        {dt: 'TX (GB)', dd: 'Шығыс трафик — ақылы'},
        {dt: 'RX (GB)', dd: 'Кіріс трафик — тегін'}
      ]
    }
  },
  de: {
    legendTitle: 'Was bedeuten die Metriken?',
    cost: {
      title: 'Kosten', items: [
        {dt: 'Cost', dd: 'Aktuelle Abrechnungsperiode (USD)'},
        {dt: 'Forecast', dd: 'Monatliche Prognose'}
      ]
    },
    resources: {
      title: 'Ressourcen', items: [
        {dt: 'CPU (min)', dd: 'vCPU-Minuten = Kerne × Zeit', ex: '2 Kerne × 30 Min = 60'},
        {dt: 'RAM (GB·min)', dd: 'Speicher × Zeit', ex: '4 GB × 60 Min = 240'},
        {dt: 'Disk (GB·min)', dd: 'Festplatte × Nutzungszeit', ex: '10 GB × 1440 Min'}
      ]
    },
    averages: {
      title: 'Durchschnitt', items: [
        {dt: 'Avg vCPU', dd: 'Durchschnittliche Kerne', ex: '0.5 = halber Kern'},
        {dt: 'Avg RAM', dd: 'Durchschnittlicher Speicher in GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Durchschnittliche Festplattengröße in GB'}
      ]
    },
    network: {
      title: 'Netzwerk', items: [
        {dt: 'TX (GB)', dd: 'Ausgehender Traffic — kostenpflichtig'},
        {dt: 'RX (GB)', dd: 'Eingehender Traffic — kostenlos'}
      ]
    }
  },
  fr: {
    legendTitle: 'Que signifient les métriques?',
    cost: {
      title: 'Coût', items: [
        {dt: 'Cost', dd: 'Coût de la période de facturation (USD)'},
        {dt: 'Forecast', dd: 'Prévision mensuelle'}
      ]
    },
    resources: {
      title: 'Ressources', items: [
        {dt: 'CPU (min)', dd: 'vCPU-minutes = cœurs × temps', ex: '2 cœurs × 30 min = 60'},
        {dt: 'RAM (GB·min)', dd: 'Mémoire × temps', ex: '4 GB × 60 min = 240'},
        {dt: 'Disk (GB·min)', dd: "Disque × temps d'utilisation", ex: '10 GB × 1440 min'}
      ]
    },
    averages: {
      title: 'Moyennes', items: [
        {dt: 'Avg vCPU', dd: 'Cœurs moyens utilisés', ex: '0.5 = demi-cœur'},
        {dt: 'Avg RAM', dd: 'Mémoire moyenne en GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Taille moyenne du disque en GB'}
      ]
    },
    network: {
      title: 'Réseau', items: [
        {dt: 'TX (GB)', dd: 'Trafic sortant — payant'},
        {dt: 'RX (GB)', dd: 'Trafic entrant — gratuit'}
      ]
    }
  },
  es: {
    legendTitle: '¿Qué significan las métricas?',
    cost: {
      title: 'Costo', items: [
        {dt: 'Cost', dd: 'Costo del período de facturación (USD)'},
        {dt: 'Forecast', dd: 'Pronóstico mensual'}
      ]
    },
    resources: {
      title: 'Recursos', items: [
        {dt: 'CPU (min)', dd: 'vCPU-minutos = núcleos × tiempo', ex: '2 núcleos × 30 min = 60'},
        {dt: 'RAM (GB·min)', dd: 'Memoria × tiempo', ex: '4 GB × 60 min = 240'},
        {dt: 'Disk (GB·min)', dd: 'Disco × tiempo de uso', ex: '10 GB × 1440 min'}
      ]
    },
    averages: {
      title: 'Promedios', items: [
        {dt: 'Avg vCPU', dd: 'Núcleos promedio usados', ex: '0.5 = medio núcleo'},
        {dt: 'Avg RAM', dd: 'Memoria promedio en GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Tamaño promedio del disco en GB'}
      ]
    },
    network: {
      title: 'Red', items: [
        {dt: 'TX (GB)', dd: 'Tráfico saliente — de pago'},
        {dt: 'RX (GB)', dd: 'Tráfico entrante — gratis'}
      ]
    }
  },
  pt: {
    legendTitle: 'O que significam as métricas?',
    cost: {
      title: 'Custo', items: [
        {dt: 'Cost', dd: 'Custo do período de faturamento (USD)'},
        {dt: 'Forecast', dd: 'Previsão mensal'}
      ]
    },
    resources: {
      title: 'Recursos', items: [
        {dt: 'CPU (min)', dd: 'vCPU-minutos = núcleos × tempo', ex: '2 núcleos × 30 min = 60'},
        {dt: 'RAM (GB·min)', dd: 'Memória × tempo', ex: '4 GB × 60 min = 240'},
        {dt: 'Disk (GB·min)', dd: 'Disco × tempo de uso', ex: '10 GB × 1440 min'}
      ]
    },
    averages: {
      title: 'Médias', items: [
        {dt: 'Avg vCPU', dd: 'Núcleos médios usados', ex: '0.5 = meio núcleo'},
        {dt: 'Avg RAM', dd: 'Memória média em GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Tamanho médio do disco em GB'}
      ]
    },
    network: {
      title: 'Rede', items: [
        {dt: 'TX (GB)', dd: 'Tráfego de saída — pago'},
        {dt: 'RX (GB)', dd: 'Tráfego de entrada — grátis'}
      ]
    }
  },
  sv: {
    legendTitle: 'Vad betyder mätvärdena?',
    cost: {
      title: 'Kostnad', items: [
        {dt: 'Cost', dd: 'Aktuell faktureringsperiod (USD)'},
        {dt: 'Forecast', dd: 'Månadsprognos'}
      ]
    },
    resources: {
      title: 'Resurser', items: [
        {dt: 'CPU (min)', dd: 'vCPU-minuter = kärnor × tid', ex: '2 kärnor × 30 min = 60'},
        {dt: 'RAM (GB·min)', dd: 'Minne × tid', ex: '4 GB × 60 min = 240'},
        {dt: 'Disk (GB·min)', dd: 'Disk × användningstid', ex: '10 GB × 1440 min'}
      ]
    },
    averages: {
      title: 'Genomsnitt', items: [
        {dt: 'Avg vCPU', dd: 'Genomsnittliga kärnor', ex: '0.5 = halv kärna'},
        {dt: 'Avg RAM', dd: 'Genomsnittligt minne i GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Genomsnittlig diskstorlek i GB'}
      ]
    },
    network: {
      title: 'Nätverk', items: [
        {dt: 'TX (GB)', dd: 'Utgående trafik — betalas'},
        {dt: 'RX (GB)', dd: 'Inkommande trafik — gratis'}
      ]
    }
  },
  no: {
    legendTitle: 'Hva betyr metrikkene?',
    cost: {
      title: 'Kostnad', items: [
        {dt: 'Cost', dd: 'Gjeldende faktureringsperiode (USD)'},
        {dt: 'Forecast', dd: 'Månedlig prognose'}
      ]
    },
    resources: {
      title: 'Ressurser', items: [
        {dt: 'CPU (min)', dd: 'vCPU-minutter = kjerner × tid', ex: '2 kjerner × 30 min = 60'},
        {dt: 'RAM (GB·min)', dd: 'Minne × tid', ex: '4 GB × 60 min = 240'},
        {dt: 'Disk (GB·min)', dd: 'Disk × brukstid', ex: '10 GB × 1440 min'}
      ]
    },
    averages: {
      title: 'Gjennomsnitt', items: [
        {dt: 'Avg vCPU', dd: 'Gjennomsnittlige kjerner', ex: '0.5 = halv kjerne'},
        {dt: 'Avg RAM', dd: 'Gjennomsnittlig minne i GB', ex: '0.25 = 256 MB'},
        {dt: 'Avg Disk', dd: 'Gjennomsnittlig diskstørrelse i GB'}
      ]
    },
    network: {
      title: 'Nettverk', items: [
        {dt: 'TX (GB)', dd: 'Utgående trafikk — betalt'},
        {dt: 'RX (GB)', dd: 'Innkommende trafikk — gratis'}
      ]
    }
  }
}

function StatCard({title, value, subtitle, color = '#1a73e8', updated = false}: {
  title: string;
  value: string;
  subtitle?: string;
  color?: string;
  updated?: boolean;
}) {
  return (
    <div className={`stat-card ${updated ? 'updated' : ''}`}>
      <div className="stat-title">{title}</div>
      <div className="stat-value" style={{color}}>{value}</div>
      {subtitle && <div className="stat-subtitle">{subtitle}</div>}
    </div>
  )
}

function App() {
  const [metrics, setMetrics] = useState<ParsedMetrics | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null)
  const [justUpdated, setJustUpdated] = useState(false)
  const apiHost = import.meta.env.VITE_API_HOST || 'localhost:9090'
  const [metricsUrl] = useState(`http://${apiHost}/metrics`)
  const [, setWsConnected] = useState(false)
  const [wsUrl] = useState(`ws://${apiHost}/ws`)
  const [useWebSocket, setUseWebSocket] = useState(false)
  // Server status from /status endpoint
  const [serverStatus, setServerStatus] = useState<ApiStatusResponse | null>(null)
  const [uptime, setUptime] = useState<number>(0)
  const [language, setLanguage] = useState<Language>(() => {
    const urlParams = new URLSearchParams(window.location.search)
    const lang = urlParams.get('lang')
    return (lang && LANGUAGES.includes(lang as Language)) ? lang as Language : 'en'
  })

  // Shorthand for UI translations
  const t = uiTranslations[language]
  const [filterService, setFilterService] = useState('')
  const [filterGroup, setFilterGroup] = useState('')
  const [showDeleted, setShowDeleted] = useState(false)
  const [sortColumn, setSortColumn] = useState<keyof ServiceMetrics>('cost')
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc')

  // Fetch /status on init to get server config (groups, websocket_enabled, etc.)
  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const response = await fetch(`http://${apiHost}/status`)
        const status: ApiStatusResponse = await response.json()
        setServerStatus(status)
        setUptime(status.uptime_seconds)
      } catch (err) {
        console.error('Failed to fetch /status:', err)
      }
    }
    fetchStatus()
  }, [apiHost])

  // Uptime ticker - increment every second
  useEffect(() => {
    const timer = setInterval(() => {
      setUptime(prev => prev + 1)
    }, 1000)
    return () => clearInterval(timer)
  }, [])


  // Get groups from server status (config) instead of extracting from services
  const groups = useMemo(() => {
    if (serverStatus?.config.service_groups) {
      return [...serverStatus.config.service_groups].sort()
    }
    // Fallback: extract from metrics if status not loaded
    if (!metrics) return []
    const activeServices = metrics.services.filter(s => !s.isDeleted)
    return [...new Set(activeServices.map(s => s.group))].sort()
  }, [serverStatus, metrics])

  // Sort handler
  const handleSort = (column: keyof ServiceMetrics) => {
    if (sortColumn === column) {
      setSortDirection(d => d === 'asc' ? 'desc' : 'asc')
    } else {
      setSortColumn(column)
      setSortDirection('desc')
    }
  }

  // Sort indicator component - always shows arrow to prevent width jumping
  const SortIndicator = ({column}: { column: keyof ServiceMetrics }) => {
    const isActive = sortColumn === column
    const arrow = isActive ? (sortDirection === 'asc' ? '↑' : '↓') : '↕'
    return (
      <span className={`sort-indicator ${isActive ? 'active' : 'inactive'}`}>
        {arrow}
      </span>
    )
  }

  // Deleted services stats
  const deletedStats = useMemo(() => {
    if (!metrics) return {count: 0, cost: 0}
    const deleted = metrics.services.filter(s => s.isDeleted)
    return {
      count: deleted.length,
      cost: deleted.reduce((a, s) => a + s.cost, 0)
    }
  }, [metrics])

  // Filter and sort services
  const filteredServices = useMemo(() => {
    if (!metrics) return []
    const filtered = metrics.services.filter(svc => {
      // Filter out deleted services unless showDeleted is true
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
      return sortDirection === 'asc' ? (aVal as number) - (bVal as number) : (bVal as number) - (aVal as number)
    })
  }, [metrics, filterService, filterGroup, showDeleted, sortColumn, sortDirection])

  // Calculate totals for filtered services
  const filteredTotals = useMemo(() => {
    return {
      cost: filteredServices.reduce((a, s) => a + s.cost, 0),
      estimatedMonthly: filteredServices.reduce((a, s) => a + s.estimatedMonthly, 0),
      cpuMinutes: filteredServices.reduce((a, s) => a + s.cpuMinutes, 0),
      avgCpu: filteredServices.reduce((a, s) => a + s.avgCpu, 0),
      memoryGbMinutes: filteredServices.reduce((a, s) => a + s.memoryGbMinutes, 0),
      avgMemory: filteredServices.reduce((a, s) => a + s.avgMemory, 0),
      diskGbMinutes: filteredServices.reduce((a, s) => a + s.diskGbMinutes, 0),
      avgDisk: filteredServices.reduce((a, s) => a + s.avgDisk, 0),
      networkTxGb: filteredServices.reduce((a, s) => a + s.networkTxGb, 0),
      networkRxGb: filteredServices.reduce((a, s) => a + s.networkRxGb, 0),
    }
  }, [filteredServices])

  const fetchMetrics = useCallback(async () => {
    setIsRefreshing(true)
    try {
      // Fetch both metrics and status in parallel
      const [metricsRes, statusRes] = await Promise.all([
        fetch(metricsUrl, {headers: {'Accept': 'application/json'}}),
        fetch(`http://${apiHost}/status`)
      ])
      const json: ApiMetricsJson = await metricsRes.json()
      const status: ApiStatusResponse = await statusRes.json()

      setMetrics(mapApiToMetrics(json))
      setServerStatus(status)
      setUptime(status.uptime_seconds)
      setError(null)
      setLastUpdate(new Date())
      // Trigger update animation
      setJustUpdated(true)
      setTimeout(() => setJustUpdated(false), 700)
    } catch (err) {
      setError(`Failed to fetch metrics: ${err}`)
    } finally {
      setLoading(false)
      setIsRefreshing(false)
    }
  }, [metricsUrl, apiHost])

  // HTTP polling - use server's configured interval (default 5s)
  const pollIntervalMs = (serverStatus?.config.scrape_interval_seconds || 5) * 1000
  useEffect(() => {
    if (!useWebSocket) {
      fetchMetrics()
      const interval = setInterval(fetchMetrics, pollIntervalMs)
      return () => clearInterval(interval)
    }
  }, [fetchMetrics, useWebSocket, pollIntervalMs])

  // WebSocket connection
  useEffect(() => {
    if (!useWebSocket) return

    let ws: WebSocket | null = null

    const connect = () => {
      ws = new WebSocket(wsUrl)

      ws.onopen = () => {
        setWsConnected(true)
        setError(null)
      }

      ws.onmessage = (event) => {
        try {
          const msg: WsMessage = JSON.parse(event.data)

          if (msg.type === 'metrics') {
            setMetrics(mapApiToMetrics(msg.data))
            setLastUpdate(new Date())
            setLoading(false)
            // Trigger update animation
            setJustUpdated(true)
            setTimeout(() => setJustUpdated(false), 700)
          } else if (msg.type === 'status') {
            // Update uptime and API status from WebSocket
            setUptime(msg.data.uptime_seconds)
            setLastUpdate(new Date())  // Update last update time for status too
            // Update API status in serverStatus if available
            setServerStatus(prev => prev ? {
              ...prev,
              uptime_seconds: msg.data.uptime_seconds,
              api: msg.data.api
            } : prev)
          }
        } catch (err) {
          console.error('Failed to parse WS message:', err)
        }
      }

      ws.onclose = () => {
        setWsConnected(false)
        // Reconnect after 3 seconds
        setTimeout(connect, 3000)
      }

      ws.onerror = () => {
        setError('WebSocket connection failed')
        setWsConnected(false)
      }
    }

    connect()

    return () => {
      if (ws) {
        ws.close()
      }
    }
  }, [useWebSocket, wsUrl])

  return (
    <div className="app">
      {isRefreshing && <div className="loading-bar"/>}
      <header>
        <div className="header-inner">
          <div className="header-main">
            <div className="header-left">
              <h1>
                <svg className="logo-icon" width="20" height="20" viewBox="0 0 32 32">
                  <circle cx="16" cy="16" r="15" fill="#4285f4"/>
                  <path d="M4 16 L10 16 L12 10 L16 22 L20 8 L22 16 L28 16" fill="none" stroke="white" strokeWidth="2.5"
                        strokeLinecap="round" strokeLinejoin="round"/>
                  <circle cx="24" cy="8" r="5" fill="#34a853"/>
                  <text x="24" y="11" textAnchor="middle" fill="white" fontSize="7" fontWeight="bold"
                        fontFamily="Arial">$
                  </text>
                </svg>
                SpendPulse
              </h1>
              {serverStatus && (
                <>
                  <span className="project-name">{serverStatus.project_name}</span>
                  <span className={`plan-badge ${serverStatus.config.plan.toLowerCase()}`}>
                    {serverStatus.config.plan}
                  </span>
                </>
              )}
            </div>
            <div className="header-right">
              <button
                className="icon-btn refresh-btn"
                onClick={fetchMetrics}
                disabled={useWebSocket}
                title={t.refresh}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M23 4v6h-6M1 20v-6h6"/>
                  <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
                </svg>
              </button>
              <button
                className="icon-btn"
                onClick={() => window.open(`http://${apiHost}/metrics`, '_blank')}
                title={t.showRaw}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/>
                  <path d="M15 3h6v6M10 14L21 3"/>
                </svg>
              </button>
              {serverStatus?.endpoints.websocket && (
                <button
                  className={`toggle-btn ${useWebSocket ? 'active' : ''}`}
                  onClick={() => setUseWebSocket(!useWebSocket)}
                  title={useWebSocket ? t.wsRealtime : t.pollInterval.replace('{interval}', formatInterval(serverStatus?.config.scrape_interval_seconds || 5))}
                >
                  {useWebSocket ? (
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M13 10V3L4 14h7v7l9-11h-7z"/>
                    </svg>
                  ) : (
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <circle cx="12" cy="12" r="10"/>
                      <polyline points="12,6 12,12 16,14"/>
                    </svg>
                  )}
                </button>
              )}
              <div className="lang-select">
                <CustomSelect
                  value={language}
                  onChange={(val) => {
                    if (val) {
                      setLanguage(val as Language)
                      const url = new URL(window.location.href)
                      url.searchParams.set('lang', val)
                      window.history.replaceState({}, '', url.toString())
                    }
                  }}
                  options={LANGUAGES.map(lang => ({
                    value: lang,
                    label: `${LANGUAGE_FLAGS[lang]} ${lang.toUpperCase()}`
                  }))}
                  placeholder="🌐"
                  allowDeselect={false}
                />
              </div>
            </div>
          </div>

        </div>
      </header>

      {error && <div className="error-banner">{error}</div>}
      {loading && <div className="loading">Loading metrics...</div>}

      {metrics && (
        <main>
          <section className="overview">
            <div className="stats-grid">
              <StatCard
                title={t.currentSpend}
                value={formatCurrency(metrics.currentUsage)}
                subtitle={`Day ${metrics.daysInPeriod} / ${metrics.daysInPeriod + metrics.daysRemaining}`}
                color={colors.success}
                updated={justUpdated}
              />
              <StatCard
                title={t.estimatedMonthly}
                value={formatCurrency(metrics.estimatedMonthly)}
                subtitle={t.projectedTotal}
                color={colors.warning}
                updated={justUpdated}
              />
              <StatCard
                title={t.dailyAverage}
                value={formatCurrency(metrics.dailyAverage)}
                subtitle={t.perDayCost}
                updated={justUpdated}
              />
              <StatCard
                title={t.minutesElapsed}
                value={(metrics.daysInPeriod * 24 * 60).toLocaleString()}
                subtitle={`${metrics.services.length} ${t.services.toLowerCase()}`}
                color={colors.primary}
                updated={justUpdated}
              />
            </div>
            <div className="stats-grid stats-grid-secondary">
              <StatCard
                title="CPU (min)"
                value={formatNumber(filteredTotals.cpuMinutes, 0)}
                subtitle={`Avg: ${formatNumber(filteredTotals.avgCpu, 2)} vCPU`}
                color={colors.cpu}
                updated={justUpdated}
              />
              <StatCard
                title="RAM (GB·min)"
                value={formatNumber(filteredTotals.memoryGbMinutes, 0)}
                subtitle={`Avg: ${formatNumber(filteredTotals.avgMemory, 2)} GB`}
                color={colors.ram}
                updated={justUpdated}
              />
              <StatCard
                title="Disk (GB·min)"
                value={formatNumber(filteredTotals.diskGbMinutes, 0)}
                subtitle={`Avg: ${formatNumber(filteredTotals.avgDisk, 2)} GB`}
                color={colors.disk}
                updated={justUpdated}
              />
              <StatCard
                title="Network (GB)"
                value={`↑${formatNumber(filteredTotals.networkTxGb, 2)} ↓${formatNumber(filteredTotals.networkRxGb, 2)}`}
                subtitle="TX / RX"
                color={colors.network}
                updated={justUpdated}
              />
            </div>
          </section>

          <section className="legend-section" dir={language === 'he' ? 'rtl' : 'ltr'}>
            <details>
              <summary>{translations[language].legendTitle}</summary>
              <div className="legend-content">
                <div className="legend-group">
                  <h4>{translations[language].cost.title}</h4>
                  <dl>
                    {translations[language].cost.items.map((item, i) => (
                      <div key={i}>
                        <dt>{item.dt}</dt>
                        <dd>{item.dd}</dd>
                      </div>
                    ))}
                  </dl>
                </div>
                <div className="legend-group">
                  <h4>{translations[language].resources.title}</h4>
                  <dl>
                    {translations[language].resources.items.map((item, i) => (
                      <div key={i}>
                        <dt>{item.dt}</dt>
                        <dd>{item.dd}{item.ex && <em> ({item.ex})</em>}</dd>
                      </div>
                    ))}
                  </dl>
                </div>
                <div className="legend-group">
                  <h4>{translations[language].averages.title}</h4>
                  <dl>
                    {translations[language].averages.items.map((item, i) => (
                      <div key={i}>
                        <dt>{item.dt}</dt>
                        <dd>{item.dd}{item.ex && <em> ({item.ex})</em>}</dd>
                      </div>
                    ))}
                  </dl>
                </div>
                <div className="legend-group">
                  <h4>{translations[language].network.title}</h4>
                  <dl>
                    {translations[language].network.items.map((item, i) => (
                      <div key={i}>
                        <dt>{item.dt}</dt>
                        <dd>{item.dd}</dd>
                      </div>
                    ))}
                  </dl>
                </div>
              </div>
            </details>
          </section>

          <div className="services-wrapper">
            <section className="services-section" dir={language === 'he' ? 'rtl' : 'ltr'}>
              <div className="services-header">
                <h3>{t.services} <span
                  dir="ltr">({filteredServices.filter(s => !s.isDeleted).length}/{metrics.services.filter(s => !s.isDeleted).length}{showDeleted && metrics.services.some(s => s.isDeleted) ?
                  <span
                    className="deleted-count"> +{filteredServices.filter(s => s.isDeleted).length}</span> : ''})</span>
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
                        onChange={(e) => setFilterService(e.target.value)}
                        className="filter-input"
                      />
                    </div>
                    <div className="filter-divider"/>
                    <div className="filter-group">
                      <label>{t.group}:</label>
                      <CustomSelect
                        value={filterGroup}
                        onChange={setFilterGroup}
                        options={groups.map(g => ({value: g, label: g}))}
                        placeholder={t.allGroups}
                      />
                    </div>
                    <div className="filter-divider"/>
                    <label className={`filter-toggle ${showDeleted ? 'active' : ''}`}>
                      <input
                        type="checkbox"
                        checked={showDeleted}
                        onChange={(e) => setShowDeleted(e.target.checked)}
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
                    {t.service} <SortIndicator column="name"/>
                  </th>
                  <th className="sortable" onClick={() => handleSort('group')}>
                    {t.group} <SortIndicator column="group"/>
                  </th>
                  <th className="right sortable" onClick={() => handleSort('cost')}>
                    {t.cost} <SortIndicator column="cost"/>
                  </th>
                  <th className="right sortable" onClick={() => handleSort('estimatedMonthly')}>
                    {t.forecast} <SortIndicator column="estimatedMonthly"/>
                  </th>
                  <th className="right sortable" onClick={() => handleSort('cpuMinutes')}>
                    {t.cpuMin} <SortIndicator column="cpuMinutes"/>
                  </th>
                  <th className="right highlight-header sortable" onClick={() => handleSort('avgCpu')}>
                    {t.avgVcpu} <SortIndicator column="avgCpu"/>
                  </th>
                  <th className="right sortable" onClick={() => handleSort('memoryGbMinutes')}>
                    {t.ramGbMin} <SortIndicator column="memoryGbMinutes"/>
                  </th>
                  <th className="right highlight-header sortable" onClick={() => handleSort('avgMemory')}>
                    {t.avgRam} <SortIndicator column="avgMemory"/>
                  </th>
                  <th className="right sortable" onClick={() => handleSort('diskGbMinutes')}>
                    {t.diskGbMin} <SortIndicator column="diskGbMinutes"/>
                  </th>
                  <th className="right highlight-header sortable" onClick={() => handleSort('avgDisk')}>
                    {t.avgDisk} <SortIndicator column="avgDisk"/>
                  </th>
                  <th className="right sortable" onClick={() => handleSort('networkTxGb')}>
                    {t.txGb} <SortIndicator column="networkTxGb"/>
                  </th>
                  <th className="right sortable" onClick={() => handleSort('networkRxGb')}>
                    {t.rxGb} <SortIndicator column="networkRxGb"/>
                  </th>
                </tr>
                </thead>
                <tbody>
                {filteredServices.length === 0 ? (
                  <tr>
                    <td colSpan={12} className="no-results">
                      🔍 {language === 'ru' ? 'Ничего не найдено' : language === 'uk' ? 'Нічого не знайдено' : 'No results found'}
                    </td>
                  </tr>
                ) : filteredServices.map((svc) => (
                  <tr key={svc.name} className={svc.isDeleted ? 'deleted-row' : ''}>
                    <td className="service-cell" title={`Icon: ${svc.icon || 'none'}`}>
                      <ServiceIcon url={svc.icon} name={svc.name}/>
                      <span className="name">
                        {svc.isDeleted ? `${svc.name.slice(0, 8)}...` : svc.name}
                      </span>
                    </td>
                    <td title={svc.isDeleted ? 'Deleted' : svc.group}>
                      {svc.isDeleted ? '—' : <span className="group-badge">{svc.group}</span>}
                    </td>
                    <td className="right money" title={`Raw: ${svc.cost}`}>{formatCurrency(svc.cost)}</td>
                    <td className="right money"
                        title={`Raw: ${svc.estimatedMonthly}`}>{formatCurrency(svc.estimatedMonthly)}</td>
                    <td className="right" title={`Raw: ${svc.cpuMinutes}`}>{formatNumber(svc.cpuMinutes, 0)}</td>
                    <td className="right highlight" title={`Raw: ${svc.avgCpu}`}>{formatNumber(svc.avgCpu, 4)}</td>
                    <td className="right"
                        title={`Raw: ${svc.memoryGbMinutes}`}>{formatNumber(svc.memoryGbMinutes, 0)}</td>
                    <td className="right highlight"
                        title={`Raw: ${svc.avgMemory}`}>{formatNumber(svc.avgMemory, 4)}</td>
                    <td className="right" title={`Raw: ${svc.diskGbMinutes}`}>{formatNumber(svc.diskGbMinutes, 0)}</td>
                    <td className="right highlight" title={`Raw: ${svc.avgDisk}`}>{formatNumber(svc.avgDisk, 4)}</td>
                    <td className="right" title={`Raw: ${svc.networkTxGb}`}>{formatNumber(svc.networkTxGb, 4)}</td>
                    <td className="right" title={`Raw: ${svc.networkRxGb}`}>{formatNumber(svc.networkRxGb, 4)}</td>
                  </tr>
                ))}
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
                  <td className="right"><strong>{formatNumber(filteredTotals.networkRxGb, 4)}</strong></td>
                </tr>
                </tfoot>
              </table>
              </div>
            </section>
            {/* Clear button outside the section - appears to the right */}
            <button
              onClick={() => {
                setFilterService('');
                setFilterGroup('')
              }}
              className={`clear-btn-side ${filterService || filterGroup ? 'visible' : ''}`}
              title={t.clear}
            >
              ✕
            </button>
          </div>

        </main>
      )}

      <footer>
        <div className="footer-inner">
          <span className="footer-left">
            SpendPulse v{serverStatus?.version || '?'}
            {serverStatus && (
              <span className="footer-server">
                • PID {serverStatus.process.pid}
                • {serverStatus.process.memory_mb.toFixed(1)}MB
                • ⏱ {formatUptime(uptime)}
              </span>
            )}
          </span>
          <span className="footer-right">
            {serverStatus && (
              <span className="footer-api">
                Railway: {serverStatus.api.last_success ? new Date(serverStatus.api.last_success * 1000).toLocaleTimeString() : '—'}
                {metrics && <span className="footer-latency"> ({(metrics.scrapeDuration * 1000).toFixed(0)}ms)</span>}
                {serverStatus.api.failed_scrapes > 0 && (
                  <span className="footer-errors"> • {serverStatus.api.failed_scrapes} err</span>
                )}
              </span>
            )}
            {lastUpdate && (
              <span className="footer-update">
                • UI: {lastUpdate.toLocaleTimeString()}
              </span>
            )}
          </span>
        </div>
      </footer>
    </div>
  )
}

export default App
