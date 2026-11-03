import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { JobMetadataEditor, isValidTag, normaliseTag } from '../JobMetadataEditor';
import { ApiError } from '@/lib/api';

describe('normaliseTag', () => {
  it('trims and lowercases the tag', () => {
    expect(normaliseTag('  Baseline ')).toBe('baseline');
  });
});

describe('isValidTag', () => {
  it('accepts letters, digits, spaces, and - _ .', () => {
    expect(isValidTag('baseline run-1')).toBe(true);
    expect(isValidTag('v1.2_final')).toBe(true);
  });

  it('rejects a tag with other characters', () => {
    expect(isValidTag('bad/tag')).toBe(false);
    expect(isValidTag('')).toBe(false);
  });

  it('rejects a tag above the length limit', () => {
    expect(isValidTag('a'.repeat(33))).toBe(false);
  });
});

describe('JobMetadataEditor', () => {
  it('shows the current name and tags', () => {
    render(<JobMetadataEditor jobId='j1' name='baseline' tags={['baseline']} onSave={vi.fn()} />);

    expect(screen.getByLabelText('Name')).toHaveValue('baseline');
    expect(screen.getByText('baseline')).toBeInTheDocument();
  });

  it('adds a tag', async () => {
    render(<JobMetadataEditor jobId='j1' name={null} tags={[]} onSave={vi.fn()} />);

    await userEvent.type(screen.getByLabelText('Add a tag'), 'urgent');
    await userEvent.click(screen.getByRole('button', { name: 'Add' }));

    expect(screen.getByText('urgent')).toBeInTheDocument();
  });

  it('adds a tag when Enter is pressed', async () => {
    render(<JobMetadataEditor jobId='j1' name={null} tags={[]} onSave={vi.fn()} />);

    await userEvent.type(screen.getByLabelText('Add a tag'), 'urgent{Enter}');

    expect(screen.getByText('urgent')).toBeInTheDocument();
  });

  it('rejects an invalid tag', async () => {
    render(<JobMetadataEditor jobId='j1' name={null} tags={[]} onSave={vi.fn()} />);

    await userEvent.type(screen.getByLabelText('Add a tag'), 'bad/tag');
    await userEvent.click(screen.getByRole('button', { name: 'Add' }));

    expect(screen.getByRole('alert')).toHaveTextContent('A tag may hold letters');
  });

  it('removes a tag', async () => {
    render(<JobMetadataEditor jobId='j1' name={null} tags={['baseline']} onSave={vi.fn()} />);

    await userEvent.click(screen.getByRole('button', { name: 'Remove tag baseline' }));

    expect(screen.queryByText('baseline')).not.toBeInTheDocument();
  });

  it('saves the name and the tags', async () => {
    const onSave = vi.fn().mockResolvedValue({ name: 'run', tags: ['a'] });
    render(<JobMetadataEditor jobId='j1' name={null} tags={[]} onSave={onSave} />);

    await userEvent.type(screen.getByLabelText('Name'), 'run');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSave).toHaveBeenCalledWith({ name: 'run', tags: [] });
    expect(await screen.findByText('Saved.')).toBeInTheDocument();
  });

  it('sends a null name when the field is cleared', async () => {
    const onSave = vi.fn().mockResolvedValue({ name: null, tags: [] });
    render(<JobMetadataEditor jobId='j1' name='old' tags={[]} onSave={onSave} />);

    await userEvent.clear(screen.getByLabelText('Name'));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSave).toHaveBeenCalledWith({ name: null, tags: [] });
  });

  it('shows the error of a failed save', async () => {
    const onSave = vi.fn().mockRejectedValue(new ApiError('HTTP_ERROR', 'Name is too long', 422));
    render(<JobMetadataEditor jobId='j1' name={null} tags={[]} onSave={onSave} />);

    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Name is too long');
  });
});
