// Service icon: displays URL from server or generic fallback

interface ServiceIconProps {
  url: string
  name: string
}

// Generic fallback icon (simple box with plus)
function GenericIcon() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect x="3" y="3" width="18" height="18" rx="3" fill="#e8eaed" stroke="#dadce0" strokeWidth="1"/>
      <path d="M8 12h8M12 8v8" stroke="#5f6368" strokeWidth="2" strokeLinecap="round"/>
    </svg>
  )
}

export function ServiceIcon({ url, name }: ServiceIconProps) {
  const containerStyle = {
    display: 'inline-flex',
    width: 24,
    height: 24,
    flexShrink: 0,
    alignItems: 'center',
    justifyContent: 'center',
  } as const

  // Server sent URL (base64 or regular) - use as img src
  // next/image doesn't work well with base64 data URIs or external URLs from arbitrary hosts
  if (url) {
    return (
      <span className="service-icon-wrapper" style={containerStyle}>
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={url}
          alt={name}
          width={24}
          height={24}
          loading="eager"
          decoding="sync"
          style={{ display: 'block', width: 24, height: 24 }}
        />
      </span>
    )
  }

  // Fallback: generic icon
  return (
    <span className="service-icon-wrapper" style={containerStyle}>
      <GenericIcon />
    </span>
  )
}
