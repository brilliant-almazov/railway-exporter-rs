'use client'

import { useQueryState, parseAsStringLiteral } from 'nuqs'
import { LANGUAGES, type Language } from '@/i18n/keys'

/**
 * Language state synced with URL
 * URL param: ?lang=en
 */
export function useLanguage() {
  const [language, setLanguage] = useQueryState(
    'lang',
    parseAsStringLiteral(LANGUAGES).withDefault('en')
  )

  return { language: language as Language, setLanguage }
}
