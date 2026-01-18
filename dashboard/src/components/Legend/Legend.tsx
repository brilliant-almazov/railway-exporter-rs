'use client'

import type { Language } from '@/i18n/keys'
import legendTranslations from '@/i18n/legend.json'

interface LegendProps {
  language: Language
}

export function Legend({ language }: LegendProps) {
  const t = legendTranslations[language] || legendTranslations.en

  return (
    <section className="legend-section" dir={language === 'he' ? 'rtl' : 'ltr'}>
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
