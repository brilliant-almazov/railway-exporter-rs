import type { Metadata } from 'next'
import { QueryProvider } from '@/components/providers/QueryProvider'
import { NuqsProvider } from '@/components/providers/NuqsProvider'
import './globals.css'

export const metadata: Metadata = {
  title: 'SpendPulse - Railway Cost Dashboard',
  description: 'Real-time Railway resource usage and cost monitoring dashboard',
  icons: {
    icon: { url: '/favicon.svg', type: 'image/svg+xml' },
  },
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <body>
        <NuqsProvider>
          <QueryProvider>{children}</QueryProvider>
        </NuqsProvider>
      </body>
    </html>
  )
}
