'use client'

import { useQueryState, parseAsStringLiteral } from 'nuqs'
import { LANGUAGE_CODES, type Language } from '@/i18n/keys'

/**
 * Language state synced with URL
 * URL param: ?lang=en
 */
export function useLanguage() {
  const [language, setLanguage] = useQueryState(
    'lang',
    parseAsStringLiteral(LANGUAGE_CODES).withDefault('en')
  )

  return { language: language as Language, setLanguage }
}
