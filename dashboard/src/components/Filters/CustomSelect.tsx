'use client'

import { useState, useRef, useEffect } from 'react'

interface Option {
  value: string
  label: string
}

interface CustomSelectProps {
  value: string
  onChange: (value: string) => void
  options: Option[]
  placeholder?: string
  allowDeselect?: boolean
}

export function CustomSelect({
  value,
  onChange,
  options,
  placeholder = 'Select...',
  allowDeselect = true
}: CustomSelectProps) {
  const [isOpen, setIsOpen] = useState(false)
  const selectRef = useRef<HTMLDivElement>(null)

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (selectRef.current && !selectRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const selectedOption = options.find(o => o.value === value)
  const displayValue = selectedOption ? selectedOption.label : placeholder

  return (
    <div className="custom-select" ref={selectRef}>
      <div
        className={`custom-select-trigger ${isOpen ? 'open' : ''}`}
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className={value ? '' : 'placeholder'}>{displayValue}</span>
        <span className="custom-select-arrow">{isOpen ? '▲' : '▼'}</span>
      </div>
      {isOpen && (
        <div className="custom-select-options">
          {allowDeselect && (
            <div
              className={`custom-select-option ${!value ? 'selected' : ''}`}
              onClick={() => {
                onChange('')
                setIsOpen(false)
              }}
            >
              {placeholder}
            </div>
          )}
          {options.map(opt => (
            <div
              key={opt.value}
              className={`custom-select-option ${value === opt.value ? 'selected' : ''}`}
              onClick={() => {
                onChange(opt.value)
                setIsOpen(false)
              }}
            >
              {opt.label}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
