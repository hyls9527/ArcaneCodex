import React from 'react'
import { render, screen, fireEvent } from '@testing-library/react'
import { DisplayConfig } from '../DisplayConfig'

describe('DisplayConfig', () => {
  it('should render theme options', () => {
    render(<DisplayConfig />)
    expect(screen.getByText('settings.display.themeLight')).toBeInTheDocument()
    expect(screen.getByText('settings.display.themeDark')).toBeInTheDocument()
    expect(screen.getByText('settings.display.themeSystem')).toBeInTheDocument()
  })

  it('should render thumbnail size selector', () => {
    render(<DisplayConfig />)
    expect(screen.getByText('settings.display.thumbnailSize')).toBeInTheDocument()
    const select = screen.getByRole('combobox')
    expect(select).toBeInTheDocument()
  })

  it('should have three thumbnail size options', () => {
    render(<DisplayConfig />)
    const options = screen.getAllByRole('option')
    expect(options).toHaveLength(3)
  })

  it('should call onChange when theme is clicked', () => {
    const onChange = vi.fn()
    render(<DisplayConfig onChange={onChange} />)
    fireEvent.click(screen.getByText('settings.display.themeDark'))
    expect(onChange).toHaveBeenCalled()
  })

  it('should call onChange when thumbnail size is changed', () => {
    const onChange = vi.fn()
    render(<DisplayConfig onChange={onChange} />)
    const select = screen.getByRole('combobox')
    fireEvent.change(select, { target: { value: '400' } })
    expect(onChange).toHaveBeenCalled()
  })

  it('should render section title', () => {
    render(<DisplayConfig />)
    expect(screen.getByText('settings.display.title')).toBeInTheDocument()
  })
})
