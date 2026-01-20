interface StatCardProps {
  title: string
  value: string
  subtitle?: string
  color?: string
  updated?: boolean
  /** Show asterisk marker when totals include deleted services */
  includesDeleted?: boolean
}

export function StatCard({
  title,
  value,
  subtitle,
  color = '#1a73e8',
  updated = false,
  includesDeleted = false
}: StatCardProps) {
  return (
    <div className={`stat-card ${updated ? 'updated' : ''}`}>
      <div className="stat-title">{title}</div>
      <div className="stat-value">
        <span style={{ color }}>{value}</span>
        {includesDeleted && (
          <span className="deleted-marker">*</span>
        )}
      </div>
      {subtitle && <div className="stat-subtitle">{subtitle}</div>}
    </div>
  )
}
