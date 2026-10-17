import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { JobFilterBar } from '../JobFilterBar';
import { EMPTY_FILTERS, type JobFilters } from '@/lib/jobs';

const renderBar = (filters: JobFilters = EMPTY_FILTERS, onChange = vi.fn()) => {
  render(
    <JobFilterBar
      filters={filters}
      onChange={onChange}
      algorithmIds={['logistic_regression']}
      knownTags={['baseline']}
    />,
  );
  return onChange;
};

describe('JobFilterBar', () => {
  it('renders the search field and the filter controls', () => {
    renderBar();

    expect(screen.getByRole('searchbox', { name: 'Search jobs' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Filter by status' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Filter by task' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Sort jobs' })).toBeInTheDocument();
  });

  it('reports the search term', async () => {
    const onChange = renderBar();

    await userEvent.type(screen.getByRole('searchbox', { name: 'Search jobs' }), 'i');

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ q: 'i' }));
  });

  it('toggles a status value', async () => {
    const onChange = renderBar();

    await userEvent.click(screen.getByRole('button', { name: 'Filter by status' }));
    await userEvent.click(screen.getByRole('checkbox', { name: 'completed' }));

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ status: ['completed'] }));
  });

  it('toggles a tag value', async () => {
    const onChange = renderBar();

    await userEvent.click(screen.getByRole('button', { name: 'Filter by tag' }));
    await userEvent.click(screen.getByRole('checkbox', { name: 'baseline' }));

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ tag: ['baseline'] }));
  });

  it('lists the active filters as removable chips', () => {
    renderBar({ ...EMPTY_FILTERS, status: ['completed'], q: 'iris' });

    expect(
      screen.getByRole('button', { name: 'Remove filter Status: completed' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Remove filter Search: iris' })).toBeInTheDocument();
  });

  it('removes a chip when it is clicked', async () => {
    const onChange = renderBar({ ...EMPTY_FILTERS, status: ['completed'] });

    await userEvent.click(screen.getByRole('button', { name: 'Remove filter Status: completed' }));

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ status: [] }));
  });

  it('clears every filter at once', async () => {
    const onChange = renderBar({ ...EMPTY_FILTERS, status: ['completed'] });

    await userEvent.click(screen.getByRole('button', { name: 'Clear all' }));

    expect(onChange).toHaveBeenCalledWith(EMPTY_FILTERS);
  });

  it('changes the sort order', async () => {
    const onChange = renderBar();

    await userEvent.click(screen.getByRole('button', { name: 'Sort jobs' }));
    await userEvent.click(screen.getByRole('button', { name: 'Created' }));

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ sort: 'createdAt:desc' }));
  });

  it('shows how many values are selected', async () => {
    renderBar({ ...EMPTY_FILTERS, status: ['completed', 'failed'] });

    expect(screen.getByRole('button', { name: 'Filter by status' })).toHaveTextContent('2');
  });
});
