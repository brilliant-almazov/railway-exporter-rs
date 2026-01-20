'use client'

import { useQueryState, parseAsBoolean, parseAsString } from 'nuqs'

/**
 * URL state for table filters with snake_case parameter names
 *
 * URL params:
 * - ?search=redis        - filter by service name
 * - ?group=backend       - filter by group
 * - ?show_deleted=true   - show deleted services
 */
export function useUrlFilters() {
  // Search filter: ?search=value
  const [search, setSearch] = useQueryState(
    'search',
    parseAsString.withDefault('')
  )

  // Group filter: ?group=value
  const [group, setGroup] = useQueryState(
    'group',
    parseAsString.withDefault('')
  )

  // Show deleted: ?show_deleted=true
  const [showDeleted, setShowDeleted] = useQueryState(
    'show_deleted',
    parseAsBoolean.withDefault(false)
  )

  // Clear all filters
  const clearFilters = () => {
    setSearch(null)
    setGroup(null)
    setShowDeleted(null)
  }

  // Check if any filter is active
  const hasActiveFilters = search !== '' || group !== '' || showDeleted

  return {
    // Values
    search,
    group,
    showDeleted,
    hasActiveFilters,

    // Setters
    setSearch,
    setGroup,
    setShowDeleted,
    clearFilters,
  }
}
