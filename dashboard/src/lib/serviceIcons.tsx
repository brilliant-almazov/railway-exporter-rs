// Pre-bundled service icons to prevent async loading and layout shift
// Icons are loaded once on server and never updated

import type { ReactNode } from 'react'

// Generic fallback icon
const GenericServiceIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="3" y="3" width="18" height="18" rx="3" fill="#e8eaed" stroke="#dadce0" strokeWidth="1"/>
    <path d="M8 12h8M12 8v8" stroke="#5f6368" strokeWidth="2" strokeLinecap="round"/>
  </svg>
)

// PostgreSQL
const PostgresIcon = () => (
  <svg width="24" height="24" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
    <path d="M93.809 92.112c.785-6.533.55-7.492 5.416-6.433l1.235.108c3.742.17 8.637-.602 11.513-1.938 6.191-2.873 9.861-7.668 3.758-6.409-13.924 2.873-14.881-1.842-14.881-1.842 14.703-21.815 20.849-49.508 15.545-56.287-14.47-18.489-39.517-9.746-39.936-9.52l-.134.025c-2.751-.571-5.83-.912-9.289-.968-6.301-.104-11.082 1.652-14.535 4.41 0 0-44.09-18.168-42.082 22.848.427 8.735 12.255 66.059 26.373 48.74 5.162-6.334 10.158-11.686 10.158-11.686 2.478 1.647 5.46 2.497 8.63 2.202l.243-.019c-.074.738-.072 1.472.003 2.322-2.86 3.199-2.019 3.761-7.734 4.939-5.78 1.19-2.386 3.314-.17 3.869 2.691.676 8.9 1.634 13.107-4.279l-.215.477c1.444 1.154 2.469 7.502 2.303 13.276-.167 5.77-.272 9.713.26 12.809.533 3.092 1.073 5.982 3.394 7.519 2.322 1.537 4.102 2.558 10.694 1.258 5.486-1.082 8.319-2.406 8.834-8.9.369-4.655.575-7.923.946-12.455z" fill="#336791"/>
    <path d="M67.198 72.578l-.18-.063c-2.86 3.199-2.019 3.761-7.734 4.939-5.78 1.19-2.386 3.314-.17 3.869 2.691.676 8.9 1.634 13.107-4.279l-.215.477s.166-1.136.22-2.914c.056-1.778-.272-3.576-.272-3.576-1.447 1.456-3.217 1.649-4.756 1.547z" fill="#fff"/>
  </svg>
)

// Redis
const RedisIcon = () => (
  <svg width="24" height="24" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
    <path fill="#A41E11" d="M121.8 93.1c-6.7 3.5-41.4 17.7-48.8 21.6-7.4 3.9-11.5 3.8-17.3 1S13 98.1 6.3 94.9c-3.3-1.6-5-2.9-5-4.2V78.2s48-10.5 55.8-13.2c7.8-2.8 10.4-2.9 17-.5s46.1 9.5 52.6 11.9v12.5c0 1.3-1.5 2.7-4.9 4.2z"/>
    <path fill="#D82C20" d="M121.8 80.5C115.1 84 80.4 98.2 73 102.1c-7.4 3.9-11.5 3.8-17.3 1-5.8-2.8-42.7-17.7-49.4-20.9C-.1 79-.5 76.8 6 74.3c6.5-2.6 43.2-17 51-19.7 7.8-2.8 10.4-2.9 17-.5s41.1 16.1 47.6 18.5c6.6 2.4 6.9 4.4.2 7.9z"/>
    <path fill="#A41E11" d="M121.8 72.5c-6.7 3.5-41.4 17.7-48.8 21.6-7.4 3.9-11.5 3.8-17.3 1S13 77.4 6.3 74.3c-3.3-1.6-5-2.9-5-4.2V57.6s48-10.5 55.8-13.2c7.8-2.8 10.4-2.9 17-.5s46.1 9.5 52.6 11.9v12.5c0 1.3-1.5 2.7-4.9 4.2z"/>
    <path fill="#D82C20" d="M121.8 59.8c-6.7 3.5-41.4 17.7-48.8 21.6-7.4 3.9-11.5 3.8-17.3 1-5.8-2.8-42.7-17.7-49.4-21C-.1 58.3-.5 56.1 6 53.6c6.5-2.6 43.2-17 51-19.7 7.8-2.8 10.4-2.9 17-.5s41.1 16.1 47.6 18.5c6.6 2.5 6.9 4.5.2 7.9z"/>
    <path fill="#A41E11" d="M121.8 51c-6.7 3.5-41.4 17.7-48.8 21.6-7.4 3.9-11.5 3.8-17.3 1S13 55.9 6.3 52.8c-3.3-1.6-5-2.9-5-4.2V36.1s48-10.5 55.8-13.2c7.8-2.8 10.4-2.9 17-.5s46.1 9.5 52.6 11.9v12.5c0 1.3-1.5 2.6-4.9 4.2z"/>
    <path fill="#D82C20" d="M121.8 38.3c-6.7 3.5-41.4 17.7-48.8 21.6-7.4 3.9-11.5 3.8-17.3 1-5.8-2.8-42.7-17.7-49.4-21C-.1 36.8-.5 34.6 6 32.1c6.5-2.6 43.2-17 51-19.7 7.8-2.8 10.4-2.9 17-.5s41.1 16.1 47.6 18.5c6.6 2.5 6.9 4.5.2 7.9z"/>
  </svg>
)

