// Consistent icons to prevent hydration mismatch
// All icons have fixed dimensions inline to prevent layout shift

interface IconProps {
  size?: number
  className?: string
}

// Common SVG props for all icons
const svgProps = {
  suppressHydrationWarning: true,
} as const

export function RefreshIcon({ size = 16, className }: IconProps) {
  return (
    <svg
      {...svgProps}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      className={className}
      style={{ display: 'block', width: size, height: size, flexShrink: 0 }}
    >
      <path d="M23 4v6h-6M1 20v-6h6" />
      <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15" />
    </svg>
  )
}

export function ExternalLinkIcon({ size = 16, className }: IconProps) {
  return (
    <svg
      {...svgProps}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      className={className}
      style={{ display: 'block', width: size, height: size, flexShrink: 0 }}
    >
      <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6" />
      <path d="M15 3h6v6M10 14L21 3" />
    </svg>
  )
}

export function BoltIcon({ size = 14, className }: IconProps) {
  return (
    <svg
      {...svgProps}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
      className={className}
      style={{ display: 'block', width: size, height: size, flexShrink: 0 }}
    >
      <path d="M13 10V3L4 14h7v7l9-11h-7z" />
    </svg>
  )
}

export function ClockIcon({ size = 14, className }: IconProps) {
  return (
    <svg
      {...svgProps}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      className={className}
      style={{ display: 'block', width: size, height: size, flexShrink: 0 }}
    >
      <circle cx="12" cy="12" r="10" />
      <polyline points="12,6 12,12 16,14" />
    </svg>
  )
}

export function LogoIcon({ size = 20, className }: IconProps) {
  return (
    <svg
      {...svgProps}
      width={size}
      height={size}
      viewBox="0 0 32 32"
      className={className}
      style={{ display: 'block', width: size, height: size, flexShrink: 0 }}
    >
      <circle cx="16" cy="16" r="15" fill="#4285f4" />
      <path
        d="M4 16 L10 16 L12 10 L16 22 L20 8 L22 16 L28 16"
        fill="none"
        stroke="white"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="24" cy="8" r="5" fill="#34a853" />
      <text
        x="24"
        y="11"
        textAnchor="middle"
        fill="white"
        fontSize="7"
        fontWeight="bold"
        fontFamily="Arial"
      >
        $
      </text>
    </svg>
  )
}
