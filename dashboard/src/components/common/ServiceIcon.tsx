// Service icon with multiple fallback strategies:
// 1. Base64 data URL from server (fastest - already inline)
// 2. Bundled SVG for known services (no network)
// 3. Generic icon fallback

import { getServiceIcon, hasServiceIcon } from '@/lib/serviceIcons'

interface ServiceIconProps {
  url: string
  name: string
}

export function ServiceIcon({ url, name }: ServiceIconProps) {
  // Fixed container to prevent layout shift
  const containerStyle = {
    display: 'inline-flex',
    width: 24,
    height: 24,
    flexShrink: 0,
    alignItems: 'center',
    justifyContent: 'center',
  } as const

  // Strategy 1: Server sent Base64 data URL - use directly (fastest)
  if (url && url.startsWith('data:')) {
    return (
      <span className="service-icon-wrapper" style={containerStyle}>
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

  // Strategy 2: Use bundled SVG icon for known services
  if (hasServiceIcon(name)) {
    return (
      <span className="service-icon-wrapper" style={containerStyle}>
        {getServiceIcon(name)}
      </span>
    )
  }

  // Strategy 3: Server sent URL (but not data URL) - use as img src
  // This is for backward compatibility during transition
  if (url) {
    return (
      <span className="service-icon-wrapper" style={containerStyle}>
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
      {getServiceIcon(name)}
    </span>
  )
}
