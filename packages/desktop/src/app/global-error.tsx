'use client';

import { useEffect } from 'react';

export const GlobalErrorRoute = ({ error, reset }: { error: Error; reset: () => void }) => {
  useEffect(() => {
    console.error('Global error caught by global-error.tsx', error);
  }, [error]);

  return (
    <html>
      <body className='flex min-h-full flex-col'>
        <main className='mx-auto max-w-5xl flex-1 px-4 py-20 text-center'>
          <h1 className='text-text-primary text-4xl font-bold'>Something went wrong</h1>
          <p className='text-text-secondary mt-4 text-lg'>{error.message}</p>
          <button onClick={reset} className='btn btn-primary mt-8'>
            Try again
          </button>
        </main>
      </body>
    </html>
  );
};

export default GlobalErrorRoute;
