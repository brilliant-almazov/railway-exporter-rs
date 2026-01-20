import { Dashboard } from '@/components/Dashboard/Dashboard'
import { fetchInitialData } from '@/lib/api.server'
import { LANGUAGE_CODES, type Language } from '@/i18n/keys'

// Force dynamic rendering - always SSR with fresh data
export const dynamic = 'force-dynamic'

// Get API host from environment or use default
const API_HOST = process.env.NEXT_PUBLIC_API_HOST || 'localhost:9090'

interface PageProps {
  searchParams: Promise<{ lang?: string }>
}

export default async function Home({ searchParams }: PageProps) {
  const params = await searchParams
  // Validate language from URL, default to 'en'
  const initialLang = (LANGUAGE_CODES.includes(params.lang as Language) ? params.lang : 'en') as Language

  // Fetch initial data on server (SSR)
  const initialData = await fetchInitialData(API_HOST)

  return <Dashboard apiHost={API_HOST} initialData={initialData} initialLang={initialLang} />
}
