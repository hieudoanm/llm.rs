import { render, act } from '@testing-library/react';
import { vi } from 'vitest';

const notFoundMock = vi.hoisted(() =>
  vi.fn(() => {
    throw new Error('NEXT_NOT_FOUND');
  }),
);

vi.mock('next/navigation', () => ({
  notFound: notFoundMock,
}));

import DocPage from '../page';

describe('DocPage', () => {
  beforeEach(() => {
    notFoundMock.mockClear();
  });

  it('renders doc content for valid slug', async () => {
    let result: ReturnType<typeof render> | null = null;
    await act(async () => {
      result = render(DocPage({ params: Promise.resolve({ slug: '01-getting-started' }) }));
    });
    expect(result!.container).not.toBeEmptyDOMElement();
    expect(result!.getByTestId('markdown')).toBeInTheDocument();
    expect(result!.container.textContent).toContain('Getting started');
  });

  it('renders next link for first doc (no previous)', async () => {
    let result: ReturnType<typeof render> | null = null;
    await act(async () => {
      result = render(DocPage({ params: Promise.resolve({ slug: '01-getting-started' }) }));
    });
    expect(result!.queryByText('Previous')).toBeNull();
    expect(result!.queryByText('Next')).not.toBeNull();
  });

  it('renders both previous and next links for middle doc', async () => {
    let result: ReturnType<typeof render> | null = null;
    await act(async () => {
      result = render(DocPage({ params: Promise.resolve({ slug: '02-cli' }) }));
    });
    expect(result!.queryByText('Previous')).not.toBeNull();
    expect(result!.queryByText('Next')).not.toBeNull();
  });

  it('does not render next link for last doc', async () => {
    let result: ReturnType<typeof render> | null = null;
    await act(async () => {
      result = render(DocPage({ params: Promise.resolve({ slug: '05-architecture' }) }));
    });
    expect(result!.queryByText('Next')).toBeNull();
    expect(result!.queryByText('Previous')).not.toBeNull();
  });

  it('calls notFound for missing doc', async () => {
    await expect(
      act(async () => {
        render(DocPage({ params: Promise.resolve({ slug: 'missing' }) }));
      }),
    ).rejects.toThrow('NEXT_NOT_FOUND');
    expect(notFoundMock).toHaveBeenCalled();
  });
});
