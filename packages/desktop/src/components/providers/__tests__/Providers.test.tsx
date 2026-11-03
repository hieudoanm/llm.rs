import { useQueryClient } from '@tanstack/react-query';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Providers } from '../Providers';

const QueryClientProbe = () => {
  const queries = useQueryClient().getDefaultOptions().queries;
  return (
    <span>{`staleTime=${String(queries?.staleTime)} retry=${String(queries?.retry)} focus=${String(queries?.refetchOnWindowFocus)}`}</span>
  );
};

describe('Providers', () => {
  it('renders its children', () => {
    render(
      <Providers>
        <span>child</span>
      </Providers>,
    );

    expect(screen.getByText('child')).toBeInTheDocument();
  });

  it('configures the query client defaults', () => {
    render(
      <Providers>
        <QueryClientProbe />
      </Providers>,
    );

    expect(screen.getByText('staleTime=5000 retry=1 focus=false')).toBeInTheDocument();
  });
});
