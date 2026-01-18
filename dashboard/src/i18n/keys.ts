export const LANGUAGES = ['en', 'ru', 'uk', 'de', 'fr', 'es', 'zh', 'he', 'kk'] as const
export type Language = typeof LANGUAGES[number]

export const LANGUAGE_FLAGS: Record<Language, string> = {
  en: '🇺🇸',
  ru: '🇷🇺',
  uk: '🇺🇦',
  de: '🇩🇪',
  fr: '🇫🇷',
  es: '🇪🇸',
  zh: '🇨🇳',
  he: '🇮🇱',
  kk: '🇰🇿'
}
