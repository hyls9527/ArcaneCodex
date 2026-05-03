import React from 'react'
import { cn } from '@/utils/cn'

interface MainLayoutProps {
  children: React.ReactNode
  className?: string
}

export function MainLayout({ children, className }: MainLayoutProps) {
  return (
    <div className={cn(
      'flex h-screen w-screen bg-gray-50 dark:bg-dark-300',
      className
    )}>
      {children}
    </div>
  )
}
