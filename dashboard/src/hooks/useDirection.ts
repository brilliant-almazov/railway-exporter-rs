'use client'

import { useLanguage } from './useLanguage'
import { LANGUAGES, type TextDirection } from '@/i18n/keys'

/**
 * Get text direction based on current language
 * Uses useLanguage internally
 */
export function useDirection(): TextDirection {
  const { language } = useLanguage()
  return LANGUAGES[language].dir
}
