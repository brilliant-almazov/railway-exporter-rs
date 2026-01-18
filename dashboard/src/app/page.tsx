import { Dashboard } from '@/components/Dashboard/Dashboard'
import { fetchInitialData } from '@/lib/api.server'

// Force dynamic rendering - always SSR with fresh data
export const dynamic = 'force-dynamic'

// Get API host from environment or use default
const API_HOST = process.env.NEXT_PUBLIC_API_HOST || 'localhost:9090'

export default async function Home() {
  // Fetch initial data on server (SSR)
  const initialData = await fetchInitialData(API_HOST)

  return <Dashboard apiHost={API_HOST} initialData={initialData} />
}
