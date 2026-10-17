import { render, screen } from '@testing-library/react';

const { getDocMock, notFoundMock } = vi.hoisted(() => ({
  getDocMock: vi.fn(),
  notFoundMock: vi.fn(() => {
    throw new Error('NEXT_NOT_FOUND');
  }),
}));

vi.mock('@/lib/docs', () => ({ getDoc: getDocMock }));
vi.mock('next/navigation', () => ({ notFound: notFoundMock }));

import DocsIndexPage from '../page';

describe('docs index page', () => {
  beforeEach(() => getDocMock.mockReset());

  it('renders the index markdown content', () => {
    getDocMock.mockReturnValue({ content: '# Documentation overview' });
    render(<DocsIndexPage />);
    expect(screen.getByRole('heading', { name: 'Documentation overview' })).toBeInTheDocument();
    expect(getDocMock).toHaveBeenCalledWith('index');
  });

  it('uses Next not-found handling when the index document is missing', () => {
    getDocMock.mockReturnValue(undefined);
    expect(() => DocsIndexPage()).toThrow('NEXT_NOT_FOUND');
    expect(notFoundMock).toHaveBeenCalledOnce();
  });
});
