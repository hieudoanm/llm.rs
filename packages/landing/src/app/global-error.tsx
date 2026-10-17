'use client';

export default function GlobalError({
  error,
  reset,
}: Readonly<{
  error: Error;
  reset: () => void;
}>) {
  return (
    <html>
      <body className='bg-bg text-ink'>
        <div className='flex min-h-screen flex-col items-center justify-center py-20'>
          <div className='max-w-[480px] text-center'>
            <h1 className='mb-2 text-[clamp(38px,7vw,62px)] leading-[1.02] font-[650] tracking-[-.075em]'>
              Something went wrong
            </h1>
            <p className='text-muted mb-4'>{error.message}</p>
            <button
              className='border-orange bg-orange inline-flex items-center gap-2 rounded-lg border px-[18px] py-[13px] text-xs font-[650] text-[#fff7ed] hover:bg-[#ff7955]'
              onClick={() => reset()}>
              Try again
            </button>
          </div>
        </div>
      </body>
    </html>
  );
}
