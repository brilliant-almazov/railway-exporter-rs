// Color palette - matches CSS variables in App.css
export const colors = {
  // Primary colors
  primary: '#4285f4',
  success: '#34a853',
  warning: '#fbbc04',
  danger: '#ea4335',

  // System metrics colors
  cpu: '#1a73e8',
  ram: '#9334ea',
  disk: '#f57c00',
  network: '#00acc1',

  // Text colors
  textPrimary: '#202124',
  textSecondary: '#5f6368',
  textMuted: '#9aa0a6',

  // Background colors
  bgPrimary: '#f8f9fa',
  bgSecondary: '#f1f3f4',
  bgWhite: 'white',

  // Border colors
  border: '#e8eaed',
  borderLight: '#dadce0',
} as const

export type ColorKey = keyof typeof colors
