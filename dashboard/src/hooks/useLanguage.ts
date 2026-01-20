'use client'

import { useQueryState, parseAsStringLiteral } from 'nuqs'
import { LANGUAGES, LANGUAGE_CODES, type Language, type TextDirection } from '@/i18n/keys'

/**
 * Language state synced with URL
 * URL param: ?lang=en
 *
 * Returns all language-related data:
 * - language: current language code ('en', 'ru', etc.)
 * - locale: BCP 47 locale for formatting ('en-US', 'ru-RU', etc.)
 * - dir: text direction ('ltr' or 'rtl')
 * - flag: emoji flag
 * - setLanguage: setter function
 *
 * @param defaultLang - Initial language from server (for SSR hydration match)
 */
export function useLanguage(defaultLang: Language = 'en') {
  const [language, setLanguage] = useQueryState(
    'lang',
    parseAsStringLiteral(LANGUAGE_CODES).withDefault(defaultLang)
  )

  const lang = language as Language
  const config = LANGUAGES[lang] || LANGUAGES.en

  return {
    language: lang,
    locale: config.locale,
    dir: config.dir as TextDirection,
    flag: config.flag,
    setLanguage,
  }
}