// RabbitMQ
const RabbitMQIcon = () => (
  <svg width="24" height="24" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
    <path fill="#F60" d="M117.1 55.3H97.7c-1.5 0-2.7-1.2-2.7-2.7V36.2c0-3.5-2.9-6.4-6.4-6.4H72.2c-3.5 0-6.4 2.9-6.4 6.4v16.4c0 1.5-1.2 2.7-2.7 2.7H51c-1.5 0-2.7-1.2-2.7-2.7V36.2c0-3.5-2.9-6.4-6.4-6.4H25.5c-3.5 0-6.4 2.9-6.4 6.4v55.5c0 3.5 2.9 6.4 6.4 6.4h91.6c3.5 0 6.4-2.9 6.4-6.4V61.7c0-3.5-2.9-6.4-6.4-6.4zM42.4 81.6c0 2.3-1.9 4.2-4.2 4.2h-7.5c-2.3 0-4.2-1.9-4.2-4.2V74c0-2.3 1.9-4.2 4.2-4.2h7.5c2.3 0 4.2 1.9 4.2 4.2v7.6z"/>
  </svg>
)

// Prometheus
const PrometheusIcon = () => (
  <svg width="24" height="24" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
    <path fill="#E6522C" d="M64 0C28.7 0 0 28.7 0 64s28.7 64 64 64 64-28.7 64-64S99.3 0 64 0zm0 119.5c-6.9 0-12.5-3.8-12.5-8.4h25c0 4.6-5.6 8.4-12.5 8.4zm20.7-13.3H43.3v-9.8h41.4v9.8zm-.3-15.5H43.6c-.2-.3-.4-.6-.6-.8-6.1-7.8-7.5-11.7-8.8-15.5-1.4-4.1-.9-5.8-.9-5.8s1.8 2.4 4.9 4.6c5.2 3.6 11.3 5.5 16 9.3 4.4 3.6 6 9.5 10.2 8.2 4.2-1.3 6-7.5 10-8.2 4-0.7 10.5-3.3 14.8-7.9s6.8-11.1 6.8-11.1-0.9 6.8-6.2 14.3c-3.4 4.8-8.3 8.8-7.2 12.9z"/>
    <path fill="#E6522C" d="M65.3 20.3c-5 0-9 6.3-9 14.1s4 14.1 9 14.1 9-6.3 9-14.1-4-14.1-9-14.1zm0 23.5c-2.8 0-5-4.2-5-9.4s2.2-9.4 5-9.4 5 4.2 5 9.4-2.2 9.4-5 9.4z"/>
  </svg>
)

