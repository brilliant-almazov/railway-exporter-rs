'use client'

import Tippy from '@tippyjs/react'
import 'tippy.js/dist/tippy.css'
import {isValidElement} from 'react'
import type {ReactNode, ReactElement} from 'react'
import type {Placement} from 'tippy.js'

interface TooltipProps {
  content: ReactNode
  children: ReactNode
  placement?: Placement
  delay?: number
}

export function Tooltip({content, children, placement = 'auto', delay = 150}: TooltipProps) {
  // Tippy needs a single React element that can receive a ref
  // If children is already a valid element (like <button>), pass directly
  // Otherwise, wrap in a span
  const child = isValidElement(children)
    ? children
    : <span className="tooltip-wrapper">{children}</span>

  return (
    <Tippy
      content={content}
      placement={placement}
      delay={[delay, 0]}
      arrow={true}
      theme="custom"
      maxWidth={400}
      appendTo={() => document.body}
    >
      {child as ReactElement}
    </Tippy>
  )
}
