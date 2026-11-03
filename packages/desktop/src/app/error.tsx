'use client';

import { useEffect } from 'react';

export const ErrorRoute = ({ error, reset }: { error: Error; reset: () => void }) => {
  useEffect(() => {
    console.error('Route error caught by error.tsx', error);
  }, [error]);

  return (
    <main className='mx-auto max-w-5xl px-4 py-20 text-center'>
      <h1 className='text-text-primary text-4xl font-bold'>Something went wrong</h1>
      <p className='text-text-secondary mt-4 text-lg'>{error.message}</p>
      <button onClick={reset} className='btn btn-primary mt-8'>
        Try again
      </button>
    </main>
  );
};

export default ErrorRoute;