// Grafana
const GrafanaIcon = () => (
  <svg width="24" height="24" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
    <path fill="#F46800" d="M117.3 70.5c-.7-3.6-2.2-6.9-4.2-9.8-1.4-2-3.1-3.8-5-5.3.2-1.7.2-3.4 0-5.1-.9-5.7-4.1-10.7-8.7-14-2.5-1.8-5.4-3.1-8.4-3.7-.5-2.6-1.5-5.1-2.9-7.3-3.4-5.4-9.2-8.7-15.4-8.7-2.3 0-4.6.4-6.8 1.3-2.9-2.5-6.5-4.2-10.3-4.8-7.2-1.2-14.5 1.6-19.2 7.3-2.6-.4-5.3-.3-7.9.4-6.5 1.7-11.8 6.3-14.3 12.4-3.7 1-7.1 2.9-9.9 5.5-5.7 5.3-8.3 13-6.9 20.4-2.4 2.6-4.1 5.8-5 9.2-2.2 8.2 1 17 8 22.1 1.1 4.4 3.5 8.4 6.8 11.4 5.4 4.9 12.8 7 20 5.6 2.3 2.3 5.1 4.1 8.1 5.2 7.4 2.8 15.7 1.5 21.9-3.4 3.8 1.4 7.9 1.8 11.9 1.1 7.4-1.2 13.7-5.8 17-12.1 4.1.1 8.2-.9 11.8-3 6.6-3.8 10.5-10.9 10.4-18.5 3.9-2.8 6.7-6.9 7.9-11.5.6-2.3.8-4.6.6-6.9.2-.3.3-.6.4-.8z"/>
    <path fill="#FFF" d="M68.4 42.2c-1.1-3.9-4.1-7-8-8.1-3.9-1.1-8.1 0-11 3-2.9 3-3.9 7.2-2.8 11.1s4.1 7 8 8.1c1.3.4 2.6.5 3.9.4l.5 6.8c-2.3.3-4.6.2-6.9-.4-6.2-1.8-10.9-6.5-12.6-12.7s.1-12.9 4.6-17.4c4.5-4.6 11-6.3 17.2-4.6 6.2 1.8 10.9 6.5 12.6 12.7.7 2.3.8 4.7.6 7.1l-6.7-.6c.2-1.8.1-3.6-.4-5.4z"/>
    <path fill="#FFF" d="M95.5 61.4l-5.2 4.5c-1.3-1.5-2.9-2.6-4.7-3.4-3.6-1.5-7.7-1.2-11.1.8-3.4 2-5.6 5.5-6 9.4-.3 3.9 1.1 7.8 4 10.4 2.8 2.7 6.7 3.9 10.5 3.3l1 6.8c-5.7.9-11.5-.9-15.8-4.9-4.3-4.1-6.5-9.8-6-15.6s3.9-11.1 9.2-14.2c5.3-3.1 11.7-3.5 17.3-1.1 2.9 1.2 5.4 3.2 7.3 5.7-.2.9-.4 1.7-.5 2.3z"/>
  </svg>
)

// Nginx
const NginxIcon = () => (
  <svg width="24" height="24" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
    <path fill="#009639" d="M64 0L6.4 33.4v61.2L64 128l57.6-33.4V33.4L64 0zm29.7 93.1l-12.6 7.3c-1.6.9-3.6.5-4.8-.9L55 71.4v28.1c0 2.1-1.7 3.9-3.9 3.9H38c-2.2 0-3.9-1.7-3.9-3.9V28.4c0-2.1 1.7-3.9 3.9-3.9h13.1c2.2 0 3.9 1.7 3.9 3.9v28.2l21.3-28.1c1.2-1.5 3.2-1.8 4.8-.9l12.6 7.3c1.6.9 2.1 3 1.1 4.6l-24.2 32 24.2 32c1 1.6.5 3.7-1.1 4.6z"/>
  </svg>
)

// VictoriaMetrics
const VictoriaMetricsIcon = () => (
  <svg width="24" height="24" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
    <circle cx="64" cy="64" r="60" fill="#621773"/>
    <path d="M30 80L50 40L70 70L90 30" stroke="#fff" strokeWidth="8" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
  </svg>
)

// Railway (generic)
const RailwayIcon = () => (
  <svg width="24" height="24" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
    <rect width="128" height="128" rx="16" fill="#0B0D0E"/>
    <path d="M40 32h48v64H40z" fill="#fff"/>
    <path d="M48 48h32M48 64h32M48 80h32" stroke="#0B0D0E" strokeWidth="4"/>
  </svg>
)

// Map of service names to icons
const SERVICE_ICONS: Record<string, () => ReactNode> = {
  // Databases
  postgres: PostgresIcon,
  postgresql: PostgresIcon,
  redis: RedisIcon,

  // Messaging
  rabbitmq: RabbitMQIcon,

  // Monitoring
  prometheus: PrometheusIcon,
  grafana: GrafanaIcon,
  victoriametrics: VictoriaMetricsIcon,

  // Web servers
  nginx: NginxIcon,

  // Railway
  railway: RailwayIcon,
}

// Try to match service name to an icon
function matchServiceIcon(name: string): (() => ReactNode) | null {
  const lowerName = name.toLowerCase()

  // Direct match
  if (SERVICE_ICONS[lowerName]) {
    return SERVICE_ICONS[lowerName]
  }

  // Partial match
  for (const [key, icon] of Object.entries(SERVICE_ICONS)) {
    if (lowerName.includes(key) || key.includes(lowerName)) {
      return icon
    }
  }

  return null
}

export function getServiceIcon(name: string, _iconUrl?: string): ReactNode {
  // Try to match by service name first
  const matchedIcon = matchServiceIcon(name)
  if (matchedIcon) {
    return matchedIcon()
  }

  // Fallback to generic icon
  return <GenericServiceIcon />
}

// Check if we have a bundled icon for this service name
export function hasServiceIcon(name: string): boolean {
  return matchServiceIcon(name) !== null
}

export { GenericServiceIcon }
