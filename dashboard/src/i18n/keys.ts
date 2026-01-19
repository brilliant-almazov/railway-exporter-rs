// Language configuration - all metadata in one place
// Add new languages here with all their properties

import uiTranslations from './ui.json'

export enum TextDirection {
  LTR = 'ltr',
  RTL = 'rtl',
}

// Translation type derived from actual JSON structure
export type Translations = typeof uiTranslations.en

export interface LanguageConfig {
  code: string
  name: string
  flag: string
  dir: TextDirection
}

export const LANGUAGES: Record<string, LanguageConfig> = {
  en: { code: 'en', name: 'English', flag: '🇺🇸', dir: TextDirection.LTR },
  ru: { code: 'ru', name: 'Русский', flag: '🇷🇺', dir: TextDirection.LTR },
  uk: { code: 'uk', name: 'Українська', flag: '🇺🇦', dir: TextDirection.LTR },
  de: { code: 'de', name: 'Deutsch', flag: '🇩🇪', dir: TextDirection.LTR },
  fr: { code: 'fr', name: 'Français', flag: '🇫🇷', dir: TextDirection.LTR },
  es: { code: 'es', name: 'Español', flag: '🇪🇸', dir: TextDirection.LTR },
  zh: { code: 'zh', name: '中文', flag: '🇨🇳', dir: TextDirection.LTR },
  he: { code: 'he', name: 'עברית', flag: '🇮🇱', dir: TextDirection.RTL },
  kk: { code: 'kk', name: 'Қазақша', flag: '🇰🇿', dir: TextDirection.LTR },
} as const

export type Language = keyof typeof LANGUAGES

export const LANGUAGE_CODES = Object.keys(LANGUAGES) as Language[]

// Get text direction for a language
export function getDir(lang: string): TextDirection {
  return LANGUAGES[lang].dir
}
