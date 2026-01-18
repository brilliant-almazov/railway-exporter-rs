import { QueryClient } from '@tanstack/react-query'

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5000, // 5 seconds
      refetchInterval: 30000, // 30 seconds auto-refetch
      refetchOnWindowFocus: true,
      retry: 2,
    },
  },
})
