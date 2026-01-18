interface StatCardProps {
  title: string
  value: string
  subtitle?: string
  color?: string
  updated?: boolean
}

export function StatCard({
  title,
  value,
  subtitle,
  color = '#1a73e8',
  updated = false
}: StatCardProps) {
  return (
    <div className={`stat-card ${updated ? 'updated' : ''}`}>
      <div className="stat-title">{title}</div>
      <div className="stat-value" style={{ color }}>{value}</div>
      {subtitle && <div className="stat-subtitle">{subtitle}</div>}
    </div>
  )
}
