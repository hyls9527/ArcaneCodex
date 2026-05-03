import React from 'react'
import { render, screen } from '@testing-library/react'
import { ImageCard } from './ImageCard'

describe('ImageCard Accessibility', () => {
  const mockProps = {
    id: 1,
    src: '/test.jpg',
    fileName: 'test-image.jpg',
    aiDescription: 'A beautiful sunset over the ocean',
    tags: ['nature', 'sunset'],
    aiStatus: 'completed' as const,
  }

  it('should use AI description as alt text when available', () => {
    render(<ImageCard {...mockProps} />)
    const img = screen.getByRole('img')
    expect(img).toHaveAttribute('alt', 'A beautiful sunset over the ocean')
  })

  it('should fallback to fileName when aiDescription is not available', () => {
    render(<ImageCard {...mockProps} aiDescription={undefined} />)
    const img = screen.getByRole('img')
    expect(img).toHaveAttribute('alt', 'test-image.jpg')
  })

  it('should have proper accessibility attributes', () => {
    render(<ImageCard {...mockProps} />)
    const img = screen.getByRole('img')
    expect(img).toBeInTheDocument()
    expect(img).toHaveAttribute('alt')
    expect(img.getAttribute('alt')).toBeTruthy()
    expect(img.getAttribute('alt')!.length).toBeGreaterThan(0)
  })
})
