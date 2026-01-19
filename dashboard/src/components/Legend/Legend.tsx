'use client'

import { useDirection } from '@/hooks/useDirection'
import { type Language } from '@/i18n/keys'
import legendTranslations from '@/i18n/legend.json'

interface LegendProps {
  language: Language
  isCompact?: boolean
}

export function Legend({ language, isCompact = false }: LegendProps) {
  const dir = useDirection()
  const t = (legendTranslations as Record<Language, typeof legendTranslations.en>)[language] || legendTranslations.en
  const className = isCompact ? 'legend-section hidden' : 'legend-section'

  return (
    <section className={className} dir={dir}>
      <details>
        <summary>{t.legendTitle}</summary>
        <div className="legend-content">
          <div className="legend-group">
            <h4>{t.cost.title}</h4>
            <dl>
              {t.cost.items.map((item, i) => (
                <div key={i}>
                  <dt>{item.dt}</dt>
                  <dd>{item.dd}</dd>
                </div>
              ))}
            </dl>
          </div>
          <div className="legend-group">
            <h4>{t.resources.title}</h4>
            <dl>
              {t.resources.items.map((item, i) => (
                <div key={i}>
                  <dt>{item.dt}</dt>
                  <dd>
                    {item.dd}
                    {item.ex && <em> ({item.ex})</em>}
                  </dd>
                </div>
              ))}
            </dl>
          </div>
          <div className="legend-group">
            <h4>{t.averages.title}</h4>
            <dl>
              {t.averages.items.map((item, i) => (
                <div key={i}>
                  <dt>{item.dt}</dt>
                  <dd>
                    {item.dd}
                    {item.ex && <em> ({item.ex})</em>}
                  </dd>
                </div>
              ))}
            </dl>
          </div>
          <div className="legend-group">
            <h4>{t.network.title}</h4>
            <dl>
              {t.network.items.map((item, i) => (
                <div key={i}>
                  <dt>{item.dt}</dt>
                  <dd>{item.dd}</dd>
                </div>
              ))}
            </dl>
          </div>
        </div>
      </details>
    </section>
  )
}
