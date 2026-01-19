'use client'

import { CustomSelect } from '@/components/Filters/CustomSelect'
import { Tooltip } from '@/components/Common/Tooltip'
import { LogoIcon, RefreshIcon, ExternalLinkIcon, BoltIcon, ClockIcon } from '@/components/Common/Icons'
import { formatInterval } from '@/lib/formatters'
import { LANGUAGES, LANGUAGE_CODES, type Language, type Translations } from '@/i18n/keys'
import type { ApiStatusResponse } from '@/types'

interface HeaderProps {
  serverStatus: ApiStatusResponse | null
  language: Language
  onLanguageChange: (lang: Language) => void
  useWebSocket: boolean
  onWebSocketToggle: () => void
  onRefresh: () => void
  onShowRaw: () => void
  t: Translations
}

export function Header({
  serverStatus,
  language,
  onLanguageChange,
  useWebSocket,
  onWebSocketToggle,
  onRefresh,
  onShowRaw,
  t
}: HeaderProps) {
  const wsTooltip = useWebSocket
    ? t.wsRealtime
    : t.pollInterval.replace(
        '{interval}',
        formatInterval(serverStatus?.config.scrape_interval_seconds || 5)
      )

  return (
    <header>
      <div className="header-inner">
        <div className="header-main">
          <div className="header-left">
            <h1>
              <LogoIcon className="logo-icon" />
              SpendPulse
            </h1>
            {serverStatus && (
              <>
                <span className="project-name">{serverStatus.project_name}</span>
                <span className={`plan-badge ${serverStatus.config.plan.toLowerCase()}`}>
                  {serverStatus.config.plan}
                </span>
              </>
            )}
          </div>
          <div className="header-right">
            <Tooltip content={t.refresh} position="left">
              <button
                className="icon-btn refresh-btn"
                onClick={onRefresh}
                disabled={useWebSocket}
              >
                <RefreshIcon />
              </button>
            </Tooltip>
            <Tooltip content={t.showRaw} position="left">
              <button
                className="icon-btn"
                onClick={onShowRaw}
              >
                <ExternalLinkIcon />
              </button>
            </Tooltip>
            {serverStatus?.endpoints.websocket && (
              <Tooltip content={wsTooltip} position="left">
                <button
                  className={`toggle-btn ${useWebSocket ? 'active' : ''}`}
                  onClick={onWebSocketToggle}
                >
                  {useWebSocket ? <BoltIcon /> : <ClockIcon />}
                </button>
              </Tooltip>
            )}
            <div className="lang-select">
              <CustomSelect
                value={language}
                onChange={(val) => {
                  if (val) {
                    onLanguageChange(val as Language)
                  }
                }}
                options={LANGUAGE_CODES.map(code => ({
                  value: code,
                  label: `${LANGUAGES[code].flag} ${code.toUpperCase()}`
                }))}
                placeholder="🌐"
                allowDeselect={false}
              />
            </div>
          </div>
        </div>
      </div>
    </header>
  )
}
