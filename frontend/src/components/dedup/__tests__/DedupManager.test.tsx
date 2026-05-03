import React from 'react'
import { render, screen, fireEvent } from '@testing-library/react'
import { DedupManager } from '../DedupManager'
import type { DuplicateGroup } from '@/lib/api'

const mockGroups: DuplicateGroup[] = [
  {
    id: 'group-1',
    images: [
      { image_id: 1, file_name: 'photo1.jpg', file_path: '/p1.jpg', width: 1920, height: 1080, file_size: 1024, thumbnail_path: '/t1.jpg' },
      { image_id: 2, file_name: 'photo2.jpg', file_path: '/p2.jpg', width: 800, height: 600, file_size: 512, thumbnail_path: '/t2.jpg' },
    ],
    image_ids: [1, 2],
    similarity: 95,
  },
  {
    id: 'group-2',
    images: [
      { image_id: 3, file_name: 'dup1.jpg', file_path: '/d1.jpg', width: 1024, height: 768, file_size: 768, thumbnail_path: '/d1t.jpg' },
      { image_id: 4, file_name: 'dup2.jpg', file_path: '/d2.jpg', width: 1024, height: 768, file_size: 768, thumbnail_path: '/d2t.jpg' },
    ],
    image_ids: [3, 4],
    similarity: 98,
  },
]

describe('DedupManager', () => {
  it('should render empty state with scan button when no groups', () => {
    render(<DedupManager groups={[]} />)
    expect(screen.getByText('dedup.title')).toBeInTheDocument()
    expect(screen.getByText('dedup.description')).toBeInTheDocument()
    expect(screen.getByText('dedup.startScan')).toBeInTheDocument()
  })

  it('should render threshold slider in empty state', () => {
    render(<DedupManager groups={[]} />)
    const slider = screen.getByRole('slider')
    expect(slider).toBeInTheDocument()
    expect(slider).toHaveValue('90')
  })

  it('should call onScan with threshold when scan button clicked', () => {
    const onScan = vi.fn()
    render(<DedupManager groups={[]} onScan={onScan} />)
    fireEvent.click(screen.getByText('dedup.startScan'))
    expect(onScan).toHaveBeenCalledWith(90)
  })

  it('should render results header when groups exist', () => {
    render(<DedupManager groups={mockGroups} />)
    expect(screen.getByText('dedup.results')).toBeInTheDocument()
    expect(screen.getByText('2')).toBeInTheDocument()
  })

  it('should show loading state', () => {
    render(<DedupManager groups={[]} isLoading={true} />)
    expect(screen.queryByText('dedup.startScan')).not.toBeInTheDocument()
  })

  it('should call onDelete with selected group ids on batch delete', () => {
    const onDelete = vi.fn()
    render(<DedupManager groups={mockGroups} onDelete={onDelete} />)

    const selectBtn = screen.getByText('dedup.selectGroup')
    fireEvent.click(selectBtn)

    expect(screen.getByText('dedup.selected')).toBeInTheDocument()
  })

  it('should navigate between groups with next/previous', () => {
    render(<DedupManager groups={mockGroups} />)

    expect(screen.getByText(/dedup.groupProgress/)).toBeInTheDocument()

    const nextBtn = screen.getByText('dedup.nextGroup')
    fireEvent.click(nextBtn)

    expect(screen.getByText(/dedup.groupProgress/)).toBeInTheDocument()
  })
})
