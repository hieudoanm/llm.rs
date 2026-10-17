import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { FileDrop } from '../FileDrop';

const makeFile = (name: string, sizeInMb = 0): File => {
  const file = new File(['content'], name, { type: 'text/csv' });
  Object.defineProperty(file, 'size', { value: sizeInMb * 1024 * 1024 });
  return file;
};

const input = (): HTMLElement => screen.getByLabelText('Upload CSV');
const dropZone = (): HTMLElement =>
  screen.getByLabelText('Upload CSV').parentElement as HTMLElement;

describe('FileDrop', () => {
  it('emits a CSV file chosen from the file input', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} />);
    const file = makeFile('dataset.csv');

    fireEvent.change(input(), { target: { files: [file] } });

    expect(onFileSelected).toHaveBeenCalledWith(file);
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('rejects a file that is not a CSV', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} />);

    fireEvent.change(input(), { target: { files: [makeFile('dataset.txt')] } });

    expect(onFileSelected).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toHaveTextContent('CSV file only');
  });

  it('rejects a CSV larger than maxSize', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} maxSize={2} />);

    fireEvent.change(input(), { target: { files: [makeFile('dataset.csv', 3)] } });

    expect(onFileSelected).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toHaveTextContent('File exceeds 2 MB');
  });

  it('accepts a CSV exactly at the size limit', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} maxSize={2} />);
    const file = makeFile('dataset.csv', 2);

    fireEvent.change(input(), { target: { files: [file] } });

    expect(onFileSelected).toHaveBeenCalledWith(file);
  });

  it('honours a custom accept list', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} accept='.tsv' />);

    fireEvent.change(input(), { target: { files: [makeFile('dataset.tsv')] } });
    expect(onFileSelected).toHaveBeenCalledTimes(1);

    fireEvent.change(input(), { target: { files: [makeFile('dataset.csv')] } });
    expect(onFileSelected).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('alert')).toHaveTextContent('CSV file only');
  });

  it('clears a previous error once a valid file is chosen', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} />);

    fireEvent.change(input(), { target: { files: [makeFile('dataset.txt')] } });
    expect(screen.getByRole('alert')).toBeInTheDocument();

    fireEvent.change(input(), { target: { files: [makeFile('dataset.csv')] } });

    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('ignores an empty file input change', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} />);

    fireEvent.change(input(), { target: { files: [] } });

    expect(onFileSelected).not.toHaveBeenCalled();
  });

  it('accepts a file dropped onto the zone', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} />);
    const file = makeFile('dataset.csv');

    fireEvent.drop(dropZone(), { dataTransfer: { files: [file] } });

    expect(onFileSelected).toHaveBeenCalledWith(file);
  });

  it('rejects an invalid dropped file', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} />);

    fireEvent.drop(dropZone(), { dataTransfer: { files: [makeFile('dataset.txt')] } });

    expect(onFileSelected).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toHaveTextContent('CSV file only');
  });

  it('ignores a drop with no files', () => {
    const onFileSelected = vi.fn();
    render(<FileDrop onFileSelected={onFileSelected} />);

    fireEvent.drop(dropZone(), { dataTransfer: { files: [] } });

    expect(onFileSelected).not.toHaveBeenCalled();
  });

  it('highlights the zone while dragging over it', () => {
    render(<FileDrop onFileSelected={vi.fn()} />);

    fireEvent.dragOver(dropZone());
    expect(dropZone()).toHaveClass('border-primary');

    fireEvent.dragLeave(dropZone());
    expect(dropZone()).not.toHaveClass('border-primary');
  });

  it('clears the highlight after a drop', () => {
    render(<FileDrop onFileSelected={vi.fn()} />);

    fireEvent.dragOver(dropZone());
    fireEvent.drop(dropZone(), { dataTransfer: { files: [makeFile('dataset.csv')] } });

    expect(dropZone()).not.toHaveClass('border-primary');
  });

  it('marks the input invalid and points at the error message', () => {
    render(<FileDrop onFileSelected={vi.fn()} />);

    fireEvent.change(input(), { target: { files: [makeFile('dataset.txt')] } });

    const error = screen.getByRole('alert');
    expect(input()).toHaveAttribute('aria-invalid', 'true');
    expect(input()).toHaveAttribute('aria-describedby', error.id);
  });

  it('disables the input when disabled', () => {
    render(<FileDrop onFileSelected={vi.fn()} disabled />);

    expect(input()).toBeDisabled();
    expect(dropZone()).toHaveClass('pointer-events-none');
  });
});
